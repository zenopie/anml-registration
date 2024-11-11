// src/execute/registration.rs
use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, Timestamp, CosmosMsg, WasmMsg,
    to_binary,
};
use secret_toolkit::snip20;
use crate::msg::UserObject;
use crate::state::{CONFIG, STATE, Id, IDS_BY_ADDRESS, IDS_BY_DOCUMENT_NUMBER};

pub fn register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_object: UserObject,
    affiliate: Option<String>,
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    // Check if the sender is authorized
    if info.sender != config.registration_address {
        return Err(StdError::generic_err("Not authorized"));
    }

    // Check if registrations are maxed out
    if state.registrations >= config.max_registrations {
        return Err(StdError::generic_err("Max registrations reached"));
    }

    let wallet_address_addr = deps.api.addr_validate(&user_object.address)?;
    // Create namespace for document numbers by country
    let document_numbers_by_country =
        IDS_BY_DOCUMENT_NUMBER.add_suffix(user_object.country.as_bytes());

    // Create document object
    let mut id = Id {
        registration_status: "not assigned".to_string(),
        country: user_object.country,
        wallet_address: wallet_address_addr.clone(),
        first_name: user_object.first_name,
        last_name: user_object.last_name,
        date_of_birth: Timestamp::from_seconds(user_object.date_of_birth),
        document_number: user_object.document_number.clone(),
        id_type: user_object.id_type,
        document_expiration: Timestamp::from_seconds(user_object.document_expiration),
        registration_timestamp: env.block.time,
        last_anml_claim: Timestamp::from_nanos(0),
    };

    // Check if document is already registered
    if document_numbers_by_country.get(deps.storage, &user_object.document_number).is_some() {
        return Err(StdError::generic_err("Document already registered"));
    }

    // Document is not registered, set registration status to registered
    id.registration_status = "registered".to_string();
    document_numbers_by_country.insert(deps.storage, &user_object.document_number, &id)?;
    IDS_BY_ADDRESS.insert(deps.storage, &wallet_address_addr, &id)?;
    state.registrations += 1;

    // Calculate the 1% registration reward
    let reward = state.registration_reward.u128() / 100;

    // Subtract the total reward from state.registration_reward
    let total_reward = if affiliate.is_some() {
        reward * 3 // 1% to registree, 1% to registration wallet, 1% to affiliate
    } else {
        reward * 2 // 1% to registree, 1% to registration wallet
    };
    state.registration_reward = Uint128::from(state.registration_reward.u128().saturating_sub(total_reward));

    // Create SNIP-20 transfer messages
    let mut messages = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.erth_token_contract.to_string(),
            code_hash: config.erth_token_hash.clone(),
            msg: to_binary(&snip20::HandleMsg::Transfer {
                recipient: wallet_address_addr.to_string(),
                amount: Uint128::from(reward),
                memo: None,
                padding: None,
            })?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.erth_token_contract.to_string(),
            code_hash: config.erth_token_hash.clone(),
            msg: to_binary(&snip20::HandleMsg::Transfer {
                recipient: config.registration_wallet.to_string(),
                amount: Uint128::from(reward),
                memo: None,
                padding: None,
            })?,
            funds: vec![],
        }),
    ];

    // If there's an affiliate, send them 1% as well
    if let Some(affiliate_address) = affiliate {
        let affiliate_addr = deps.api.addr_validate(&affiliate_address)?;
        messages.push(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.erth_token_contract.to_string(),
                code_hash: config.erth_token_hash.clone(),
                msg: to_binary(&snip20::HandleMsg::Transfer {
                    recipient: affiliate_addr.to_string(),
                    amount: Uint128::from(reward),
                    memo: None,
                    padding: None,
                })?,
                funds: vec![],
            }),
        );
    }

    // Update state after successful registration
    STATE.save(deps.storage, &state)?;

    // Respond with the transaction messages
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("result", id.registration_status))
}
