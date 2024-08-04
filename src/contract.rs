use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp, Addr, CosmosMsg, WasmMsg,
    Uint128,
};

use crate::msg::{RegistrationStatusResponse, ExecuteMsg, InstantiateMsg, QueryMsg, UserObject, Snip20Msg,
    UpdateStateMsg, StateResponse,
};
use crate::state::{State, IDS_BY_ADDRESS, IDS_BY_DOCUMENT_NUMBER, STATE, Id,};


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        registrations: 0,
        manager_address: msg.manager_address,
        registration_address: msg.registration_address,
        max_registrations: 50,
        anml_contract: msg.anml_contract,
        anml_hash: msg.anml_hash,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateState {msg} => execute_update_state(deps, env, info, msg),
        ExecuteMsg::Register {user_object} => try_register(deps, env, info, user_object),
        ExecuteMsg::Claim {} => try_claim(deps, env, info),
    }
}

pub fn execute_update_state(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateStateMsg,
) -> Result<Response, StdError> {
    // Load the state
    let mut state = STATE.load(deps.storage)?;

    // Only allow the contract manager to update the state
    if info.sender != state.manager_address {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(registrations) = msg.registrations {
        state.registrations = registrations;
    }
    if let Some(registration_address) = msg.registration_address {
        state.registration_address = registration_address;
    }
    if let Some(manager_address) = msg.manager_address {
        state.manager_address = manager_address;
    }
    if let Some(max_registrations) = msg.max_registrations {
        state.max_registrations = max_registrations;
    }
    if let Some(anml_contract) = msg.anml_contract {
        state.anml_contract = anml_contract;
    }
    if let Some(anml_hash) = msg.anml_hash {
        state.anml_hash = anml_hash;
    }

    // Save the updated state
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "update_state")
        .add_attribute("registrations", state.registrations.to_string())
        .add_attribute("registration_address", state.registration_address.to_string())
        .add_attribute("manager_address", state.manager_address.to_string())
        .add_attribute("max_registrations", state.max_registrations.to_string())
        .add_attribute("anml_contract", state.anml_contract.to_string())
        .add_attribute("anml_hash", state.anml_hash.clone()))
}


pub fn try_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_object: UserObject,
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;

    // Check if the sender is authorized
    if info.sender != state.registration_address {
        return Err(StdError::generic_err("not authorized"));
    }

    // Check if registrations are maxed out
    if state.registrations >= state.max_registrations {
        return Err(StdError::generic_err("max registrations"));
    }

    // Create namespace for document numbers by country
    let document_numbers_by_country =
        IDS_BY_DOCUMENT_NUMBER.add_suffix(user_object.country.as_bytes());

    // Create document object
    let mut id = Id {
        registration_status: "not assigned".to_string(),
        country: user_object.country,
        address: user_object.address.clone(),
        first_name: user_object.first_name,
        last_name: user_object.last_name,
        date_of_birth: user_object.date_of_birth,
        document_number: user_object.document_number.clone(),
        id_type: user_object.id_type,
        document_expiration: user_object.document_expiration,
        registration_timestamp: env.block.time,
        last_anml_claim: Timestamp::from_nanos(0),
    };

    // Check if document is already registered
    if document_numbers_by_country
        .get(deps.storage, &user_object.document_number)
        .is_some()
    {
        // Document already registered, set registration status to declined
        id.registration_status = "document already registered".to_string();
    } else {
        // Document is not registered, set registration status to registered
        id.registration_status = "registered".to_string();
        document_numbers_by_country.insert(deps.storage, &user_object.document_number, &id)?;
        IDS_BY_ADDRESS.insert(deps.storage, &user_object.address, &id)?;
        state.registrations += 1;
    }

    // Save state
    STATE.save(deps.storage, &state)?;

    // Add attribute to tell API status of registration
    let response = Response::new().add_attribute("result", id.registration_status);
    Ok(response)
}

pub fn try_claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    if let Some(mut user_data) = IDS_BY_ADDRESS.get(deps.storage, &info.sender) {
        let elapsed_time = env.block.time.seconds() - user_data.last_anml_claim.seconds();
        let seconds_in_a_day = 86400;
        if elapsed_time < seconds_in_a_day {
            return Err(StdError::generic_err(
                "One day hasn't passed since the last claim",
            ));
        }

        let midnight_timestamp = Timestamp::from_seconds((env.block.time.seconds() / seconds_in_a_day) * seconds_in_a_day);
        user_data.last_anml_claim = midnight_timestamp;
        IDS_BY_ADDRESS.insert(deps.storage, &info.sender, &user_data)?;

        let state = STATE.load(deps.storage)?;

        let msg = to_binary(&Snip20Msg::mint_msg(info.sender.clone(), Uint128::from(1000000u32)))?;
        let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.anml_contract.to_string(),
            code_hash: state.anml_hash.clone(),
            funds: vec![],
            msg,
        });

        let response = Response::new()
            .add_attribute("result", "success")
            .add_message(execute_msg);
        Ok(response)
    } else {
        Err(StdError::generic_err("User data not found"))
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryState {} => to_binary(&query_state(deps)?),
        QueryMsg::RegistrationStatus { address } => to_binary(&query_anml_status(deps, address)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse { state: state })
}

fn query_anml_status(deps: Deps, address: Addr) -> StdResult<RegistrationStatusResponse> {
    let registration_status;
    let last_claim;

    if let Some(user_data) = IDS_BY_ADDRESS.get(deps.storage, &address) {
        registration_status = true;
        last_claim = user_data.last_anml_claim;
    } else {
        registration_status = false;
        last_claim = Timestamp::default();
    }

    let response = RegistrationStatusResponse {
        registration_status,
        last_claim,
    };
    Ok(response)
}

