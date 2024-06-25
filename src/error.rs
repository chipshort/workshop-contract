use cosmwasm_std::{Coin, CoinsError, StdError};
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Coins(#[from] CoinsError),

    #[error("Validation Error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Expected {expected} as registration fee but received {given:?}")]
    InvalidRegistrationFee { given: Vec<Coin>, expected: Coin },

    #[error("The owner of the entry is not a validator")]
    NotAValidator {},

    #[error("Unauthorized")]
    Unauthorized {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
