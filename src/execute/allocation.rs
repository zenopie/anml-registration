// src/execute/allocation.rs
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, Addr, to_binary, CosmosMsg, WasmMsg};
use crate::state::{ALLOCATION_OPTIONS, USER_ALLOCATIONS, State, CONFIG, Allocation, AllocationConfig, AllocationPercentage,
    IDS_BY_ADDRESS, AllocationState, STATE};
use crate::msg::SendMsg;
use secret_toolkit::snip20::{HandleMsg};


pub fn set_allocation(
    deps: DepsMut,
    info: MessageInfo,
    percentages: Vec<AllocationPercentage>,
) -> StdResult<Response> {
    // Load user info or return an error if no deposit is found
    if IDS_BY_ADDRESS.get(deps.storage, &info.sender).is_none() {
        return Err(StdError::generic_err("User not registered"));
    }

    let old_user_allocations = USER_ALLOCATIONS.get(deps.storage, &info.sender).unwrap_or_default();

    // Load allocation options and state
    let mut allocation_options = ALLOCATION_OPTIONS.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // Subtract the old allocations using the helper function
    subtract_old_allocations(&old_user_allocations, &mut allocation_options, &mut state)?;

    // Add the new allocations using the helper function
    add_new_allocations(&percentages, &mut allocation_options, &mut state)?;

    // Save the updated user info back to storage
    USER_ALLOCATIONS.insert(deps.storage, &info.sender, &percentages)?;

    // Save the updated allocation options and state
    ALLOCATION_OPTIONS.save(deps.storage, &allocation_options)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

fn subtract_old_allocations(
    old_percentages: &[AllocationPercentage],
    allocation_options: &mut Vec<Allocation>,
    state: &mut State,
) -> StdResult<()> {
    for old_percent in old_percentages {
        for allocation_option in allocation_options.iter_mut() {
            if old_percent.allocation_id == allocation_option.state.allocation_id {
                allocation_option.state.amount_allocated = allocation_option.state.amount_allocated.checked_sub(old_percent.percentage)
                    .map_err(|_| StdError::generic_err("Underflow in allocation subtraction"))?;
                state.total_allocations = state.total_allocations.checked_sub(old_percent.percentage)
                    .map_err(|_| StdError::generic_err("Underflow in total allocations"))?;
            }
        }
    }
    Ok(())
}

fn add_new_allocations(
    percentages: &[AllocationPercentage],
    allocation_options: &mut Vec<Allocation>,
    state: &mut State,
) -> StdResult<()> {
    let mut total_percentage = Uint128::zero();

    for percentage in percentages {
        if percentage.percentage > Uint128::zero() {
            for allocation in allocation_options.iter_mut() {
                if percentage.allocation_id == allocation.state.allocation_id {
                    allocation.state.amount_allocated = allocation.state.amount_allocated.checked_add(percentage.percentage)
                        .map_err(|_| StdError::generic_err("Overflow in allocation addition"))?;
                    state.total_allocations = state.total_allocations.checked_add(percentage.percentage)
                        .map_err(|_| StdError::generic_err("Overflow in total allocations"))?;
                    total_percentage = total_percentage.checked_add(percentage.percentage)
                        .map_err(|_| StdError::generic_err("Overflow in total percentage"))?;
                }
            }
        }
    }

    // Ensure that the total percentages add up to 100%
    if total_percentage != Uint128::from(100u32) {
        return Err(StdError::generic_err("Percentage error: allocations must sum to 100%"));
    }

    Ok(())
}

pub fn claim_allocation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    allocation_id: u32,
) -> StdResult<Response> {
    // Constants for rewards
    let reward_rate_per_second: Uint128 = Uint128::from(1_000_000u128); // 1,000,000 ERTH per second

    // Load the current state
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    // Find the allocation by ID
    let mut allocation_options = ALLOCATION_OPTIONS.load(deps.storage)?;
    let allocation = allocation_options.iter_mut().find(|alloc| alloc.state.allocation_id == allocation_id)
        .ok_or_else(|| StdError::generic_err("Allocation not found"))?;

    // If there's a claimer address, check that the info.sender is the claimer
    if let Some(claimer_addr) = &allocation.config.claimer_addr {
        if &info.sender != claimer_addr {
            return Err(StdError::generic_err("Unauthorized: Only the claimer can claim this allocation"));
        }
    }

    // Calculate the time elapsed since the last claim
    let time_elapsed = env.block.time.seconds().checked_sub(allocation.state.last_claim.seconds())
        .ok_or_else(|| StdError::generic_err("Invalid block time"))?;

    // Calculate allocation share
    if state.total_allocations.is_zero() {
        return Err(StdError::generic_err("Total allocations is zero"));
    }
    let allocation_share = allocation.state.amount_allocated * reward_rate_per_second * Uint128::from(time_elapsed) / state.total_allocations;

    let mut messages = Vec::new();

    // Prepare the minting message based on the `use_send` flag
    if allocation.config.use_send {
        // Mint to the staking contract and trigger the receive function
        let mint_msg = HandleMsg::Mint {
            recipient: env.contract.address.to_string(),
            amount: allocation_share,
            padding: None,
            memo: None,
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.erth_token_contract.to_string(),
            code_hash: config.erth_token_hash.clone(),
            msg: to_binary(&mint_msg)?,
            funds: vec![],
        }));
        
        let send_msg = if let Some(receive_hash) = &allocation.config.receive_hash {
            HandleMsg::Send {
                recipient: allocation.config.receive_addr.to_string(),
                recipient_code_hash: Some(receive_hash.clone()),
                amount: allocation_share,
                msg: Some(to_binary(&SendMsg::AllocationSend {
                    allocation_id,
                })?),
                memo: None,
                padding: None,
            }
        } else {
            return Err(StdError::generic_err("Missing recipient code hash for allocation"));
        };

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.erth_token_contract.to_string(),
            code_hash: config.erth_token_hash.clone(),
            msg: to_binary(&send_msg)?,
            funds: vec![],
        }));
    } else {
        // Mint directly to the allocation receiver address
        let mint_msg = HandleMsg::Mint {
            recipient: allocation.config.receive_addr.to_string(),
            amount: allocation_share,
            padding: None,
            memo: None,
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.erth_token_contract.to_string(),
            code_hash: config.erth_token_hash.clone(),
            msg: to_binary(&mint_msg)?,
            funds: vec![],
        }));
    };

    // Update the claim time
    allocation.state.last_claim = env.block.time;
    ALLOCATION_OPTIONS.save(deps.storage, &allocation_options)?;

    // Return the response with the mint message
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "claim_allocation")
        .add_attribute("allocation_id", allocation_id.to_string())
        .add_attribute("allocation_share", allocation_share.to_string()))
}

