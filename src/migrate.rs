// src/migrate.rs
use cosmwasm_std::{DepsMut, Env, Response, StdResult, to_binary,
    CosmosMsg, WasmMsg};
use secret_toolkit::snip20;
use crate::msg::MigrateMsg;
use crate::state::{
    CONFIG,
};



pub fn perform_migration(
    deps: DepsMut, 
    env: Env, 
    msg: MigrateMsg,
) -> StdResult<Response> {
    match msg {
        MigrateMsg::Migrate {} => migrate_state(deps, env),
    }
}

fn migrate_state(deps: DepsMut, env: Env) -> StdResult<Response> {


    // Load the old config
    let config = CONFIG.load(deps.storage)?;


    // Register this contract as a receiver for ERTH (important to maintain)
    let register_erth_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.erth_token_contract.to_string(),
        code_hash: config.erth_token_hash,
        msg: to_binary(&snip20::HandleMsg::RegisterReceive {
            code_hash: env.contract.code_hash.clone(),
            padding: None,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(register_erth_msg)
        .add_attribute("action", "migrate")
        .add_attribute("status", "success"))
}
