use alloy::{
    primitives::{Address, Signed, Uint, U256},
    providers::{Provider, WalletProvider},
};

use super::*;
use crate::{
    error::ArenaError,
    types::{
        fetcher::Fetcher,
        modify_liquidity::{IPoolManager::ModifyLiquidityParams, PoolModifyLiquidityTest},
        pool_manager::{PoolManager, PoolManager::PoolKey},
        ArenaToken,
    },
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

#[derive(Debug, Clone)]
pub struct Engine {
    pub pool: PoolParameters,
    pub(crate) provider: AnvilProvider,
    pub(crate) liquidity_manager: Address,
}

impl Engine {
    pub async fn modify_liquidity(
        &self,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Bytes,
    ) -> Result<(), ArenaError> {
        let lp_manager =
            PoolModifyLiquidityTest::new(self.liquidity_manager, self.provider.clone());

        lp_manager
            .modifyLiquidity_0(key.into(), params, hook_data, false, false)
            .nonce(
                self.provider
                    .get_transaction_count(self.provider.default_signer_address())
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
