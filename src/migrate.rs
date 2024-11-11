// src/migrate.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{DepsMut, Env, Response, StdResult, Timestamp, Uint128, Addr, to_binary,
    CosmosMsg, WasmMsg, };
use secret_toolkit_storage::Item;
use secret_toolkit::snip20;
use crate::msg::MigrateMsg;
use crate::state::{
    STATE, CONFIG, ALLOCATION_OPTIONS, Allocation, AllocationConfig, AllocationState,
    State, Config
};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OldState {
    pub registrations: u32,
    pub registration_address: Addr,
    pub registration_wallet: Addr,
    pub contract_manager: Addr,
    pub max_registrations: u32,
    pub anml_token_contract: Addr,
    pub anml_token_hash: String,
    pub erth_token_contract: Addr,
    pub erth_token_hash: String,
    pub anml_pool_contract: Addr,
    pub anml_pool_hash: String,
    pub last_anml_buyback: Timestamp,
    pub total_allocations: Uint128,
    pub allocation_counter: u32,
    pub registration_reward: Uint128,
}

pub static OLDSTATE: Item<OldState> = Item::new(b"state");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OldAllocation {
    pub allocation_id: u32,
    pub receive_addr: Addr,
    pub receive_hash: Option<String>,
    pub manager_addr: Option<Addr>,
    pub claimer_addr: Option<Addr>,
    pub use_send: bool,
    pub amount_allocated: Uint128,
    pub last_claim: Timestamp,
}

pub static OLD_ALLOCATION_OPTIONS: Item<Vec<OldAllocation>> = Item::new(b"allocation_options");

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
    // Load the old state
    let old_state = OLDSTATE.load(deps.storage)?;

    // Split the old state into new state and config
    let new_state = State {
        registrations: old_state.registrations,
        last_anml_buyback: old_state.last_anml_buyback,
        total_allocations: old_state.total_allocations,
        allocation_counter: old_state.allocation_counter,
        registration_reward: old_state.registration_reward,
    };

    let new_config = Config {
        contract_manager: old_state.contract_manager,
        registration_address: old_state.registration_address,
        registration_wallet: old_state.registration_wallet,
        max_registrations: old_state.max_registrations,
        anml_token_contract: old_state.anml_token_contract,
        anml_token_hash: old_state.anml_token_hash,
        erth_token_contract: old_state.erth_token_contract,
        erth_token_hash: old_state.erth_token_hash,
        anml_pool_contract: old_state.anml_pool_contract,
        anml_pool_hash: old_state.anml_pool_hash,
    };

    // Save the new state and config
    STATE.save(deps.storage, &new_state)?;
    CONFIG.save(deps.storage, &new_config)?;

    // Load old allocations
    let old_allocations = OLD_ALLOCATION_OPTIONS.load(deps.storage)?;
    let mut new_allocations = vec![];
    for old_alloc in old_allocations {
        let allocation_state = AllocationState {
            allocation_id: old_alloc.allocation_id,
            amount_allocated: old_alloc.amount_allocated,
            last_claim: old_alloc.last_claim,
        };

        let allocation_config = AllocationConfig {
            receive_addr: old_alloc.receive_addr,
            receive_hash: old_alloc.receive_hash,
            manager_addr: old_alloc.manager_addr,
            claimer_addr: old_alloc.claimer_addr,
            use_send: old_alloc.use_send,
        };

        let allocation = Allocation {
            state: allocation_state,
            config: allocation_config,
        };

        new_allocations.push(allocation);
    }

    ALLOCATION_OPTIONS.save(deps.storage, &new_allocations)?;

    // Register this contract as a receiver for ERTH
    let register_erth_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: new_config.erth_token_contract.to_string(),
        code_hash: new_config.erth_token_hash,
        msg: to_binary(&snip20::HandleMsg::RegisterReceive {
            code_hash: env.contract.code_hash.clone(),
            padding: None,  // Optional padding
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(register_erth_msg)
        .add_attribute("action", "migrate")
        .add_attribute("status", "success"))
}