pub fn edit_allocation(
    deps: DepsMut,
    info: MessageInfo,
    allocation_id: u32,
    allocation_config: AllocationConfig,
) -> StdResult<Response> {
    // Load the current config
    let config = CONFIG.load(deps.storage)?;

    // Load allocation options from storage
    let mut allocations = ALLOCATION_OPTIONS.load(deps.storage)?;

    // Find the allocation by ID
    let allocation = allocations.iter_mut().find(|alloc| alloc.state.allocation_id == allocation_id)
        .ok_or_else(|| StdError::generic_err("Allocation not found"))?;

    // Check if the sender is authorized to edit the allocation
    if info.sender != config.contract_manager {
        if let Some(manager_addr) = &allocation.config.manager_addr {
            if &info.sender != manager_addr {
                return Err(StdError::generic_err("Unauthorized: Only the allocation manager or contract manager can edit this allocation"));
            }
        } else {
            return Err(StdError::generic_err("Unauthorized: Only the contract manager can edit this allocation"));
        }
    }

    allocation.config = allocation_config;

    // Save the updated allocation options back to storage
    ALLOCATION_OPTIONS.save(deps.storage, &allocations)?;

    Ok(Response::new()
        .add_attribute("action", "edit_allocation")
        .add_attribute("allocation_id", allocation_id.to_string()))
}

pub fn add_allocation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receive_addr: Addr, 
    receive_hash: Option<String>, 
    manager_addr: Option<Addr>, 
    claimer_addr: Option<Addr>, 
    use_send: bool, 
) -> StdResult<Response> {

    // Load the current config and state
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // Check if the sender is the contract manager
    if info.sender != config.contract_manager {
        return Err(StdError::generic_err("Unauthorized: Only the contract manager can add an allocation"));
    }

    // Load allocation options, or use an empty Vec if it doesn't exist
    let mut allocation_options = ALLOCATION_OPTIONS.load(deps.storage).unwrap_or_else(|_| Vec::new());
    
    state.allocation_counter += 1;

    // Create a new allocation
    let allocation_state = AllocationState {
        allocation_id: state.allocation_counter,
        amount_allocated: Uint128::zero(),
        last_claim: env.block.time,
    };

    let allocation_config = AllocationConfig {
        receive_addr,
        receive_hash,
        manager_addr,
        claimer_addr,
        use_send,
    };

    let allocation = Allocation {
        state: allocation_state,
        config: allocation_config,
    };

    // Add the new allocation to the list
    allocation_options.push(allocation);

    // Save the updated allocation options and state
    ALLOCATION_OPTIONS.save(deps.storage, &allocation_options)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "add_allocation"))
}
