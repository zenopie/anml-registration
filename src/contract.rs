use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp, Addr, CosmosMsg, WasmMsg,
};
use secret_toolkit::snip20;


use crate::msg::{RegistrationStatusResponse, ExecuteMsg, InstantiateMsg, QueryMsg, UserObject,
    StateResponse, MigrateMsg, SendMsg,
};
use crate::state::{State, IDS_BY_ADDRESS, IDS_BY_DOCUMENT_NUMBER, STATE, Id,};


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let registration_address_addr = deps.api.addr_validate(&msg.registration_address)?;
    let anml_token_contract_addr = deps.api.addr_validate(&msg.anml_token_contract)?;
    let erth_token_contract_addr = deps.api.addr_validate(&msg.erth_token_contract)?;
    let contract_manager_addr = deps.api.addr_validate(&msg.contract_manager)?;
    let anml_pool_contract_addr = deps.api.addr_validate(&msg.anml_pool_contract)?;

    let state = State {
        registrations: 0,
        contract_manager: contract_manager_addr,
        registration_address: registration_address_addr,
        max_registrations: 50,
        anml_token_contract: anml_token_contract_addr,
        anml_token_hash: msg.anml_token_hash,
        erth_token_contract: erth_token_contract_addr,
        erth_token_hash: msg.erth_token_hash,
        anml_pool_contract: anml_pool_contract_addr,
        anml_pool_hash: msg.anml_pool_hash,
        last_anml_buyback: env.block.time,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateState {key, value} => execute_update_state(deps, env, info, key, value),
        ExecuteMsg::Register {user_object} => try_register(deps, env, info, user_object),
        ExecuteMsg::Claim {} => try_claim(deps, env, info),
    }
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

        let buyback_elapsed = (env.block.time.seconds() - state.last_anml_buyback.seconds()) * 1000000;

        let mut messages = vec![];

        // Create messages for transferring tokens from the user to the contract using allowances
        let mint_anml = snip20::HandleMsg::Mint {
            recipient: info.sender.clone().to_string(),
            amount: 1000000u32.into(),
            padding: None,
            memo: None,
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.anml_token_contract.to_string(),
            code_hash: state.anml_token_hash.clone(),
            msg: to_binary(&mint_anml)?,
            funds: vec![],
        }));
        // Create messages for minting ERTH for the ANML buyback
        let mint_erth = snip20::HandleMsg::Mint {
            recipient: env.contract.address.to_string(),
            amount: buyback_elapsed.into(),
            padding: None,
            memo: None,
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.erth_token_contract.to_string(),
            code_hash: state.erth_token_hash.clone(),
            msg: to_binary(&mint_erth)?,
            funds: vec![],
        }));
        // Swap Erth for ANML
        let swap_msg = snip20::HandleMsg::Send {
            recipient: state.anml_pool_contract.to_string(),
            recipient_code_hash: Some(state.anml_token_hash.clone()),
            amount: buyback_elapsed.into(),
            msg: Some(to_binary(&SendMsg::AnmlBuyback {})?),
            memo: None,
            padding: None,
        };

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.anml_token_contract.to_string(),
            code_hash: state.anml_token_hash.clone(),
            msg: to_binary(&swap_msg)?,
            funds: vec![],
        }));
      

        let response = Response::new()
            .add_messages(messages)
            .add_attribute("result", "success");
        Ok(response)
    } else {
        return Err(StdError::generic_err("User data not found"))
    }
}

pub fn execute_update_state(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    key: String,
    value: String,
) -> Result<Response, StdError> {
    let mut state = STATE.load(deps.storage)?;

    if info.sender != state.contract_manager {
        return Err(StdError::generic_err("unauthorized"));
    }

    match key.as_str() {
        "contract_manager" => {
            state.contract_manager = deps.api.addr_validate(&value)?;
        }
        "registration_address" => {
            state.registration_address = deps.api.addr_validate(&value)?;
        }
        "max_registrations" => {
            let max_registrations: u32 = value.parse().map_err(|_| StdError::generic_err("Invalid max_registrations"))?;
            state.max_registrations = max_registrations;
        }
        "anml_token_hash" => {
            state.anml_token_hash = value.clone();
        }
        "erth_token_hash" => {
            state.erth_token_hash = value.clone();
        }
        "anml_pool_contract" => {
            state.anml_pool_contract = deps.api.addr_validate(&value)?;
        }
        "anml_pool_hash" => {
            state.anml_pool_hash = value.clone();
        }
        _ => return Err(StdError::generic_err("Invalid state key")),
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "update_state")
        .add_attribute("key", key)
        .add_attribute("value", value))
}


#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    match msg {
        MigrateMsg::Migrate {} => {

            Ok(Response::new().add_attribute("action", "migrate"))
        }
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

