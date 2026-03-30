// src/state/mod.rs

pub mod config;
pub mod registration;
pub mod allocation;

pub use config::{Config, CONFIG, State, STATE, ContractInfo, query_registry};
pub use registration::{REGISTRATIONS, Registration, NEW_REGISTRATIONS_COUNT};
pub use allocation::{Allocation, AllocationConfig, AllocationPercentage, AllocationState,
    UserAllocations, USER_ALLOCATIONS, ALLOCATION_OPTIONS, ALLOCATION_IDS, MAX_DESCRIPTION_LENGTH};
pub use crate::msg::{RegistrationStatusResponse};
