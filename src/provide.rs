use cosmwasm_std::{
    to_binary, DepsMut, Env, Deps, MessageInfo, Response, StdError, Addr, CosmosMsg, WasmMsg, StdResult,
    Uint256, Timestamp, 
};

use crate::msg::{Snip20Msg, PoolInfoResponse};
use crate::state::{PARAMS, PROVIDER_INFO, ProviderInfo, POOL, Pool, RewardRateChange, UnstakeRequest};

fn ceil_div(a: Uint256, b: Uint256) -> StdResult<Uint256> {
    if b == Uint256::zero() {
        return Err(StdError::generic_err("division by zero"));
    }
    Ok((a + b - Uint256::from(1u128)) / b)
}

// function to initialize a liquidity pool
pub fn try_initialize_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    other_contract: Addr,
    other_hash: String,
    initial_anml: Uint256,
    initial_other: Uint256,
) -> StdResult<Response> {
    // Check if pool exists
    let pool_option: Option<Pool> = POOL.get(deps.storage, &other_contract);
    if pool_option.is_some() {
        return Err(StdError::generic_err("Pool already exists"));
    }
    const INITIAL_LIQUIDITY_AMOUNT: u32 = 100 * 1000000;
    if initial_anml < Uint256::from(INITIAL_LIQUIDITY_AMOUNT) {
        return Err(StdError::generic_err("Initial liquidity must include 100 ANML"));
    }
    // Initialize the pool with provided values
    let reward_rate = RewardRateChange{
        rate: Uint256::from(1000000u32), //temp test value
        since: env.block.time,
    };
    let pool = Pool {
        anml_balance: initial_anml,
        other_balance: initial_other,
        other_contract: other_contract.clone(),
        other_hash: other_hash.clone(),
        reward_rate: vec![reward_rate],
        volume: Uint256::zero(),
        shares: initial_anml,
    };

    // Save the initialized pool to storage
    POOL.insert(deps.storage, &other_contract, &pool)?;

    let provider_info = PROVIDER_INFO.add_suffix(other_contract.to_string().as_bytes());

    let provider_details = ProviderInfo {
        provide_amount: initial_anml - Uint256::from(10000000u64),
        last_claim: env.block.time,
        withdraw_requests: None,
    };

    provider_info.insert(deps.storage, &info.sender, &provider_details)?;

    let params = PARAMS.load(deps.storage)?;
    
    // Handle reward distribution here, e.g., add to staker's balance
    let msg_anml = to_binary(&Snip20Msg::transfer_from_msg(
        info.sender.clone(),
        env.contract.address.clone(),
        initial_anml,
    ))?;

    // Create the contract execution message
    let message_anml = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: params.anml_contract.to_string(),
        code_hash: params.anml_hash,
        msg: msg_anml,
        funds: vec![],
    });
    // Handle reward distribution here, e.g., add to staker's balance
    let msg_other = to_binary(&Snip20Msg::transfer_from_msg(
        info.sender.clone(),
        env.contract.address,
        initial_other,
    ))?;

    // Create the contract execution message
    let message_other = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: other_contract.to_string(),
        code_hash: other_hash,
        msg: msg_other,
        funds: vec![],
    });

    let response = Response::new()
        .add_message(message_anml)
        .add_message(message_other);
    Ok(response)
}


