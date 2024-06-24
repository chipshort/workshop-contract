use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

use crate::state::Metadata;

#[cw_serde]
pub struct InstantiateMsg {
    pub registration_fee: Coin,
    pub transfer_fee: Coin,
}

#[cw_serde]
pub enum ExecuteMsg {
    Register { name: String, metadata: Metadata },
    Transfer { name: String, to: String },
    Change { name: String, metadata: Metadata },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Entry)]
    Lookup { name: String },

    #[returns(EntryListResponse)]
    List {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(cosmwasm_std::ValidatorResponse)]
    QueryValidatorInfo { name: String },
}

#[cw_serde]
struct EntryListResponse {
    entries: Vec<crate::state::Entry>,
}
