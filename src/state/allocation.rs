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
    pub last_reward_index: Uint128,
}

pub const MAX_DESCRIPTION_LENGTH: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllocationConfig {
    pub description: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UserAllocations {
    pub epoch: u32,
    pub allocations: Vec<AllocationPercentage>,
}

impl Default for UserAllocations {
    fn default() -> Self {
        UserAllocations {
            epoch: 0,
            allocations: vec![],
        }
    }
}

pub static ALLOCATION_OPTIONS: Keymap<u32, Allocation> = Keymap::new(b"allocation_options_v2");
pub static ALLOCATION_IDS: Item<Vec<u32>> = Item::new(b"allocation_ids");
pub static USER_ALLOCATIONS: Keymap<Addr, UserAllocations> = Keymap::new(b"user_allocations_v0.0.2");
