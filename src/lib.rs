#![warn(missing_docs)]

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
    primitives::{Address, Bytes},
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
}

/// A signal that is passed to a [`Strategy`] to provide information about the current state of the pool.
#[derive(Debug, Clone)]
pub struct Signal {
    /// Address of the pool manager.
    pub manager: Address,

    /// Key of the pool.
    pub pool: PoolKey,

    /// Current theoretical value of the pool.
    pub current_value: f64,

    /// Current step of the simulation.
    pub step: Option<usize>,
}

impl Signal {
    /// Public constructor function for a new [`Signal`].
    pub fn new(manager: Address, pool: PoolKey, current_value: f64, step: Option<usize>) -> Self {
        Self {
            manager,
            pool,
            current_value,
            step,
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
            arbitrageur::{Arbitrageur, EmptyArbitrageur},
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

        arena.run(Config::new(1)).await;
    }
}
