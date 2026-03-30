// src/execute/allocation.rs
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, Addr, to_binary, CosmosMsg, WasmMsg, Timestamp};
use crate::state::{ALLOCATION_OPTIONS, ALLOCATION_IDS, USER_ALLOCATIONS, State, CONFIG, Allocation, AllocationConfig, AllocationPercentage,
    AllocationState, STATE, REGISTRATIONS, UserAllocations, MAX_DESCRIPTION_LENGTH, query_registry};
use crate::msg::SendMsg;
use secret_toolkit::snip20::{HandleMsg};

const INDEX_PRECISION: u128 = 1_000_000_000_000;
const REWARD_RATE: u128 = 1_000_000; // 1 ERTH per second (6 decimal places)

pub fn update_reward_index(state: &mut State, current_time: Timestamp) {
    let time_elapsed = current_time.seconds().saturating_sub(state.last_upkeep.seconds());
    if time_elapsed > 0 && !state.total_allocations.is_zero() {
        let new_rewards = Uint128::from(time_elapsed) * Uint128::from(REWARD_RATE);
        state.reward_index = state.reward_index +
            (new_rewards * Uint128::from(INDEX_PRECISION) / state.total_allocations);
    }
    state.last_upkeep = current_time;
}

fn settle_allocation(allocation_state: &mut AllocationState, reward_index: Uint128) {
    if !allocation_state.amount_allocated.is_zero() {
        let delta = reward_index - allocation_state.last_reward_index;
        let pending = allocation_state.amount_allocated * delta / Uint128::from(INDEX_PRECISION);
        allocation_state.accumulated_rewards = allocation_state.accumulated_rewards + pending;
    }
    allocation_state.last_reward_index = reward_index;
}

pub fn set_allocation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    percentages: Vec<AllocationPercentage>,
) -> StdResult<Response> {
    // Load config to get registration validity period
    let config = CONFIG.load(deps.storage)?;

    // Load user registration and check validity
    if let Some(registration) = REGISTRATIONS.get_by_address(deps.storage, &info.sender)? {
        let registration_age = env.block.time.seconds() - registration.registration_timestamp.seconds();
        if registration_age > config.registration_validity_seconds {
            return Err(StdError::generic_err("Registration has expired"));
        }
    } else {
        return Err(StdError::generic_err("User not registered"));
    }

    let mut state = STATE.load(deps.storage)?;

    // Update global reward index
    update_reward_index(&mut state, env.block.time);

    // Load old user allocations, checking epoch
    let old_user_data = USER_ALLOCATIONS.get(deps.storage, &info.sender).unwrap_or_default();
    let old_allocations = if old_user_data.epoch == state.epoch {
        old_user_data.allocations
    } else {
        vec![]
    };

    // Subtract old allocations
    for old_pct in &old_allocations {
        if let Some(mut allocation) = ALLOCATION_OPTIONS.get(deps.storage, &old_pct.allocation_id) {
            settle_allocation(&mut allocation.state, state.reward_index);
            allocation.state.amount_allocated = allocation.state.amount_allocated.checked_sub(old_pct.percentage)
                .map_err(|_| StdError::generic_err("Underflow in allocation subtraction"))?;
            state.total_allocations = state.total_allocations.checked_sub(old_pct.percentage)
                .map_err(|_| StdError::generic_err("Underflow in total allocations"))?;
            ALLOCATION_OPTIONS.insert(deps.storage, &old_pct.allocation_id, &allocation)?;
        }
    }

    // Check for duplicate allocation IDs
    let mut seen_ids = std::collections::HashSet::new();
    for pct in &percentages {
        if !seen_ids.insert(pct.allocation_id) {
            return Err(StdError::generic_err("Duplicate allocation ID found"));
        }
    }

    // Add new allocations and validate
    let mut total_percentage = Uint128::zero();
    for new_pct in &percentages {
        if new_pct.percentage > Uint128::zero() {
            let mut allocation = ALLOCATION_OPTIONS.get(deps.storage, &new_pct.allocation_id)
                .ok_or_else(|| StdError::generic_err("Allocation not found"))?;
            settle_allocation(&mut allocation.state, state.reward_index);
            allocation.state.amount_allocated = allocation.state.amount_allocated.checked_add(new_pct.percentage)
                .map_err(|_| StdError::generic_err("Overflow in allocation addition"))?;
            state.total_allocations = state.total_allocations.checked_add(new_pct.percentage)
                .map_err(|_| StdError::generic_err("Overflow in total allocations"))?;
            total_percentage = total_percentage.checked_add(new_pct.percentage)
                .map_err(|_| StdError::generic_err("Overflow in total percentage"))?;
            ALLOCATION_OPTIONS.insert(deps.storage, &new_pct.allocation_id, &allocation)?;
        }
    }

    // Ensure that the total percentages add up to 100%
    if total_percentage != Uint128::from(100u32) {
        return Err(StdError::generic_err("Percentage error: allocations must sum to 100%"));
    }

    // Save user allocations with current epoch
    USER_ALLOCATIONS.insert(deps.storage, &info.sender, &UserAllocations {
        epoch: state.epoch,
        allocations: percentages,
    })?;

    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

pub fn claim_allocation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    allocation_id: u32,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // Update global reward index
    update_reward_index(&mut state, env.block.time);

    // Load and settle the specific allocation
    let mut allocation = ALLOCATION_OPTIONS.get(deps.storage, &allocation_id)
        .ok_or_else(|| StdError::generic_err("Allocation not found"))?;

    settle_allocation(&mut allocation.state, state.reward_index);

    // If there's a claimer address, check that the info.sender is the claimer
    if let Some(claimer_addr) = &allocation.config.claimer_addr {
        if &info.sender != claimer_addr {
            return Err(StdError::generic_err("Unauthorized: Only the claimer can claim this allocation"));
        }
    }

    // Get the accumulated rewards for this allocation
    let allocation_share = allocation.state.accumulated_rewards;

    if allocation_share.is_zero() {
        ALLOCATION_OPTIONS.insert(deps.storage, &allocation_id, &allocation)?;
        STATE.save(deps.storage, &state)?;
        return Ok(Response::new()
            .add_attribute("action", "claim_allocation")
            .add_attribute("allocation_id", allocation_id.to_string())
            .add_attribute("allocation_share", "0"));
    }

    // Reset accumulated rewards after claiming
    allocation.state.accumulated_rewards = Uint128::zero();

    // Update the claim time
    allocation.state.last_claim = env.block.time;

    // Query registry for erth_token
    let deps_ref = deps.as_ref();
    let contracts = query_registry(
        &deps_ref,
        &config.registry_contract,
        &config.registry_hash,
        vec!["erth_token"],
    )?;
    let erth_token = &contracts[0];

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
            contract_addr: erth_token.address.to_string(),
            code_hash: erth_token.code_hash.clone(),
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
            contract_addr: erth_token.address.to_string(),
            code_hash: erth_token.code_hash.clone(),
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
            contract_addr: erth_token.address.to_string(),
            code_hash: erth_token.code_hash.clone(),
            msg: to_binary(&mint_msg)?,
            funds: vec![],
        }));
    };

    // Save the updated allocation and state
    ALLOCATION_OPTIONS.insert(deps.storage, &allocation_id, &allocation)?;
    STATE.save(deps.storage, &state)?;

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
    let config = CONFIG.load(deps.storage)?;

    let mut allocation = ALLOCATION_OPTIONS.get(deps.storage, &allocation_id)
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

    if allocation_config.description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(StdError::generic_err(format!("Description exceeds max length of {}", MAX_DESCRIPTION_LENGTH)));
    }

    allocation.config = allocation_config;

    ALLOCATION_OPTIONS.insert(deps.storage, &allocation_id, &allocation)?;

    Ok(Response::new()
        .add_attribute("action", "edit_allocation")
        .add_attribute("allocation_id", allocation_id.to_string()))
}

