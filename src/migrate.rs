// src/migrate.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{DepsMut, Env, Response, StdResult, Timestamp, Uint128, to_binary,
    CosmosMsg, WasmMsg, };
use secret_toolkit_storage::Item;
use secret_toolkit::snip20;
use crate::msg::MigrateMsg;
use crate::state::{
    STATE, CONFIG, ALLOCATION_OPTIONS, Allocation, AllocationConfig, AllocationState,
    State
};

// Define the previous state struct without last_upkeep
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OldState {
    pub registrations: u32,
    pub last_anml_buyback: Timestamp,
    pub total_allocations: Uint128,
    pub allocation_counter: u32,
    pub registration_reward: Uint128,
}

// Define the previous allocation state without accumulated_rewards
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OldAllocationState {
    pub allocation_id: u32,
    pub amount_allocated: Uint128,
    pub last_claim: Timestamp,
}

// Define the previous allocation struct
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OldAllocation {
    pub state: OldAllocationState,
    pub config: AllocationConfig,
}

// Use the same storage keys as in the current contract
pub static OLD_STATE: Item<OldState> = Item::new(b"state");
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
    // Load the old state format - propagate any errors
    let old_state = OLD_STATE.load(deps.storage)?;

    // Create new state with last_upkeep field initialized to current time
    let new_state = State {
        registrations: old_state.registrations,
        last_anml_buyback: old_state.last_anml_buyback,
        total_allocations: old_state.total_allocations,
        allocation_counter: old_state.allocation_counter,
        registration_reward: old_state.registration_reward,
        last_upkeep: env.block.time,  // Initialize last_upkeep with current block time
    };

    // Save the new state format
    STATE.save(deps.storage, &new_state)?;

    // Load the existing config (no changes needed)
    let config = CONFIG.load(deps.storage)?;

    // Load old allocations - propagate any errors
    let old_allocations = OLD_ALLOCATION_OPTIONS.load(deps.storage)?;

    // Convert old allocations to new format with accumulated_rewards
    let mut new_allocations = vec![];
    for old_alloc in old_allocations {
        let allocation_state = AllocationState {
            allocation_id: old_alloc.state.allocation_id,
            amount_allocated: old_alloc.state.amount_allocated,
            last_claim: old_alloc.state.last_claim,
            accumulated_rewards: Uint128::zero(),  // Initialize accumulated_rewards to zero
        };

        let allocation = Allocation {
            state: allocation_state,
            config: old_alloc.config,
        };

        new_allocations.push(allocation);
    }

    // Save the new allocations format
    ALLOCATION_OPTIONS.save(deps.storage, &new_allocations)?;

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
        .add_attribute("status", "success")
        .add_attribute("allocations_migrated", new_allocations.len().to_string()))
}
