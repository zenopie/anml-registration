use cosmwasm_std::{
    to_binary, DepsMut, Deps, Env, MessageInfo, Response, StdError, Addr, CosmosMsg, WasmMsg, Uint256, StdResult
};

use crate::msg::{Snip20Msg, StakerInfoResponse, };
use crate::state::{STATE, STAKER_INFO, PARAMS, StakerInfo, UnstakeRequest};



// Helper function to calculate the accumulated reward
fn calculate_accumulated_reward(
    env: &Env,
    staker_info: &StakerInfo,
    total_erth_staked: Uint256
) -> Uint256 {
    let mut accumulated_reward = Uint256::zero();
    let time_staked: u64 = env.block.time.seconds() - staker_info.last_reward_claim.seconds();
    const SECONDS_TWO_YEARS: u64 = 60 * 60 * 24 * 365 * 2;

    if time_staked < SECONDS_TWO_YEARS {
        accumulated_reward += staker_info.staked_amount * (Uint256::from(time_staked) * Uint256::from(1000000u32)) / total_erth_staked;
    } else {
        accumulated_reward = Uint256::from(1u32);
    }
    accumulated_reward
}


// When a user stakes
pub fn try_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint256,
    compound: bool,
) -> StdResult<Response> {
    // load state and staker info
    let params = PARAMS.load(deps.storage)?;
    if info.sender != params.erth_contract {
        return Err(StdError::generic_err("coin doesn't match registered ERTH address"));
    }
    let staker_info_option:Option<StakerInfo> = STAKER_INFO.get(deps.storage, &from);
    let mut staker_info = if let Some(data) = staker_info_option {
        data
    } else {
        StakerInfo {
            staked_amount: Uint256::zero(),
            last_reward_claim: env.block.time,
            unstake_requests: None, 
        }
    };
    // Compute the rewards using the helper function
    let mut state = STATE.load(deps.storage)?;
    let accumulated_reward = calculate_accumulated_reward(&env, &staker_info, state.total_erth_staked);
    let response;
    if accumulated_reward > Uint256::zero() { 
        if compound == false {
            // Handle reward distribution here, e.g., add to staker's balance
            let msg = to_binary(&Snip20Msg::mint_msg(
                from.clone(),
                accumulated_reward,
            ))?;

            // Create the contract execution message
            let message = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: params.erth_contract.to_string(),
                code_hash: params.erth_hash.clone(),
                msg,
                funds: vec![],
            });
            response = Response::new().add_message(message);
        } else{
            // Handle reward distribution here, e.g., add to staker's balance
            let msg = to_binary(&Snip20Msg::mint_msg(
                env.contract.address,
                accumulated_reward,
            ))?;
            // Create the contract execution message
            let message = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: params.erth_contract.to_string(),
                code_hash: params.erth_hash.clone(),
                msg,
                funds: vec![],
            });
            response = Response::new().add_message(message);
            staker_info.staked_amount += accumulated_reward;
            state.total_erth_staked += accumulated_reward;
        }
    } else {
        response = Response::default();
    }
    // Update global and staker info
    state.total_erth_staked += amount;
    staker_info.staked_amount += amount;
    staker_info.last_reward_claim = env.block.time;
    STATE.save(deps.storage, &state)?;
    STAKER_INFO.insert(deps.storage, &from, &staker_info)?;
    Ok(response)
}



pub fn try_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    compound: bool,
) -> StdResult<Response> {
    // load data
    let mut state = STATE.load(deps.storage)?;
    let staker_info_option: Option<StakerInfo> = STAKER_INFO.get(deps.storage, &info.sender);

    // if staker info option has data, declare variable staker info or return an error
    let mut staker_info = staker_info_option.ok_or(StdError::generic_err("no staking info found"))?;

    // Compute the rewards using the helper function
    let accumulated_reward = calculate_accumulated_reward(&env, &staker_info, state.total_erth_staked);
    let response;
    let params = PARAMS.load(deps.storage)?;
    if compound == false {
        // Handle reward distribution here, e.g., add to staker's balance
        let msg = to_binary(&Snip20Msg::mint_msg(
            info.sender.clone(),
            accumulated_reward,
        ))?;

        // Create the contract execution message
        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: params.erth_contract.to_string(),
            code_hash: params.erth_hash,
            msg,
            funds: vec![],
        });
        response = Response::new().add_message(message);
     
    } else {
        // Handle reward distribution here, e.g., add to staker's balance
        let msg = to_binary(&Snip20Msg::mint_msg(
            env.contract.address,
            accumulated_reward,
        ))?;
        // Create the contract execution message
        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: params.erth_contract.to_string(),
            code_hash: params.erth_hash.clone(),
            msg,
            funds: vec![],
        });
        response = Response::new().add_message(message);
        // Update staker and state info
        staker_info.staked_amount += accumulated_reward;
        state.total_erth_staked += accumulated_reward;
        STATE.save(deps.storage, &state)?;
    }
    staker_info.last_reward_claim = env.block.time;
    STAKER_INFO.insert(deps.storage, &info.sender, &staker_info)?;
    Ok(response)
}


