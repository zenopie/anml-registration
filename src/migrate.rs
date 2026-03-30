// src/migrate.rs
use cosmwasm_std::{DepsMut, Env, Response, StdResult, to_binary, Uint128, Addr, Timestamp,
    CosmosMsg, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::snip20;
use secret_toolkit_storage::Item;
use crate::msg::MigrateMsg;
use crate::state::{
    Config, CONFIG, State, STATE, ALLOCATION_OPTIONS, ALLOCATION_IDS,
    Allocation, AllocationState, AllocationConfig,
};

// Old types matching what's currently in storage (bincode format)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OldConfig {
    pub registration_address: Addr,
    pub registration_wallet: Addr,
    pub contract_manager: Addr,
    pub registration_validity_seconds: u64,
    pub anml_token_contract: Addr,
    pub anml_token_hash: String,
    pub erth_token_contract: Addr,
    pub erth_token_hash: String,
    pub anml_pool_contract: Addr,
    pub anml_pool_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OldState {
    pub registrations: u32,
    pub last_anml_buyback: Timestamp,
    pub total_allocations: Uint128,
    pub allocation_counter: u32,
    pub registration_reward: Uint128,
    pub last_upkeep: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OldAllocationState {
    pub allocation_id: u32,
    pub amount_allocated: Uint128,
    pub last_claim: Timestamp,
    pub accumulated_rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OldAllocationConfig {
    pub receive_addr: Addr,
    pub receive_hash: Option<String>,
    pub manager_addr: Option<Addr>,
    pub claimer_addr: Option<Addr>,
    pub use_send: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OldAllocation {
    pub state: OldAllocationState,
    pub config: OldAllocationConfig,
}

pub fn perform_migration(
    deps: DepsMut,
    env: Env,
    msg: MigrateMsg,
) -> StdResult<Response> {
    match msg {
        MigrateMsg::Migrate {
            registry_contract,
            registry_hash,
        } => migrate_state(deps, env, registry_contract, registry_hash),
        MigrateMsg::Upgrade {} => {
            Ok(Response::new().add_attribute("action", "upgrade"))
        }
    }
}

fn migrate_state(
    deps: DepsMut,
    env: Env,
    registry_contract: String,
    registry_hash: String,
) -> StdResult<Response> {

    // Load old config and convert to new format
    let old_config_storage: Item<OldConfig> = Item::new(b"config");
    let old_config = old_config_storage.load(deps.storage)?;

    let registry_addr = deps.api.addr_validate(&registry_contract)?;

    let new_config = Config {
        registration_address: old_config.registration_address,
        registration_wallet: old_config.registration_wallet,
        contract_manager: old_config.contract_manager,
        registration_validity_seconds: old_config.registration_validity_seconds,
        registry_contract: registry_addr,
        registry_hash: registry_hash.clone(),
    };
    CONFIG.save(deps.storage, &new_config)?;

    // Load old state explicitly (without reward_index/epoch fields)
    let old_state_storage: Item<OldState> = Item::new(b"state");
    let old_state = old_state_storage.load(deps.storage)?;

    // Load old allocation options (without description/last_reward_index fields)
    let old_alloc_storage: Item<Vec<OldAllocation>> = Item::new(b"allocation_options");
    let old_allocations = old_alloc_storage.load(deps.storage).unwrap_or_default();

    // Calculate final reward distribution from old system
    let time_elapsed = env.block.time.seconds().saturating_sub(old_state.last_upkeep.seconds());
    let total_rewards = Uint128::from(time_elapsed) * Uint128::from(1_000_000u128);
    let old_total_allocations: Uint128 = old_allocations.iter()
        .fold(Uint128::zero(), |acc, a| acc + a.state.amount_allocated);

    // Migrate each allocation to new Keymap storage with new fields
    let mut ids = Vec::new();
    for old_alloc in old_allocations {
        let mut accumulated_rewards = old_alloc.state.accumulated_rewards;

        // Settle final rewards from old system
        if !old_total_allocations.is_zero() && !old_alloc.state.amount_allocated.is_zero() {
            let share = old_alloc.state.amount_allocated * total_rewards / old_total_allocations;
            accumulated_rewards = accumulated_rewards + share;
        }

        let new_alloc = Allocation {
            state: AllocationState {
                allocation_id: old_alloc.state.allocation_id,
                amount_allocated: Uint128::zero(),
                last_claim: old_alloc.state.last_claim,
                accumulated_rewards,
                last_reward_index: Uint128::zero(),
            },
            config: AllocationConfig {
                description: String::new(),
                receive_addr: old_alloc.config.receive_addr,
                receive_hash: old_alloc.config.receive_hash,
                manager_addr: old_alloc.config.manager_addr,
                claimer_addr: old_alloc.config.claimer_addr,
                use_send: old_alloc.config.use_send,
            },
        };

        let id = new_alloc.state.allocation_id;
        ids.push(id);
        ALLOCATION_OPTIONS.insert(deps.storage, &id, &new_alloc)?;
    }

    ALLOCATION_IDS.save(deps.storage, &ids)?;

    // Create new state with reward index fields
    let new_state = State {
        registrations: old_state.registrations,
        last_anml_buyback: old_state.last_anml_buyback,
        total_allocations: Uint128::zero(),
        allocation_counter: old_state.allocation_counter,
        registration_reward: old_state.registration_reward,
        last_upkeep: env.block.time,
        reward_index: Uint128::zero(),
        epoch: 0,
    };
    STATE.save(deps.storage, &new_state)?;

    // Query registry for erth_token to register receiver
    let deps_ref = deps.as_ref();
    let contracts = crate::state::query_registry(
        &deps_ref,
        &new_config.registry_contract,
        &new_config.registry_hash,
        vec!["erth_token"],
    )?;
    let erth_token = &contracts[0];

    // Register this contract as a receiver for ERTH
    let register_erth_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: erth_token.address.to_string(),
        code_hash: erth_token.code_hash.clone(),
        msg: to_binary(&snip20::HandleMsg::RegisterReceive {
            code_hash: env.contract.code_hash.clone(),
            padding: None,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(register_erth_msg)
        .add_attribute("action", "migrate")
        .add_attribute("allocations_migrated", ids.len().to_string())
        .add_attribute("status", "success"))
}