pub fn try_add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_id: Addr,
    anml_deposit: Uint256,
    other_deposit: Uint256,
) -> StdResult<Response> {
    // Load the pool using the other_contract as the identifier
    let mut pool = POOL.get(deps.storage, &pool_id)
        .ok_or(StdError::generic_err("Pool doesn't exist"))?;


    // Load provider information
    let provider_info = PROVIDER_INFO.add_suffix(pool_id.to_string().as_bytes());
    let provider_details_option: Option<ProviderInfo> = provider_info.get(deps.storage, &info.sender);
    // Declare the variable outside of the match block so it's available afterwards
    let mut provider_details: ProviderInfo;
    let mut accumulated_reward = Uint256::zero();
    // Match against the Option to handle Some and None cases
    match provider_details_option {
        Some(details) => {
            provider_details = details.clone();
            // init variables
            
            let mut timer = env.block.time;
            const SECONDS_TWO_YEARS: u64 = 60 * 60 * 24 * 365 * 2;
            let total_time: u64 = env.block.time.seconds() - details.last_claim.seconds();

            if total_time < SECONDS_TWO_YEARS {
                // Fetch all reward rate changes since the last claim
                let reward_rate_changes: Vec<RewardRateChange> = get_reward_rate_changes_since(pool.reward_rate.clone(), details.last_claim)?;

                for mut rate_change in reward_rate_changes {
                    if rate_change.since.seconds() < provider_details.last_claim.seconds() {
                        rate_change.since = provider_details.last_claim;
                    }
                    // For each rate change, calculate the reward up to that time
                    let time_with_rate = timer.seconds() - rate_change.since.seconds();
                    accumulated_reward += Uint256::from(details.provide_amount * Uint256::from(time_with_rate) * rate_change.rate / pool.shares);
                    // set last claim time to the timestamp of the new rate
                    timer = rate_change.since;
                }
            }
        },
        None => {
            provider_details = ProviderInfo {
                provide_amount: Uint256::zero(),
                last_claim: env.block.time,
                withdraw_requests: None,
            };
        }
    }

    // Calculate total pool tokens (product of the balances)
    let total_pool_tokens = pool.anml_balance.checked_mul(pool.other_balance)
    .map_err(|_| StdError::generic_err("multiplication overflow"))?;

    // Calculate user addition
    let user_addition = anml_deposit.checked_mul(other_deposit)
    .map_err(|_| StdError::generic_err("multiplication overflow"))?;

    // If user addition is greater than pool throw error
    if user_addition > total_pool_tokens {
        return Err(StdError::generic_err("pool liquidity is too low compared to deposit"));
    }

    // Define a scaling factor. This should be a large number to help with small values division.
    let scale_factor = Uint256::from(1_000u32);

    // Scale up the total pool tokens
    let total_pool_tokens_scaled = total_pool_tokens.checked_mul(scale_factor)
    .map_err(|_| StdError::generic_err("scaling overflow"))?;

    
    // find the inverser proportion
    let inverse_user_proportion_scaled = ceil_div(total_pool_tokens_scaled, user_addition)?;
    

    // Now, calculate the added shares by dividing the shares by the inverse of the user's proportion
    // Since the inverse_user_proportion is scaled, it adjusts for the precision loss during division.
    let shares_scaled = pool.shares.checked_div(inverse_user_proportion_scaled)
    .map_err(|_| StdError::generic_err("division by zero or overflow"))?;

    // if shares scaled is zero, liquidity is too low to add 
    if shares_scaled == Uint256::zero() {
        return Err(StdError::generic_err("below minimum liquidity added"));
    }

    // Now, scale up the shares to the original scale
    let shares = shares_scaled.checked_mul(scale_factor)
    .map_err(|_| StdError::generic_err("scaling up overflow"))?;

    // Update pool's token balances
    pool.anml_balance += anml_deposit;
    pool.other_balance += other_deposit;
    pool.shares += shares;
    // Save the updated pool state
    POOL.insert(deps.storage, &pool_id, &pool)?;

    // Update staker info
    provider_details.last_claim = env.block.time;
    provider_details.provide_amount += shares;
    provider_info.insert(deps.storage, &info.sender, &provider_details)?;

    // Handle reward distribution here, e.g., add to provider's balance
    let params = PARAMS.load(deps.storage)?;

    // Initialize an empty set of Cosmos messages.
    let mut messages: Vec<CosmosMsg> = vec![];

    if accumulated_reward > Uint256::zero() {
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
        messages.push(message);
    }

    // Transfer ANML tokens from the user to the contract
    let msg_anml = to_binary(&Snip20Msg::transfer_from_msg(
        info.sender.clone(),
        env.contract.address.clone(),
        anml_deposit,
    ))?;

    let message_anml = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: params.anml_contract.to_string(),
        code_hash: params.anml_hash,
        msg: msg_anml,
        funds: vec![],
    });

    messages.push(message_anml);

    // Transfer other tokens from the user to the contract
    let msg_other = to_binary(&Snip20Msg::transfer_from_msg(
        info.sender.clone(),
        env.contract.address,
        other_deposit,
    ))?;

    let message_other = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pool_id.to_string(),
        code_hash: pool.other_hash,
        msg: msg_other,
        funds: vec![],
    });

    messages.push(message_other);

    let response = Response::new()
        .add_messages(messages);

    Ok(response)
}

