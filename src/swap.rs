use cosmwasm_std::{
    to_binary, Deps, DepsMut, MessageInfo, Response, StdError, Addr, CosmosMsg, WasmMsg, StdResult,
    Uint256,
};

use crate::msg::{Snip20Msg, SwapSimulationResponse};
use crate::state::{STATE, PARAMS, POOL, Pool,};



pub fn try_swap(
    deps: DepsMut,
    info: MessageInfo,
    from: Addr,
    amount: Uint256,
    token: Addr,
) -> StdResult<Response> {

    // Ensure the input and output tokens are different.
    if info.sender == token {
        return Err(StdError::generic_err("input and output tokens must be different"));
    }

    let params = PARAMS.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // If the sender is the ANML contract.
    if info.sender == params.anml_contract {
        let fee = amount / Uint256::from(10000u32) * params.scaled_swap_fee;
        state.fee_balance += fee;
        STATE.save(deps.storage, &state)?;
        let amount_after_fee = amount - fee;

        // Load pool information.
        let pool_option: Option<Pool> = POOL.get(deps.storage, &token);
        let mut pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;

        // Ensure liquidity is sufficient.
        if pool.anml_balance < amount {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        // Calculate the constant for maintaining liquidity.
        let constant = pool.anml_balance * pool.other_balance;

        // Calculate the new balances after the swap.
        let new_anml_balance = pool.anml_balance + amount_after_fee;
        let new_other_balance = constant / new_anml_balance;
        let token_received = pool.other_balance - new_other_balance;

        // Update pool balances.
        pool.anml_balance += amount_after_fee;
        pool.other_balance -= token_received;
        pool.volume += amount;
        POOL.insert(deps.storage, &token, &pool)?;

        // Construct the transfer message.
        let msg = to_binary(&Snip20Msg::transfer_snip_msg(from, token_received))?;
        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pool.other_contract.to_string(),
            code_hash: pool.other_hash,
            msg,
            funds: vec![],
        });

        Ok(Response::new().add_message(message))

    // If the token is the ANML contract.
    } else if token == params.anml_contract {

        // Load pool information.
        let pool_option: Option<Pool> = POOL.get(deps.storage, &info.sender);
        let mut pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;

        // Ensure liquidity is sufficient.
        if pool.other_balance < amount {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        // Calculate the constant for maintaining liquidity.
        let constant = pool.anml_balance * pool.other_balance;

        // Calculate the new balances after the swap.
        let new_other_balance = pool.other_balance + amount;
        let new_anml_balance = constant / new_other_balance;
        let token_received = pool.anml_balance - new_anml_balance;

        let fee = token_received / Uint256::from(10000u32) * params.scaled_swap_fee;
        state.fee_balance += fee;
        STATE.save(deps.storage, &state)?;
        let token_received_after_fee = token_received - fee;

        // Update pool balances.
        pool.other_balance += amount;
        pool.anml_balance -= token_received;
        pool.volume += token_received;
        POOL.insert(deps.storage, &info.sender, &pool)?;

        // Construct the transfer message.
        let msg = to_binary(&Snip20Msg::transfer_snip_msg(from, token_received_after_fee))?;
        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: params.anml_contract.to_string(),
            code_hash: params.anml_hash,
            msg,
            funds: vec![],
        });

        Ok(Response::new().add_message(message))

    // For double swaps
    } else {

        // Load the first pool information.
        let pool_one_option: Option<Pool> = POOL.get(deps.storage, &info.sender);
        let mut pool_one = pool_one_option.ok_or(StdError::generic_err("no pool found"))?;

        // Ensure liquidity is sufficient.
        if pool_one.other_balance < amount {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        // Calculate the constant for maintaining liquidity.
        let constant_one = pool_one.anml_balance * pool_one.other_balance;

        // Calculate the new balances after the swap for the first pool.
        let new_other_balance_one = pool_one.other_balance + amount;
        let new_anml_balance_one = constant_one / new_other_balance_one;
        let anml_received = pool_one.anml_balance - new_anml_balance_one;

        // Update the first pool's balances.
        pool_one.other_balance += amount;
        pool_one.anml_balance -= anml_received;
        pool_one.volume += anml_received;
        POOL.insert(deps.storage, &info.sender, &pool_one)?;

        let fee = anml_received / Uint256::from(10000u32) * params.scaled_swap_fee;
        state.fee_balance += fee;
        STATE.save(deps.storage, &state)?;
        let anml_received_after_fee = anml_received - fee;

        // Load the second pool information.
        let pool_two_option: Option<Pool> = POOL.get(deps.storage, &token);
        let mut pool_two = pool_two_option.ok_or(StdError::generic_err("no pool found"))?;

        // Ensure liquidity is sufficient for the second pool.
        if pool_two.anml_balance < anml_received {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        // Calculate the constant for maintaining liquidity for the second pool.
        let constant_two = pool_two.anml_balance * pool_two.other_balance;

        // Calculate the new balances after the swap for the second pool.
        let new_anml_balance_two = pool_two.anml_balance + anml_received_after_fee;
        let new_other_balance_two = constant_two / new_anml_balance_two;
        let token_received = pool_two.other_balance - new_other_balance_two;

        // Update the second pool's balances.
        pool_two.other_balance -= token_received;
        pool_two.anml_balance += anml_received;
        pool_two.volume += anml_received;
        POOL.insert(deps.storage, &token, &pool_two)?;

        // Construct the transfer message for the second pool.
        let msg = to_binary(&Snip20Msg::transfer_snip_msg(from, token_received))?;
        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pool_two.other_contract.to_string(),
            code_hash: pool_two.other_hash,
            msg,
            funds: vec![],
        });

        Ok(Response::new().add_message(message))
    }
}


