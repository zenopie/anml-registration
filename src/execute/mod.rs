// src/execute/mod.rs
pub mod update_config;
pub mod registration;
pub mod allocation;
pub mod claim_anml;
pub mod receive;

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};
use crate::msg::ExecuteMsg;

pub fn execute_dispatch(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo, 
    msg: ExecuteMsg
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { config } => update_config::update_config(deps, env, info, config),
        ExecuteMsg::Register { address, id_hash, affiliate } => registration::register(deps, env, info, address, id_hash, affiliate),
        ExecuteMsg::ClaimAnml {} => claim_anml::claim_anml(deps, env, info),
        ExecuteMsg::SetAllocation { percentages } => allocation::set_allocation(deps, env, info, percentages),
        ExecuteMsg::ClaimAllocation { allocation_id } => allocation::claim_allocation(deps, env, info, allocation_id),
        ExecuteMsg::EditAllocation { allocation_id, config } => allocation::edit_allocation(deps, info, allocation_id, config),
        ExecuteMsg::AddAllocation { receive_addr, receive_hash, manager_addr, claimer_addr, use_send } => 
            allocation::add_allocation(deps, env, info, receive_addr, receive_hash, manager_addr, claimer_addr, use_send),
        ExecuteMsg::Receive { sender, from, amount, msg, memo: _ } => 
            receive::receive(deps, env, info, sender, from, amount, msg),
    }
}