pub fn get_reward_rate_changes_since(
    reward_rate_changes: Vec<RewardRateChange>,
    last_claim_time: Timestamp,
) -> StdResult<Vec<RewardRateChange>> {
    let mut relevant_changes = Vec::new();

    // The changes are from oldest to newest, so we reverse the iterator to start from the newest
    for change in reward_rate_changes.iter().rev() { // reverse iteration, from newest to oldest
        // We clone and push changes while their timestamp is greater than the last claim time
        relevant_changes.push(change.clone());
        
        // Break the loop when we find the first change that should not be included,
        // but after we've already added it to our vector
        if change.since.seconds() <= last_claim_time.seconds() {
            break;
        }
    }

    Ok(relevant_changes)
}

pub fn try_claim_provide(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_id: Addr,
) -> StdResult<Response> {
    // Load provider information
    let provider_info = PROVIDER_INFO.add_suffix(pool_id.to_string().as_bytes());
    let provider_details_option: Option<ProviderInfo> = provider_info.get(deps.storage, &info.sender);
    let mut provider_details = provider_details_option.ok_or(StdError::generic_err("no staking info found"))?;
    // Load pool information.
    let pool_option: Option<Pool> = POOL.get(deps.storage, &pool_id);
    let pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;


    // init variables
    let mut accumulated_reward = Uint256::zero();
    let mut timer = env.block.time;
    const SECONDS_TWO_YEARS: u64 = 60 * 60 * 24 * 365 * 2;
    let total_time: u64 = env.block.time.seconds() - provider_details.last_claim.seconds();

    if total_time < SECONDS_TWO_YEARS {
        // Fetch all reward rate changes since the last claim
        let reward_rate_changes: Vec<RewardRateChange> = get_reward_rate_changes_since(pool.reward_rate, provider_details.last_claim)?;
    
        for mut rate_change in reward_rate_changes {
            if rate_change.since.seconds() < provider_details.last_claim.seconds() {
                rate_change.since = provider_details.last_claim;
            }
            // For each rate change, calculate the reward up to that time
            let time_with_rate = timer.seconds() - rate_change.since.seconds();
            accumulated_reward += Uint256::from(provider_details.provide_amount * Uint256::from(time_with_rate) * rate_change.rate / pool.shares);
            // set last claim time to the timestamp of the new rate
            timer = rate_change.since;
        }
    }

    // Update staker info
    provider_details.last_claim = env.block.time;
    provider_info.insert(deps.storage, &info.sender, &provider_details)?;

    // Handle reward distribution here, e.g., add to provider's balance
    let params = PARAMS.load(deps.storage)?;
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
    let response = Response::new().add_message(message);
    Ok(response)
}


