use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("NotReceivedFunds")]
    NotReceivedFunds {},

    #[error("NotAllowZeroAmount")]
    NotAllowZeroAmount {},

    #[error("NotAllowedDenom")]
    NotAllowedDenom { denom: String },

    #[error("NotAllowedMultipleDenoms")]
    NotAllowedMultipleDenoms {},
}
