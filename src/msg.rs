use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint256, Timestamp, Binary};
use crate::state::{StakerInfo, ProviderInfo, Pool};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registration_address: Addr,
    pub anml_contract: Addr,
    pub anml_hash: String,
    pub erth_contract: Addr,
    pub erth_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UserObject {
    pub country: String,
    pub address: Addr,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub document_number: String,
    pub id_type: String,
    pub document_expiration: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Upkeep {},
    Register {user_object: UserObject},
    Mint {},
    ClaimStakingRewards {compound: bool},
    RequestUnstake {amount: Uint256},
    WithdrawUnstake {},
    InitializePool {
        other_contract: Addr,
        other_hash: String,
        initial_anml: Uint256,
        initial_other: Uint256,
    },
    AddLiquidity {
        pool_id: Addr,
        anml_deposit: Uint256,
        other_deposit: Uint256,
    },
    RequestRemoveLiquidity {
        pool_id: Addr,
        amount: Uint256,
    },
    WithdrawLiquidity {
        pool_id: Addr,
    },
    ClaimProvideRewards {
        pool_id: Addr,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint256,
        memo: Option<String>,
        msg: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Stake {compound: bool},
    Swap {token: Addr},
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    RegistrationStatus {address: Addr},
    StakeInfo {address: Addr},
    SwapSimulation {input: Addr, output: Addr, amount: Uint256},
    ReverseSwapSimulation {input: Addr, output: Addr, desired_amount: Uint256},
    PoolInfo {pool_id: Addr, address: Addr,},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct RegistrationStatusResponse {
    pub registration_status: String,
    pub last_claim: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct StakerInfoResponse {
    pub staker_info: Option<StakerInfo>,
    pub accumulated_reward: Option<Uint256>,
    pub total_staked: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct PoolInfoResponse {
    pub pool: Pool,
    pub provider_info: Option<ProviderInfo>,
    pub accumulated_reward: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct SwapSimulationResponse {
    pub amount: Uint256,
    pub scaled_price_impact: Uint256,
    pub fee: Uint256,
}

// Messages sent to SNIP-20 contracts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Msg {
    RegisterReceive {
        code_hash: String,
        padding: Option<String>,
    },
    Transfer {
        recipient: Addr,
        amount: Uint256,
        padding: Option<String>,
    },
    Mint {
        recipient: Addr,
        amount: Uint256,
    },
    TransferFrom {
        owner: Addr,
        recipient: Addr,
        amount: Uint256,
    },
    Burn {
        amount: Uint256,
    },
}

impl Snip20Msg {
    pub fn register_receive_msg(code_hash: String) -> Self {
        Snip20Msg::RegisterReceive {
            code_hash,
            padding: None, // TODO add padding calculation
        }
    }
    pub fn transfer_snip_msg(recipient: Addr, amount: Uint256) -> Self {
        Snip20Msg::Transfer {
            recipient,
            amount,
            padding: None, // TODO add padding calculation
        }
    }
    pub fn mint_msg(recipient: Addr, amount: Uint256) -> Self {
        Snip20Msg::Mint {
            recipient,
            amount,
        }
    }
    pub fn transfer_from_msg(owner: Addr, recipient: Addr, amount: Uint256) -> Self {
        Snip20Msg::TransferFrom {
            owner,
            recipient,
            amount,
        }
    }
    pub fn burn_msg(amount: Uint256) -> Self {
        Snip20Msg::Burn {
            amount,
        }
    }
}


