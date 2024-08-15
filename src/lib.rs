#![warn(missing_docs)]
#[doc = include_str!("../README.md")]

/// Defines the main simulation runtime.
pub mod arena;

/// Contains configuration types for the simulation.
pub mod config;

/// Contains the types for various price processes.
pub mod feed;

/// Defines the base strategy trait.
pub mod strategy;

/// Defines the [`Inspector`] trait.
pub mod inspector;

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

use crate::{inspector::Inspector, types::PoolManager::PoolKey};

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
    use alloy_sol_macro::sol;

    #[allow(missing_docs)]
    sol! {
        #[sol(rpc)]
        #[derive(Debug, Default)]
        PoolManager,
        "contracts/v4-core/out/PoolManager.sol/PoolManager.json"
    }

    #[allow(missing_docs)]
    sol! {
        #[sol(rpc)]
        #[derive(Debug)]
        LiquidExchange,
        "contracts/utils/out/LiquidExchange.sol/LiquidExchange.json"
    }

    #[allow(missing_docs)]
    sol! {
        #[sol(rpc)]
        #[derive(Debug)]
        ArenaToken,
        "contracts/utils/out/ArenaToken.sol/ArenaToken.json"
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
        feed::OrnsteinUhlenbeck,
        strategy::Strategy,
    };
    struct StrategyMock;
    struct InspectorMock;

    impl Inspector<f64> for InspectorMock {
        fn inspect(&self, _step: usize) -> Option<f64> {
            None
        }
        fn log(&mut self, _value: f64) {}
        fn save(&self) {}
    }

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
            .with_inspector(Box::new(InspectorMock {}))
            .build();

        arena.run(Config::new(0)).await;
    }
}
