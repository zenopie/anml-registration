// src/state/mod.rs

pub mod config;
pub mod registration;
pub mod allocation;

pub use config::{Config, CONFIG, State, STATE};
pub use registration::{REGISTRATIONS, Registration};
pub use allocation::{Allocation, AllocationConfig, AllocationPercentage, AllocationState,
    USER_ALLOCATIONS, ALLOCATION_OPTIONS};
pub use crate::msg::{RegistrationStatusResponse};

