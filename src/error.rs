use thiserror::Error;

/// Error type for Arena.
#[derive(Error, Debug)]
pub enum ArenaError {
    /// Contract interaction failed.
    #[error("alloy contract error {0}")]
    ContractError(#[from] alloy_contract::Error),

    /// Pending transaction error.
    #[error("alloy pending transaction error {0}")]
    PendingTransactionError(#[from] alloy::providers::PendingTransactionError),

    /// Conversion error when parsing ether values.
    #[error("alloy conversion error {0}")]
    ConversionError(#[from] alloy::primitives::utils::UnitsError),
}
