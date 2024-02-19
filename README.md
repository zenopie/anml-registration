make build to compile

Instantiation Message
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registration_address: Addr,
    pub anml_contract: Addr,
    pub anml_hash: String,
}

registration address is the app api wallet that sends the ID info from the verriff platform

allow this contract to mint in the SNIP contract to be able to claim daily coin