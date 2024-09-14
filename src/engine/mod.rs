use alloy::{
    primitives::{Address, Signed, Uint, I256},
    providers::{Provider, WalletProvider},
};

use super::*;
use crate::{
    error::ArenaError,
    types::{controller::ArenaController, pool_manager::PoolManager::PoolKey},
};
/// Defines a trait for custom arbitrage strategies.
pub mod arbitrageur;

/// Defines a trait that allows custom strategy logging and telemetry.
pub mod inspector;

/// Type that allows the parameters of a Uniswap v4 pool to be set.
#[derive(Default, Debug, Clone)]
pub struct PoolParameters {
    /// Pool fee.
    pub fee: Uint<24, 1>,

    /// Pool tick spacing.
    pub tick_spacing: Signed<24, 1>,

    /// Pool hooks.
    pub hooks: Address,
}

impl From<PoolKey> for PoolParameters {
    fn from(key: PoolKey) -> Self {
        Self {
            fee: key.fee,
            tick_spacing: key.tickSpacing,
            hooks: key.hooks,
        }
    }
}

impl PoolParameters {
    /// Public constructor function to instantiate a new `PoolParameters` struct.
    pub fn new(hooks: Address, tick_spacing: Signed<24, 1>, fee: Uint<24, 1>) -> Self {
        Self {
            fee,
            tick_spacing,
            hooks,
        }
    }
}

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
