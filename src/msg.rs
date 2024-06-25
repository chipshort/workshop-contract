use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use validator::Validate;

use crate::state::Metadata;

#[cw_serde]
pub struct InstantiateMsg {
    pub registration_fee: Coin,
    pub transfer_fee: Coin,
}

#[cw_serde]
pub enum ExecuteMsg {
    Register(RegistrationMsg),
    Transfer { name: String, to: String },
    SendFundsTo { name: String },
    // ... Change, etc.
}

#[cw_serde]
#[derive(Validate)]
pub struct RegistrationMsg {
    #[validate(length(min = 3, max = 64))]
    pub name: String,
    #[validate(nested)]
    pub metadata: Metadata,
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

    #[returns(cosmwasm_std::Validator)]
    QueryValidatorInfo { name: String },
}

#[cw_serde]
pub struct EntryListResponse {
    pub entries: Vec<(String, crate::state::Entry)>,
}
