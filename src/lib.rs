#![warn(missing_docs)]
//! Arena

/// Defines the main simulation runtime.
pub mod arena;

/// Contains configuration types for the simulation.
pub mod config;

/// Contains the types for various price processes.
pub mod feed;

/// Defines the base strategy trait.
pub mod strategy;

/// Defines core simulation logic types, such as an [`Arbitrageur`].
pub mod engine;

use alloy::{
    network::{Ethereum, EthereumWallet},
    node_bindings::{Anvil, AnvilInstance},
    primitives::{Address, Bytes, U256},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, RootProvider,
    },
    transports::http::{Client, Http},
};

use crate::{engine::inspector::Inspector, types::PoolManager::PoolKey};

/// Provider type that includes all necessary fillers to execute transactions on an [`Anvil`] node.
pub type AnvilProvider = FillProvider<
    JoinFill<
        JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

mod types {
    #![allow(clippy::too_many_arguments)]
    use alloy_sol_macro::sol;

    use crate::types::{
        Fetcher::PoolKey as FetcherPoolKey, PoolManager::PoolKey as ManagerPoolKey,
        PoolSwapTest::PoolKey as SwapPoolKey,
    };

    sol! {
        #[sol(rpc)]
        #[derive(Debug, Default)]
        PoolManager,
        "src/artifacts/PoolManager.json"
    }

    sol! {
        #[sol(rpc)]
        #[derive(Debug)]
        LiquidExchange,
        "src/artifacts/LiquidExchange.json"
    }

    sol! {
        #[sol(rpc)]
        #[derive(Debug)]
        ArenaToken,
        "src/artifacts/ArenaToken.json"
    }

    sol! {
        #[sol(rpc)]
        #[derive(Debug)]
        Fetcher,
        "src/artifacts/Fetcher.json"
    }

    sol! {
        #[sol(rpc)]
        #[derive(Debug)]
        PoolSwapTest,
        "src/artifacts/PoolSwapTest.json"
    }

    impl From<FetcherPoolKey> for ManagerPoolKey {
        fn from(fetcher: FetcherPoolKey) -> Self {
            ManagerPoolKey {
                currency0: fetcher.currency0,
                currency1: fetcher.currency1,
                fee: fetcher.fee,
                tickSpacing: fetcher.tickSpacing,
                hooks: fetcher.hooks,
            }
        }
    }

    impl From<ManagerPoolKey> for FetcherPoolKey {
        fn from(manager: ManagerPoolKey) -> Self {
            FetcherPoolKey {
                currency0: manager.currency0,
                currency1: manager.currency1,
                fee: manager.fee,
                tickSpacing: manager.tickSpacing,
                hooks: manager.hooks,
            }
        }
    }

    impl From<SwapPoolKey> for ManagerPoolKey {
        fn from(swap: SwapPoolKey) -> Self {
            ManagerPoolKey {
                currency0: swap.currency0,
                currency1: swap.currency1,
                fee: swap.fee,
                tickSpacing: swap.tickSpacing,
                hooks: swap.hooks,
            }
        }
    }

    impl From<ManagerPoolKey> for SwapPoolKey {
        fn from(manager: ManagerPoolKey) -> Self {
            SwapPoolKey {
                currency0: manager.currency0,
                currency1: manager.currency1,
                fee: manager.fee,
                tickSpacing: manager.tickSpacing,
                hooks: manager.hooks,
            }
        }
    }
}

/// A signal that is passed to a [`Strategy`] to provide information about the current state of the pool.
#[derive(Debug, Clone)]
pub struct Signal {
    /// Address of the pool manager.
    pub manager: Address,

    /// Address of the fetcher.
    pub fetcher: Address,

    /// Key of the pool.
    pub pool: PoolKey,

    /// Current theoretical value of the pool.
    pub current_value: f64,

    /// Current step of the simulation.
    pub step: Option<usize>,

    /// Current tick of the pool.
    pub tick: i32,

    /// Current price of the pool.
    pub sqrt_price_x96: U256,
}

impl Signal {
    /// Public constructor function for a new [`Signal`].
    pub fn new(
        manager: Address,
        fetcher: Address,
        pool: PoolKey,
        current_value: f64,
        step: Option<usize>,
        tick: i32,
        sqrt_price_x96: U256,
    ) -> Self {
        Self {
            manager,
            fetcher,
            pool,
            current_value,
            step,
            tick,
            sqrt_price_x96,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        arena::{Arena, ArenaBuilder},
        config::Config,
        engine::{
            arbitrageur::{Arbitrageur, DefaultArbitrageur, EmptyArbitrageur},
            inspector::EmptyInspector,
        },
        feed::OrnsteinUhlenbeck,
        strategy::Strategy,
    };

    struct StrategyMock;

    impl<V> Strategy<V> for StrategyMock {
        fn init(
            &self,
            _provider: AnvilProvider,
            _signal: Signal,
            _inspector: &mut Box<dyn Inspector<V>>,
        ) {
        }
        fn process(
            &self,
            _provider: AnvilProvider,
            _signal: Signal,
            _inspector: &mut Box<dyn Inspector<V>>,
        ) {
        }
    }

    #[tokio::test]
    async fn test_arena() {
        let builder: ArenaBuilder<_> = ArenaBuilder::new();

        let mut arena: Arena<f64> = builder
            .with_strategy(Box::new(StrategyMock {}))
            .with_fee(4000)
            .with_tick_spacing(2)
            .with_feed(Box::new(OrnsteinUhlenbeck::new(0.1, 0.1, 0.1, 0.1, 0.1)))
            .with_inspector(Box::new(EmptyInspector {}))
            .with_arbitrageur(Box::new(EmptyArbitrageur {}))
            .build();

        arena.run(Config::new(2)).await;
    }
}
