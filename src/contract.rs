use cosmwasm_std::{
    entry_point, to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp, Addr, CosmosMsg, WasmMsg,
    Uint256,
};

use crate::msg::{RegistrationStatusResponse, ExecuteMsg, InstantiateMsg, QueryMsg, UserObject, Snip20Msg, ReceiveMsg,
};
use crate::state::{State, PARAMS, Params, IDS_BY_ADDRESS, IDS_BY_DOCUMENT_NUMBER, STATE, DECLINE, Id,};
use crate::staking::{try_stake, try_claim_rewards, try_request_unstake, query_stake_info, try_complete_unstake};


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        registrations: 0,
        declines: 0,
        total_erth_staked: Uint256::from(1000000u32), //divide by zero prevention
        last_upkeep: env.block.time,
        fee_balance: Uint256::zero(),
    };
    STATE.save(deps.storage, &state)?;
    let params = Params {
        scaled_swap_fee: Uint256::from(100u32),
        registration_address: msg.registration_address,
        max_registrations: 50,
        erth_contract: msg.erth_contract,
        erth_hash: msg.erth_hash,
        anml_contract: msg.anml_contract,
        anml_hash: msg.anml_hash,
    };
    PARAMS.save(deps.storage, &params)?;
    
    let msg = to_binary(&Snip20Msg::register_receive_msg(env.contract.code_hash))?;
    let anml_register_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: params.anml_contract.to_string(),
        code_hash: params.anml_hash,
        msg: msg.clone(),
        funds: vec![],
    });
    let erth_register_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: params.erth_contract.to_string(),
        code_hash: params.erth_hash,
        msg: msg,
        funds: vec![],
    });
    let response = Response::new()
    .add_message(anml_register_message)
    .add_message(erth_register_message);
    Ok(response)
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Register {user_object} => try_register(deps, env, info, user_object),
        ExecuteMsg::Mint {} => try_mint(deps, env, info),
        ExecuteMsg::ClaimStakingRewards {compound} => try_claim_rewards(deps, env, info, compound),
        ExecuteMsg::RequestUnstake {amount} => try_request_unstake(deps, env, info, amount),
        ExecuteMsg::WithdrawUnstake {} => try_complete_unstake(deps, env, info),
        ExecuteMsg::Receive {sender, from, amount, msg, memo: _,} => try_receive(deps, env, info, sender, from, amount, msg),
    }
}

pub fn try_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint256,
    msg: Binary,
) -> Result<Response, StdError> {
    // get msg from snip recieve 
    let msg: ReceiveMsg = from_binary(&msg)?;
    // match to the correct function and send varibles
    match msg {
        ReceiveMsg::Stake{compound} => try_stake(deps, env, info, from, amount, compound),
    }   
}

pub fn try_register(deps: DepsMut, env: Env, info: MessageInfo, user_object: UserObject) -> StdResult<Response> {
    // load params
    let params = PARAMS.load(deps.storage).unwrap();
    // check that user is admin
    if info.sender != params.registration_address {
        return Err(StdError::generic_err("not authorized"));
    }
    // create namespace for document numbers by country
    let document_numbers_by_country = IDS_BY_DOCUMENT_NUMBER.add_suffix(user_object.country.as_bytes());
    // create document object
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
    // load state
    let mut state = STATE.load(deps.storage).unwrap();
    // check if document is already registered
    let already_registered_option:Option<Id> = document_numbers_by_country.get(deps.storage, &user_object.document_number);
    if already_registered_option.is_some() {
        // document already registered, set registration status to declined
        id.registration_status = "document already registered".to_string();
        // save to declined registration storage
        DECLINE.insert(deps.storage, &user_object.address, &id).unwrap();
        // update total registration number
        state.declines += 1;
    } else {
        // document is not registed, set registration status to registered
        id.registration_status = "registered".to_string();
        // save valid registration to document numbers by country storage to check for future duplicates
        document_numbers_by_country.insert(deps.storage, &user_object.document_number, &id).unwrap();
        // save valid registration to ids by address to associate with address for proof of humanity check
        IDS_BY_ADDRESS.insert(deps.storage, &user_object.address, &id).unwrap();
        // update total registration number
        state.registrations += 1;
    }
    // save state 
    STATE.save(deps.storage, &state).unwrap();
    // add attribute to tell api status of registration
    let response = Response::new()
    .add_attribute("result", id.registration_status);
    Ok(response)
}

pub fn try_mint(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    // load user data
    let user_data_option: Option<Id> = IDS_BY_ADDRESS.get(deps.storage, &info.sender);
    // if user data exists assign it to the user_data variable
    match user_data_option {
        Some(mut user_data) => {
            //find elapsed time since last claim
            let elapsed_time = env.block.time.seconds() - user_data.last_anml_claim.seconds();
            // compare elapsed time with 1 day (86400 seconds)
            let seconds_in_a_day = 86400;
            if elapsed_time < seconds_in_a_day {
                // If less than one day has passed, return an error
                return Err(StdError::generic_err("One day hasn't passed since the last claim"));
            }
            let midnight_calculation = (env.block.time.seconds() / seconds_in_a_day) * seconds_in_a_day;
            let midnight_timestamp = Timestamp::from_seconds(midnight_calculation);
            user_data.last_anml_claim = midnight_timestamp;
            // save last claim
            IDS_BY_ADDRESS.insert(deps.storage, &info.sender, &user_data).unwrap();
            // load state
            let params = PARAMS.load(deps.storage).unwrap();
            // Create the message to send to the other contract
            let msg = to_binary(&Snip20Msg::mint_msg(
                info.sender.clone(),
                Uint256::from(1000000u32),
            ))?;
            // Create the contract execution message
            let execute_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: params.anml_contract.to_string(),
                code_hash: params.anml_hash.to_string(),
                funds: vec![],
                msg: msg,
            });
            // Return the execution message in the Response
            let response = Response::new()
            .add_attribute("result", "success")
            .add_message(execute_msg);
            Ok(response)
        },
        None => {
            // Return an error if user_data_option is None
            return Err(StdError::generic_err("User data not found"))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::RegistrationStatus {address} => to_binary(&query_anml_status(deps, address)?),
        QueryMsg::StakeInfo {address} => to_binary(&query_stake_info(deps, env, address)?),
    }
}

fn query_anml_status(deps: Deps, address: Addr) -> StdResult<RegistrationStatusResponse> {
    // initiate variable for sendback
    let registration_status;
    let last_claim;
    // see if address is registered
    let user_data_option:Option<Id> = IDS_BY_ADDRESS.get(deps.storage, &address);
    match user_data_option {
        Some(user_data) => {
            // address is registered
            registration_status = "registered".to_string();
            // send claim timestamp
            last_claim = user_data.last_anml_claim;
        },
        None => {
            // address isn't registed
            registration_status = "not_registed".to_string();
            last_claim = Timestamp::default();
        }  
    } 
    let response = RegistrationStatusResponse {
        registration_status: registration_status,
        last_claim: last_claim,
    };
    Ok(response)
}
