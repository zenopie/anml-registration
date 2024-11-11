// src/query/mod.rs
use cosmwasm_std::{Deps, Env, Binary, StdResult, to_binary, Timestamp,};
use crate::msg::{QueryMsg, RegistrationStatusResponse};
use crate::state::{USER_ALLOCATIONS, AllocationPercentage, ALLOCATION_OPTIONS, Allocation, IDS_BY_ADDRESS,
    STATE, State, Config, CONFIG};


pub fn query_dispatch(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryState {} => to_binary(&query_state(deps)?),
        QueryMsg::QueryConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::QueryRegistrationStatus { address } => to_binary(&query_anml_status(deps, address)?),
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

fn query_anml_status(deps: Deps, address: String) -> StdResult<RegistrationStatusResponse> {
    // Validate the provided address to ensure it's a valid format.
    let addr = deps.api.addr_validate(&address)?;

    // Attempt to retrieve the user data from storage using the validated address.
    let (registration_status, last_claim) = match IDS_BY_ADDRESS.get(deps.storage, &addr) {
        // If user data is found, set registration_status to true and use the last claim timestamp.
        Some(user_data) => (true, user_data.last_anml_claim),
        // If no user data is found, set registration_status to false and use the default timestamp.
        None => (false, Timestamp::default()),
    };

    // Create and return the response containing the registration status and last claim timestamp.
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
