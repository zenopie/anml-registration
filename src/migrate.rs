// src/migrate.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{DepsMut, Env, Response, StdResult, to_binary,
    CosmosMsg, WasmMsg, Addr, Uint128};
use secret_toolkit_storage::Item;
use secret_toolkit::snip20;
use crate::msg::MigrateMsg;
use crate::state::{
    CONFIG, Config, ALLOCATION_OPTIONS
};


// For deserializing the old Config struct
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OldConfig {
    pub registration_address: Addr,
    pub registration_wallet: Addr,
    pub contract_manager: Addr,
    pub max_registrations: u32, // The old field you're removing
    pub anml_token_contract: Addr,
    pub anml_token_hash: String,
    pub erth_token_contract: Addr,
    pub erth_token_hash: String,
    pub anml_pool_contract: Addr,
    pub anml_pool_hash: String,
}

pub static OLD_CONFIG: Item<OldConfig> = Item::new(b"config");

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
    let old_config = OLD_CONFIG.load(deps.storage)?;
    
    // Create new config with default validity period
    let config = Config {
        registration_address: old_config.registration_address,
        registration_wallet: old_config.registration_wallet,
        contract_manager: old_config.contract_manager,
        registration_validity_seconds: 60 * 60 * 24 * 7,
        anml_token_contract: old_config.anml_token_contract,
        anml_token_hash: old_config.anml_token_hash,
        erth_token_contract: old_config.erth_token_contract,
        erth_token_hash: old_config.erth_token_hash,
        anml_pool_contract: old_config.anml_pool_contract,
        anml_pool_hash: old_config.anml_pool_hash,
    };

    // Save the new config
    CONFIG.save(deps.storage, &config)?;

    // Load and reset allocation options
    let mut allocation_options = ALLOCATION_OPTIONS.load(deps.storage)?;
    for allocation in &mut allocation_options {
        allocation.state.amount_allocated = Uint128::zero();
        allocation.state.accumulated_rewards = Uint128::zero();
    }
    ALLOCATION_OPTIONS.save(deps.storage, &allocation_options)?;


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
