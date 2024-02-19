This project is a simple Secret network implementation of the Variff API that matches people to their IDs and scrapes the information and sends it to a registration wallet to publish on-chain in private meta-data, this contract compares the info with previous registrations and completes registration if unique. There is a minting function to mint one ANML token per day for registrees.
the steps to build your own version of this repository are below.

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