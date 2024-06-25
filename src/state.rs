use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use validator::Validate;

#[cw_serde]
pub struct Config {
    pub registration_fee: Coin,
    pub transfer_fee: Coin,
}

#[cw_serde]
pub struct Entry {
    pub owner: Addr,
    pub metadata: Metadata,
}

#[cw_serde]
#[derive(Validate)]
pub struct Metadata {
    #[validate(url)]
    pub url: Option<String>,
    pub validator_address: Option<String>,
    // ...
}

pub const ENTRIES: Map<&str, Entry> = Map::new("entries");
pub const CONFIG: Item<Config> = Item::new("config");