pub fn add_allocation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    description: String,
    receive_addr: Addr,
    receive_hash: Option<String>,
    manager_addr: Option<Addr>,
    claimer_addr: Option<Addr>,
    use_send: bool,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if info.sender != config.contract_manager {
        return Err(StdError::generic_err("Unauthorized: Only the contract manager can add an allocation"));
    }

    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(StdError::generic_err(format!("Description exceeds max length of {}", MAX_DESCRIPTION_LENGTH)));
    }

    state.allocation_counter += 1;

    let allocation = Allocation {
        state: AllocationState {
            allocation_id: state.allocation_counter,
            amount_allocated: Uint128::zero(),
            last_claim: env.block.time,
            accumulated_rewards: Uint128::zero(),
            last_reward_index: state.reward_index,
        },
        config: AllocationConfig {
            description,
            receive_addr,
            receive_hash,
            manager_addr,
            claimer_addr,
            use_send,
        },
    };

    ALLOCATION_OPTIONS.insert(deps.storage, &state.allocation_counter, &allocation)?;

    // Add to ID list
    let mut ids = ALLOCATION_IDS.load(deps.storage).unwrap_or_default();
    ids.push(state.allocation_counter);
    ALLOCATION_IDS.save(deps.storage, &ids)?;

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "add_allocation"))
}

pub fn reset_allocations(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.contract_manager {
        return Err(StdError::generic_err("Unauthorized: Only the contract manager can reset allocations"));
    }

    let mut state = STATE.load(deps.storage)?;

    // Update global reward index one final time
    update_reward_index(&mut state, env.block.time);

    // Iterate all allocations: settle pending rewards, zero amount_allocated
    let ids = ALLOCATION_IDS.load(deps.storage).unwrap_or_default();
    for id in &ids {
        if let Some(mut allocation) = ALLOCATION_OPTIONS.get(deps.storage, id) {
            settle_allocation(&mut allocation.state, state.reward_index);
            allocation.state.amount_allocated = Uint128::zero();
            ALLOCATION_OPTIONS.insert(deps.storage, id, &allocation)?;
        }
    }

    // Reset total allocations and increment epoch
    state.total_allocations = Uint128::zero();
    state.epoch += 1;

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "reset_allocations")
        .add_attribute("epoch", state.epoch.to_string()))
}
