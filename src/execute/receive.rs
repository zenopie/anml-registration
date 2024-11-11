// src/execute/receive.rs
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, Addr,
    Binary, from_binary};
use crate::state::{CONFIG, STATE};
use crate::msg::ReceiveMsg;

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    _from: Addr,
    amount: Uint128,
    msg: Binary,
) -> StdResult<Response> {

    let msg: ReceiveMsg = from_binary(&msg)?;

    match msg {
        ReceiveMsg::AllocationSend { allocation_id } => 
            receive_allocation(deps, env, info, amount, allocation_id),
    }   
}

fn receive_allocation(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    _allocation_id: u32,
) -> StdResult<Response> {

    // Load the state
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if info.sender != config.erth_token_contract {
        return Err(StdError::generic_err("Invalid token sender"));
    }

    state.registration_reward = state.registration_reward.checked_add(amount)
        .map_err(|_| StdError::generic_err("Overflow in registration reward"))?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
            .add_attribute("action", "receive_allocation")
            .add_attribute("amount", amount.to_string()))
}