pub fn try_request_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_id: Addr,
    amount: Uint256,
) -> StdResult<Response> {
    // Load staker info
    let provider_info = PROVIDER_INFO.add_suffix(pool_id.to_string().as_bytes());
    let provider_details_option: Option<ProviderInfo> = provider_info.get(deps.storage, &info.sender);
    let mut provider_details = provider_details_option.ok_or(StdError::generic_err("no provider info found"))?;
    // Error if they don't have enough staked to unstake this amount
    if provider_details.provide_amount < amount {
        return Err(StdError::generic_err("withdraw request is more than amount provided"));
    }
    // Create a new unstake request
    let unstake_request = UnstakeRequest {
        amount: amount,
        request_time: env.block.time,
    };
    // Check if the staker has any existing unstake requests
    match &mut provider_details.withdraw_requests {
        Some(requests) => {
            // If they do, push the new request to the existing list
            requests.push(unstake_request.clone());
        }
        None => {
            // If they don't, initialize a new list with the request
            provider_details.withdraw_requests = Some(vec![unstake_request.clone()]);
        }
    } 
    // init variables
    let mut accumulated_reward = Uint256::zero();
    let mut timer = env.block.time;
    const SECONDS_TWO_YEARS: u64 = 60 * 60 * 24 * 365 * 2;
    let total_time: u64 = env.block.time.seconds() - provider_details.last_claim.seconds();
    // Load pool information.
    let pool_option: Option<Pool> = POOL.get(deps.storage, &pool_id);
    let pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;

    if total_time < SECONDS_TWO_YEARS {
        // Fetch all reward rate changes since the last claim
        let reward_rate_changes: Vec<RewardRateChange> = get_reward_rate_changes_since(pool.reward_rate, provider_details.last_claim)?;
    
        for mut rate_change in reward_rate_changes {
            if rate_change.since.seconds() < provider_details.last_claim.seconds() {
                rate_change.since = provider_details.last_claim;
            }
            // For each rate change, calculate the reward up to that time
            let time_with_rate = timer.seconds() - rate_change.since.seconds();
            accumulated_reward += Uint256::from(provider_details.provide_amount * Uint256::from(time_with_rate) * rate_change.rate / pool.shares);
            // set last claim time to the timestamp of the new rate
            timer = rate_change.since;
        }
    }
    // Store the updated info
    provider_details.last_claim = env.block.time;
    provider_details.provide_amount -= amount;
    provider_info.insert(deps.storage, &info.sender, &provider_details)?;
   
    // Handle reward distribution here, e.g., add to provider's balance
    let params = PARAMS.load(deps.storage)?;
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
    let response = Response::new().add_message(message);
    Ok(response)
}