pub fn query_swap_simulation(
    deps: Deps,
    input: Addr,
    output: Addr,
    amount: Uint256,
) -> StdResult<SwapSimulationResponse> {

    // Ensure the input and output tokens are different.
    if input == output {
        return Err(StdError::generic_err("input and output tokens must be different"));
    }
    const SCALING_FACTOR : u32 = 10000;
    let params = PARAMS.load(deps.storage)?;

    // If the sender is the ANML contract.
    if input == params.anml_contract {
        let fee = amount / Uint256::from(10000u32) * params.scaled_swap_fee;
        let amount_after_fee = amount - fee;
        // Load pool information.
        let pool_option: Option<Pool> = POOL.get(deps.storage, &output);
        let pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;

        // Ensure liquidity is sufficient.
        if pool.anml_balance < amount {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        // Calculate the constant for maintaining liquidity.
        let constant = pool.anml_balance * pool.other_balance;

        // Calculate the new balances after the swap.
        let new_anml_balance = pool.anml_balance + amount_after_fee;
        let new_other_balance = constant / new_anml_balance;
        let token_received = pool.other_balance - new_other_balance;


        let response = SwapSimulationResponse {
            amount: token_received,
            scaled_price_impact: Uint256::from(SCALING_FACTOR) - (new_other_balance * Uint256::from(10000u32)/ pool.other_balance),
            fee: fee,
        };
        Ok(response)

    // If the token is the ANML contract.
    } else if output == params.anml_contract {

        // Load pool information.
        let pool_option: Option<Pool> = POOL.get(deps.storage, &input);
        let pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;

        // Ensure liquidity is sufficient.
        if pool.other_balance < amount {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        // Calculate the constant for maintaining liquidity.
        let constant = pool.anml_balance * pool.other_balance;

        // Calculate the new balances after the swap.
        let new_other_balance = pool.other_balance + amount;
        let new_anml_balance = constant / new_other_balance;
        let token_received = pool.anml_balance - new_anml_balance;

        let fee = token_received / Uint256::from(10000u32) * params.scaled_swap_fee;
        let token_received_after_fee = token_received - fee;

        let response = SwapSimulationResponse {
            amount: token_received_after_fee,
            scaled_price_impact: Uint256::from(SCALING_FACTOR) - (new_anml_balance * Uint256::from(10000u32)/ pool.anml_balance),
            fee: fee,
        };
        Ok(response)

    // For double swaps
    } else {

        // Load the first pool information.
        let pool_one_option: Option<Pool> = POOL.get(deps.storage, &input);
        let pool_one = pool_one_option.ok_or(StdError::generic_err("no pool found"))?;

        // Ensure liquidity is sufficient.
        if pool_one.other_balance < amount {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        // Calculate the constant for maintaining liquidity.
        let constant_one = pool_one.anml_balance * pool_one.other_balance;

        // Calculate the new balances after the swap for the first pool.
        let new_other_balance_one = pool_one.other_balance + amount;
        let new_anml_balance_one = constant_one / new_other_balance_one;
        let anml_received = pool_one.anml_balance - new_anml_balance_one;

        // Load the second pool information.
        let pool_two_option: Option<Pool> = POOL.get(deps.storage, &output);
        let pool_two = pool_two_option.ok_or(StdError::generic_err("no pool found"))?;

        let fee = anml_received / Uint256::from(10000u32) * params.scaled_swap_fee;
        let anml_received_after_fee = anml_received - fee;

        // Ensure liquidity is sufficient for the second pool.
        if pool_two.anml_balance < anml_received_after_fee {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        // Calculate the constant for maintaining liquidity for the second pool.
        let constant_two = pool_two.anml_balance * pool_two.other_balance;

        // Calculate the new balances after the swap for the second pool.
        let new_anml_balance_two = pool_two.anml_balance + anml_received_after_fee;
        let new_other_balance_two = constant_two / new_anml_balance_two;
        let token_received = pool_two.other_balance - new_other_balance_two;

        let scaled_price_impact_one = Uint256::from(SCALING_FACTOR) - (new_anml_balance_one * Uint256::from(10000u32)/ pool_one.anml_balance);
        let scaled_price_impact_two = Uint256::from(SCALING_FACTOR) - (new_other_balance_two * Uint256::from(10000u32)/ pool_two.other_balance);
        // Pick the greater value without an import
        let greater_value = if scaled_price_impact_one > scaled_price_impact_two {
            scaled_price_impact_one
        } else {
            scaled_price_impact_two
        };

        let response = SwapSimulationResponse {
            amount: token_received,
            scaled_price_impact: greater_value,
            fee: fee,
        };
        Ok(response)
    }
}

pub fn query_reverse_swap(
    deps: Deps,
    input: Addr,
    output: Addr,
    desired_amount: Uint256,
) -> StdResult<SwapSimulationResponse> {

    if input == output {
        return Err(StdError::generic_err("input and output tokens must be different"));
    }
    const SCALING_FACTOR : u32 = 10000; 
    let params = PARAMS.load(deps.storage)?;

    if output == params.anml_contract {
        // Load pool information.
        let pool_option: Option<Pool> = POOL.get(deps.storage, &input);
        let pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;

        let fee = desired_amount / Uint256::from(10000u32) * params.scaled_swap_fee;
        let desired_amount_after_fee = desired_amount + fee;

        // Calculate the constant for maintaining liquidity.
        let constant = pool.anml_balance * pool.other_balance;

        // Calculate the new balances after the swap.
        let new_anml_balance = pool.anml_balance - desired_amount_after_fee;
        let new_other_balance = constant / new_anml_balance;
        let required_input = new_other_balance - pool.other_balance;
        // Ensure liquidity is sufficient.
        if required_input > pool.other_balance {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        let response = SwapSimulationResponse {
            amount: required_input,
            scaled_price_impact: Uint256::from(SCALING_FACTOR) - (new_anml_balance * Uint256::from(10000u32)/ pool.anml_balance),
            fee: fee,
        };
        Ok(response)
    } else if input == params.anml_contract {
        // Load pool information.
        let pool_option: Option<Pool> = POOL.get(deps.storage, &output);
        let pool = pool_option.ok_or(StdError::generic_err("no pool found"))?;
        
        // Calculate the constant for maintaining liquidity.
        let constant = pool.anml_balance * pool.other_balance;
        // Calculate the new balances after the swap.
        let new_other_balance = pool.other_balance - desired_amount;
        let new_anml_balance = constant / new_other_balance;
        let required_input = new_anml_balance - pool.anml_balance;

        let fee = required_input / Uint256::from(10000u32) * params.scaled_swap_fee;
        let required_input_after_fee = required_input + fee;
        // Ensure liquidity is sufficient.
        if required_input_after_fee > pool.anml_balance {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }
        let response = SwapSimulationResponse {
            amount: required_input_after_fee,
            scaled_price_impact: Uint256::from(SCALING_FACTOR) - (new_other_balance * Uint256::from(10000u32)/ pool.other_balance),
            fee: fee,
        };
        Ok(response)
    } else {
        // Load the second pool information.
        let pool_two_option: Option<Pool> = POOL.get(deps.storage, &output);
        let pool_two = pool_two_option.ok_or(StdError::generic_err("no pool found"))?;
        // Calculate the constant for maintaining liquidity.
        let constant_two = pool_two.anml_balance * pool_two.other_balance;
        // Calculate the new balances after the swap.
        let new_other_balance_two = pool_two.other_balance - desired_amount;
        let new_anml_balance_two = constant_two / new_other_balance_two;
        let required_input_two = new_anml_balance_two - pool_two.anml_balance;

        let fee = required_input_two / Uint256::from(10000u32) * params.scaled_swap_fee;
        let required_input_two_after_fee = required_input_two + fee;
        // Ensure liquidity is sufficient.
        if required_input_two_after_fee > pool_two.anml_balance {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }
    
        // Load the first pool information.
        let pool_one_option: Option<Pool> = POOL.get(deps.storage, &input);
        let pool_one = pool_one_option.ok_or(StdError::generic_err("no pool found"))?;
        // Calculate the constant for maintaining liquidity.
        let constant_one = pool_one.anml_balance * pool_one.other_balance;
        // Calculate the new balances after the swap.
        let new_anml_balance_one = pool_one.anml_balance - required_input_two_after_fee;
        let new_other_balance_one = constant_one / new_anml_balance_one;
        let required_input_one = new_other_balance_one - pool_one.other_balance;
        // Ensure liquidity is sufficient.
        if required_input_one > pool_one.other_balance {
            return Err(StdError::generic_err("insufficient liquidity for trade"));
        }

        let scaled_price_impact_one = Uint256::from(SCALING_FACTOR) - (new_anml_balance_one * Uint256::from(10000u32)/ pool_one.anml_balance);
        let scaled_price_impact_two = Uint256::from(SCALING_FACTOR) - (new_other_balance_two * Uint256::from(10000u32)/ pool_two.other_balance);
        // Pick the greater value without an import
        let greater_value = if scaled_price_impact_one > scaled_price_impact_two {
            scaled_price_impact_one
        } else {
            scaled_price_impact_two
        };
        let response = SwapSimulationResponse {
            amount: required_input_one,
            scaled_price_impact: greater_value,
            fee: fee,
        };
        Ok(response)
        
    }
}

