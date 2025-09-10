// src/query/mod.rs
use cosmwasm_std::{Deps, Env, Binary, StdResult, to_binary, Timestamp,};
use crate::msg::{QueryMsg, RegistrationStatusResponse, StateResponse};
use crate::state::{USER_ALLOCATIONS, AllocationPercentage, ALLOCATION_OPTIONS, Allocation,
    STATE, Config, CONFIG, REGISTRATIONS, Registration, NEW_REGISTRATIONS_COUNT};


pub fn query_dispatch(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryState {} => to_binary(&query_state(deps)?),
        QueryMsg::QueryConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::QueryRegistrationStatus { address } => to_binary(&query_registration_status(deps, env, address)?),
        QueryMsg::QueryRegistrationStatusByIdHash { id_hash } => to_binary(&query_registration_status_by_id_hash(deps, env, id_hash)?),
        QueryMsg::QueryAllocationOptions {} => to_binary(&query_allocation_options(deps)?),
        QueryMsg::QueryUserAllocations{address} => to_binary(&query_user_allocations(deps, address)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    let new_registrations = NEW_REGISTRATIONS_COUNT.may_load(deps.storage)?.unwrap_or(0);
    
    Ok(StateResponse {
        registrations: state.registrations,
        new_registrations,
        last_anml_buyback: state.last_anml_buyback,
        total_allocations: state.total_allocations,
        allocation_counter: state.allocation_counter,
        registration_reward: state.registration_reward,
        last_upkeep: state.last_upkeep,
    })
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

/// Helper function to check registration validity, avoiding code duplication.
fn check_registration_validity(
    registration_opt: Option<Registration>,
    config: &Config,
    current_time: Timestamp,
) -> (bool, Timestamp) {
    match registration_opt {
        Some(registration) => {
            // Check if registration is still valid
            let registration_age = current_time.seconds() - registration.registration_timestamp.seconds();
            if registration_age > config.registration_validity_seconds {
                // Registration has expired
                (false, Timestamp::default())
            } else {
                // Registration is still valid
                (true, registration.last_anml_claim)
            }
        }
        None => (false, Timestamp::default()),
    }
}

// Updated original function to use the helper
pub fn query_registration_status(deps: Deps, env: Env, address: String) -> StdResult<RegistrationStatusResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let config = CONFIG.load(deps.storage)?;
    let current_time = env.block.time;

    // Retrieve the registration data by address
    let registration_opt = REGISTRATIONS.get_by_address(deps.storage, &addr)?;

    // Use the helper to determine status
    let (registration_status, last_claim) = check_registration_validity(registration_opt, &config, current_time);

    Ok(RegistrationStatusResponse {
        registration_status,
        last_claim,
    })
}

// New query function to get status by ID hash
pub fn query_registration_status_by_id_hash(deps: Deps, env: Env, id_hash: String) -> StdResult<RegistrationStatusResponse> {
    let config = CONFIG.load(deps.storage)?;
    let current_time = env.block.time;

    // Retrieve the registration data by hash using your DualKeymap
    let registration_opt = REGISTRATIONS.get_by_hash(deps.storage, &id_hash)?;

    // Use the same helper to determine status
    let (registration_status, last_claim) = check_registration_validity(registration_opt, &config, current_time);

    Ok(RegistrationStatusResponse {
        registration_status,
        last_claim,
    })
}


fn query_allocation_options(deps: Deps) -> StdResult<Vec<Allocation>> {

    // Load allocations options
    let allocations = ALLOCATION_OPTIONS.load(deps.storage)?;

    Ok(allocations)
}

pub fn query_user_allocations(deps: Deps, address: String) -> StdResult<Vec<AllocationPercentage>> {

    let addr = deps.api.addr_validate(&address)?;

    // Load user info or use default if not found
    let allocation_percentages = USER_ALLOCATIONS.get(deps.storage, &addr).unwrap_or_default();

    Ok(allocation_percentages)
}