pub fn try_complete_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_id: Addr,
) -> StdResult<Response> {
    // Load staker info and params
    let params = PARAMS.load(deps.storage)?;
    let provider_info = PROVIDER_INFO.add_suffix(pool_id.to_string().as_bytes());
    let provider_details_option: Option<ProviderInfo> = provider_info.get(deps.storage, &info.sender);
    let mut provider_details = provider_details_option.ok_or(StdError::generic_err("no provider info found"))?;
    // Load pool information.
    let pool_option: Option<Pool> = POOL.get(deps.storage, &pool_id);
    let mut pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;
    // Track if any request is processed and prepare a vector to hold messages
    let mut request_processed = false;
    let mut cosmos_msgs = Vec::new();
    // Process requests that have completed the cooldown into a vec of matured requests, keep the remainder
    const SECONDS_IN_SEVEN_DAYS: u64 = 60;
    if let Some(ref mut unstake_requests) = provider_details.withdraw_requests.as_mut() {
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

            // Define a scaling factor. This should be a large number to help with small values division.
            let scale_factor = Uint256::from(1_000u32);

            // Scale up the total pool tokens
            let total_shares_scaled = pool.shares.checked_mul(scale_factor)
            .map_err(|_| StdError::generic_err("scaling overflow"))?;

            // find the inverser proportion
            let inverse_user_proportion_scaled = ceil_div(total_shares_scaled, request.amount)?;

            // Now, calculate the withdrawal amounts by dividing the pool balances by the inverse of the user's proportion
            // Since the inverse_user_proportion is scaled, it adjusts for the precision loss during division.
            let withdraw_anml_scaled = pool.anml_balance.checked_div(inverse_user_proportion_scaled)
            .map_err(|_| StdError::generic_err("division by zero or overflow"))?;
            let withdraw_other_scaled = pool.other_balance.checked_div(inverse_user_proportion_scaled)
            .map_err(|_| StdError::generic_err("division by zero or overflow"))?;

            // If the scaled inverse proportion is zero, the withdraw is too small too be processed accurately, add to provide amount
            if withdraw_anml_scaled == Uint256::zero() || withdraw_other_scaled == Uint256::zero() {
                provider_details.provide_amount += request.amount;
            } else {
                // Now, scale down the withdrawal amounts to the original scale
                let withdraw_anml = withdraw_anml_scaled.checked_mul(scale_factor)
                .map_err(|_| StdError::generic_err("scaling down overflow"))?;
                let withdraw_other = withdraw_other_scaled.checked_mul(scale_factor)
                .map_err(|_| StdError::generic_err("scaling down overflow"))?;

                // Use checked_sub to perform subtraction and handle any potential underflow
                pool.anml_balance = pool.anml_balance.checked_sub(withdraw_anml)
                .map_err(|_| StdError::generic_err("underflow on anml_balance subtraction"))?;

                pool.other_balance = pool.other_balance.checked_sub(withdraw_other)
                .map_err(|_| StdError::generic_err("underflow on other_balance subtraction"))?;

                pool.shares = pool.shares.checked_sub(request.amount)
                .map_err(|_| StdError::generic_err("underflow on shares subtraction"))?;

                // Create CosmosMsg for each matured request and push to the vector
                let anml_msg = to_binary(&Snip20Msg::transfer_snip_msg(
                    info.sender.clone(),
                    withdraw_anml,
                ))?;
                let anml_message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: params.anml_contract.to_string(),
                    code_hash: params.anml_hash.clone(),
                    msg: anml_msg,
                    funds: vec![],
                });
                cosmos_msgs.push(anml_message);
                let other_msg = to_binary(&Snip20Msg::transfer_snip_msg(
                    info.sender.clone(),
                    withdraw_other,
                ))?;
                let other_message = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: pool.other_contract.to_string(),
                    code_hash: pool.other_hash.clone(),
                    msg: other_msg,
                    funds: vec![],
                });
                cosmos_msgs.push(other_message);
            }
        }
    } else {
        return Err(StdError::generic_err("no unstake request found"));
    }
    // If no request was processed, return an error
    if !request_processed {
        return Err(StdError::generic_err("cooldown not met for any request"));
    }
    POOL.insert(deps.storage, &pool_id, &pool)?;
    provider_info.insert(deps.storage, &info.sender, &provider_details)?;
    // Build the response
    let response = Response::new().add_messages(cosmos_msgs);
    Ok(response)
}

pub fn query_pool_info(
    deps: Deps,
    env: Env,
    pool_id: Addr,
    address: Addr,
) -> StdResult<PoolInfoResponse> {
    // Load pool information.
    let pool_option: Option<Pool> = POOL.get(deps.storage, &pool_id);
    let pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;

    // Load provider information
    let provider_info = PROVIDER_INFO.add_suffix(pool_id.to_string().as_bytes());
    let provider_details_option: Option<ProviderInfo> = provider_info.get(deps.storage, &address);
    let mut accumulated_reward = Uint256::zero();
    if let Some(ref provider_details) = provider_details_option {
        // init variables
        let mut timer = env.block.time;
        // Fetch all reward rate changes since the last claim
        let reward_rate_changes: Vec<RewardRateChange> = get_reward_rate_changes_since(pool.reward_rate.clone(), provider_details.last_claim)?;
        
        for mut rate_change in reward_rate_changes {
            if rate_change.since.seconds() < provider_details.last_claim.seconds() {
                rate_change.since = provider_details.last_claim;
            }
            // For each rate change, calculate the reward up to that time
            let time_with_rate = timer.seconds() - rate_change.since.seconds();
            accumulated_reward += Uint256::from(provider_details.provide_amount * Uint256::from(time_with_rate) * rate_change.rate / pool.shares);
            // set last claim time to the timestamp of the new rate
            timer = rate_change.since;
        }
    }
    
    //construct and send response
    let response = PoolInfoResponse {
        pool: pool,
        provider_info: provider_details_option,
        accumulated_reward: accumulated_reward,
    };
    Ok(response)
}