pub fn try_request_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint256,
) -> StdResult<Response> {
    // Load staker info
    let staker_info_option: Option<StakerInfo> = STAKER_INFO.get(deps.storage, &info.sender);
    let mut staker_info = staker_info_option.ok_or(StdError::generic_err("no staking info found"))?;
    // Error if they don't have enough staked to unstake this amount
    if staker_info.staked_amount < amount {
        return Err(StdError::generic_err("unstake request is more than amount staked"));
    }
    // Create a new unstake request
    let unstake_request = UnstakeRequest {
        amount: amount,
        request_time: env.block.time,
    };
    // Check if the staker has any existing unstake requests
    match &mut staker_info.unstake_requests {
        Some(requests) => {
            // If they do, push the new request to the existing list
            requests.push(unstake_request.clone());
        }
        None => {
            // If they don't, initialize a new list with the request
            staker_info.unstake_requests = Some(vec![unstake_request.clone()]);
        }
    }
    // Store the updated info
    let mut state = STATE.load(deps.storage)?;
    staker_info.staked_amount -= amount;
    state.total_erth_staked -= amount;
    STATE.save(deps.storage, &state)?;
    STAKER_INFO.insert(deps.storage, &info.sender, &staker_info)?;
    Ok(Response::default())
}

pub fn try_complete_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    // Load staker info and state
    let params = PARAMS.load(deps.storage)?;
    let staker_info_option: Option<StakerInfo> = STAKER_INFO.get(deps.storage, &info.sender);
    let mut staker_info = staker_info_option.ok_or(StdError::generic_err("no staking info found"))?;
    // Track if any request is processed and prepare a vector to hold messages
    let mut request_processed = false;
    let mut cosmos_msgs = Vec::new();
    // Process requests that have completed the cooldown into a vec of matured requests, keep the remainder
    const SECONDS_IN_SEVEN_DAYS: u64 = 60;
    if let Some(ref mut unstake_requests) = staker_info.unstake_requests.as_mut() {
        let mut matured_requests = Vec::new();
        unstake_requests.retain(|request| {
            if env.block.time.seconds() >= request.request_time.seconds() + SECONDS_IN_SEVEN_DAYS {
                matured_requests.push(request.clone()); // Assuming UnstakeRequest implements Clone
                false
            } else {
                true
            }
        });
        for request in matured_requests {
            request_processed = true;
            // Create CosmosMsg for each matured request and push to the vector
            let msg = to_binary(&Snip20Msg::transfer_snip_msg(
                info.sender.clone(),
                request.amount,
            ))?;
            let message = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: params.erth_contract.to_string(),
                code_hash: params.erth_hash.clone(),
                msg,
                funds: vec![],
            });
            cosmos_msgs.push(message);
        }
    } else {
        return Err(StdError::generic_err("no unstake request found"));
    }
    // If no request was processed, return an error
    if !request_processed {
        return Err(StdError::generic_err("cooldown not met for any request"));
    }
    STAKER_INFO.insert(deps.storage, &info.sender, &staker_info)?;
    // Build the response
    let response = Response::new().add_messages(cosmos_msgs);
    Ok(response)
}

pub fn query_stake_info(
    deps: Deps, 
    env: Env, 
    address: Addr
) -> StdResult<StakerInfoResponse> {
    let state = STATE.load(deps.storage)?;
    let staker_info_option: Option<StakerInfo> = STAKER_INFO.get(deps.storage, &address);

    // Pattern matching to handle Some and None cases
    let (staker_info, accumulated_reward) = match staker_info_option {
        Some(info) => {
            // Compute the rewards using the helper function
            let accumulated_reward = calculate_accumulated_reward(&env, &info, state.total_erth_staked);
            (Some(info), Some(accumulated_reward))
        }
        None => {
            // Handle the None case (no staking info found)
            (None, None)
        }
    };

    // Construct and send response
    let response = StakerInfoResponse {
        staker_info: staker_info,
        accumulated_reward: accumulated_reward,
        total_staked: state.total_erth_staked,
    };
    Ok(response)
}





