// src/query/mod.rs
use cosmwasm_std::{Deps, Env, Binary, StdResult, to_binary, Timestamp,};
use crate::msg::{QueryMsg, RegistrationStatusResponse};
use crate::state::{USER_ALLOCATIONS, AllocationPercentage, ALLOCATION_OPTIONS, Allocation,
    STATE, State, Config, CONFIG, REGISTRATIONS};


pub fn query_dispatch(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryState {} => to_binary(&query_state(deps)?),
        QueryMsg::QueryConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::QueryRegistrationStatus { address } => to_binary(&query_registration_status(deps, env, address)?),
        QueryMsg::QueryAllocationOptions {} => to_binary(&query_allocation_options(deps)?),
        QueryMsg::QueryUserAllocations{address} => to_binary(&query_user_allocations(deps, address)?),
    }
}

fn query_state(deps: Deps) -> StdResult<State> {
    let state = STATE.load(deps.storage)?;
    Ok(state)
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_registration_status(deps: Deps, env: Env, address: String) -> StdResult<RegistrationStatusResponse> {

    // Validate the provided address to ensure it's a valid format.
    let addr = deps.api.addr_validate(&address)?;

    // Load config to get registration validity period
    let config = CONFIG.load(deps.storage)?;

    // Get current block time
    let current_time = env.block.time;

    // Attempt to retrieve the registration data
    let (registration_status, last_claim) = match REGISTRATIONS.get_by_address(deps.storage, &addr)? {
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
    };

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
