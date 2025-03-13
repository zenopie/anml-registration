// src/state/allocation.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Timestamp, Uint128};
use secret_toolkit_storage::{Keymap, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllocationState {
    pub allocation_id: u32,
    pub amount_allocated: Uint128,
    pub last_claim: Timestamp,
    pub accumulated_rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllocationConfig {
    pub receive_addr: Addr,
    pub receive_hash: Option<String>,
    pub manager_addr: Option<Addr>,
    pub claimer_addr: Option<Addr>,
    pub use_send: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Allocation {
    pub state: AllocationState,
    pub config: AllocationConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllocationPercentage {
    pub allocation_id: u32,
    pub percentage: Uint128,
}

pub static ALLOCATION_OPTIONS: Item<Vec<Allocation>> = Item::new(b"allocation_options");
pub static USER_ALLOCATIONS: Keymap<Addr, Vec<AllocationPercentage>> = Keymap::new(b"user_allocations");
