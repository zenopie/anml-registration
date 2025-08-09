use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Timestamp, Storage, StdResult};
use secret_toolkit_storage::Keymap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Registration {
    pub id_hash: String,
    pub registration_timestamp: Timestamp,
    pub last_anml_claim: Timestamp,
    pub address: Addr, // New field
}

pub struct DualKeymap<'a> { // Lifetime 'a is correct here
    pub by_address: Keymap<'a, Addr, Registration>,
    pub by_hash: Keymap<'a, String, Registration>,
}

impl<'a> DualKeymap<'a> {
    pub const fn new() -> Self {
        DualKeymap {
            by_address: Keymap::new(b"registrations_by_address_v0.0.4"), // use versioned keys
            by_hash: Keymap::new(b"registrations_by_hash_v0.0.4"),
        }
    }

    pub fn insert(
        &self,
        storage: &mut dyn Storage, // Storage reference with lifetime 'a
        address: Addr,
        hash: String,
        data: Registration,
    ) -> StdResult<()> {
        self.by_address.insert(storage, &address, &data)?;
        self.by_hash.insert(storage, &hash, &data)?;
        Ok(())
    }

    pub fn get_by_address(&self, storage: &dyn Storage, address: &Addr) -> StdResult<Option<Registration>> {
        Ok(self.by_address.get(storage, address))
    }

    pub fn get_by_hash(&self, storage: &dyn Storage, hash: &String) -> StdResult<Option<Registration>> {
        Ok(self.by_hash.get(storage, hash))
    }

    pub fn remove(&self, storage: &mut dyn Storage, address: &Addr, hash: &String) -> StdResult<()> {
        self.by_address.remove(storage, address)?;
        self.by_hash.remove(storage, hash)?;
        Ok(())
    }

}

pub const REGISTRATIONS: DualKeymap = DualKeymap::new();