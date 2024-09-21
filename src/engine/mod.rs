use alloy::{
    primitives::{Address, Signed, I256},
    providers::{Provider, WalletProvider},
};

use super::*;
use crate::{
    error::ArenaError,
    types::controller::ArenaController,
};
/// Defines a trait for custom arbitrage strategies.
pub mod arbitrageur;

/// Defines a trait that allows custom strategy logging and telemetry.
pub mod inspector;

/// Abstraction to allow strategies to call state changing functions on the PoolManager without having to worry about callbacks.
#[derive(Debug, Clone)]
pub struct Engine {
    pub(crate) controller: Address,
}

#[allow(clippy::redundant_closure)]
impl Engine {
    /// Modify pool liquidity.
    pub async fn modify_liquidity(
        &self,
        liquidity_delta: I256,
        tick_lower: Signed<24, 1>,
        tick_upper: Signed<24, 1>,
        provider: AnvilProvider,
    ) -> Result<(), ArenaError> {
        let controller = ArenaController::new(self.controller, provider.clone());

        controller
            .addLiquidity(liquidity_delta, tick_lower, tick_upper)
            .nonce(
                provider
                    .get_transaction_count(provider.default_signer_address())
                    .await
                    .unwrap(),
            )
            .send()
            .await
            .map_err(ArenaError::ContractError)?
            .watch()
            .await
            .map_err(|e| ArenaError::PendingTransactionError(e))?;

        Ok(())
    }
}
