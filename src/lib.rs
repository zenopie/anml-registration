// src/lib.rs
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use crate::execute::execute_dispatch;
use crate::query::query_dispatch;
use crate::migrate::perform_migration;
use crate::instantiate::execute_instantiate;

pub mod msg;
pub mod state;
pub mod execute;
pub mod query;
pub mod migrate;
pub mod instantiate;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: msg::InstantiateMsg,
) -> StdResult<Response> {
    execute_instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo, 
    msg: msg::ExecuteMsg
) -> StdResult<Response> {
    execute_dispatch(deps, env, info, msg)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut, 
    env: Env, 
    msg: msg::MigrateMsg
) -> StdResult<Response> {
    perform_migration(deps, env, msg)
}

#[entry_point]
pub fn query(
    deps: Deps, 
    env: Env, 
    msg: msg::QueryMsg
) -> StdResult<Binary> {
    query_dispatch(deps, env, msg)
}
