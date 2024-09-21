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

/// Contains error types for Arena.
pub mod error;
use alloy::{
    network::{Ethereum, EthereumWallet},
    node_bindings::{Anvil, AnvilInstance},
    primitives::{Address, Bytes, Signed, Uint},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, RootProvider,
    },
    transports::http::{Client, Http},
};

use crate::types::controller::ArenaController::PoolKey;
pub use crate::{
    arena::{Arena, ArenaBuilder},
    config::Config,
    engine::{
        arbitrageur::{Arbitrageur, EmptyArbitrageur},
        inspector::{EmptyInspector, Inspector, LogMessage, Logger},
        Engine,
    },
    feed::{Feed, GeometricBrownianMotion, OrnsteinUhlenbeck},
    strategy::Strategy,
};

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
    pub mod controller {
        use alloy_sol_macro::sol;
        sol! {
            #[sol(rpc)]
            #[derive(Debug)]
            ArenaController,
            "src/artifacts/ArenaController.json"
        }
    }
}

/// A signal that is passed to a [`Strategy`] to provide information about the current state of the pool.
#[derive(Debug, Clone)]
pub struct Signal {
    /// Current theoretical value of the pool.
    pub lex_price: Uint<256, 4>,

    /// Current step of the simulation.
    pub step: Option<usize>,

    /// Current tick of the pool.
    pub tick: Signed<24, 1>,

    /// Current price of the pool.
    pub sqrt_price_x96: Uint<160, 3>,

    /// Pool manager.
    pub manager: Address,

    /// Pool key.
    pub pool: PoolKey,

    /// Fetcher.
    pub fetcher: Address,

    /// Current value of the price feed.
    pub current_value: f64,

    /// The arena controller.
    pub controller: Address,
}

impl Signal {
    /// Public constructor function for a new [`Signal`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lex_price: Uint<256, 4>,
        step: Option<usize>,
        tick: Signed<24, 1>,
        sqrt_price_x96: Uint<160, 3>,
        manager: Address,
        pool: PoolKey,
        fetcher: Address,
        current_value: f64,
        controller: Address,
    ) -> Self {
        Self {
            lex_price,
            step,
            tick,
            sqrt_price_x96,
            manager,
            pool,
            fetcher,
            current_value,
            controller,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{Signed, Uint, I256};
    use async_trait::async_trait;
    use rug::{ops::Pow, Float};

    use super::*;
    use crate::{
        arena::{Arena, ArenaBuilder},
        config::Config,
        engine::{arbitrageur::FixedArbitrageur, inspector::EmptyInspector},
        feed::OrnsteinUhlenbeck,
        strategy::Strategy,
    };

    struct StrategyMock;

    #[async_trait]
    impl<T> Strategy<T> for StrategyMock {
        async fn init(
            &self,
            provider: AnvilProvider,
            _signal: Signal,
            _inspector: &mut Box<dyn Inspector<T>>,
            engine: Engine,
        ) {
            engine
                .modify_liquidity(
                    I256::try_from(10000000).unwrap(),
                    Signed::try_from(-887272).unwrap(),
                    Signed::try_from(887272).unwrap(),
                    Bytes::new(),
                    provider,
                )
                .await
                .unwrap();
        }
        async fn process(
            &self,
            _provider: AnvilProvider,
            _signal: Signal,
            _inspector: &mut Box<dyn Inspector<T>>,
            _engine: Engine,
        ) {
        }
    }

    #[tokio::test]
    async fn test_arena() {
        let builder: ArenaBuilder<_> = ArenaBuilder::new();

        let mut arena: Arena<_> = builder
            .with_strategy(Box::new(StrategyMock))
            .with_feed(Box::new(OrnsteinUhlenbeck::new(1.0, 0.1, 1.0, 0.1, 0.1)))
            .with_inspector(Box::new(EmptyInspector {}))
            .with_arbitrageur(Box::new(FixedArbitrageur {
                depth: Signed::try_from(10000).unwrap(),
            }))
            .build();

        arena
            .run(Config::new(
                100,
                Uint::from(0),
                Signed::try_from(2).unwrap(),
                Bytes::new(),
                Uint::from(79228162514264337593543950336_u128),
                Uint::from(0),
                Uint::from(1),
                Address::ZERO,
            ))
            .await
            .unwrap();
    }
}
