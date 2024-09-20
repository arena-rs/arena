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
        Engine, PoolParameters,
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
    #![allow(clippy::too_many_arguments)]
    use alloy_sol_macro::sol;

    use crate::types::{
        controller::ArenaController::PoolKey as ControllerPoolKey,
        fetcher::Fetcher::PoolKey as FetcherPoolKey,
        modify_liquidity::PoolModifyLiquidityTest::PoolKey as ModifyLiquidityPoolKey,
        pool_manager::PoolManager::PoolKey as ManagerPoolKey,
        swap::PoolSwapTest::PoolKey as SwapPoolKey,
    };

    pub mod pool_manager {
        use alloy_sol_macro::sol;
        sol! {
            #[sol(rpc)]
            #[derive(Debug, Default)]
            PoolManager,
            "src/artifacts/PoolManager.json"
        }
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

    pub mod fetcher {
        use alloy_sol_macro::sol;
        sol! {
            #[sol(rpc)]
            #[derive(Debug)]
            Fetcher,
            "src/artifacts/Fetcher.json"
        }
    }

    pub mod swap {
        use alloy_sol_macro::sol;
        sol! {
            #[sol(rpc)]
            #[derive(Debug)]
            PoolSwapTest,
            "src/artifacts/PoolSwapTest.json"
        }
    }

    pub mod modify_liquidity {
        use alloy_sol_macro::sol;
        sol! {
            #[sol(rpc)]
            #[derive(Debug)]
            PoolModifyLiquidityTest,
            "src/artifacts/PoolModifyLiquidityTest.json"
        }
    }

    pub mod controller {
        use alloy_sol_macro::sol;
        sol! {
            #[sol(rpc)]
            #[derive(Debug)]
            ArenaController,
            "src/artifacts/ArenaController.json"
        }
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

    impl From<FetcherPoolKey> for ControllerPoolKey {
        fn from(fetcher: FetcherPoolKey) -> Self {
            ControllerPoolKey {
                currency0: fetcher.currency0,
                currency1: fetcher.currency1,
                fee: fetcher.fee,
                tickSpacing: fetcher.tickSpacing,
                hooks: fetcher.hooks,
            }
        }
    }

    impl From<ControllerPoolKey> for FetcherPoolKey {
        fn from(manager: ControllerPoolKey) -> Self {
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

    impl From<SwapPoolKey> for ControllerPoolKey {
        fn from(swap: SwapPoolKey) -> Self {
            ControllerPoolKey {
                currency0: swap.currency0,
                currency1: swap.currency1,
                fee: swap.fee,
                tickSpacing: swap.tickSpacing,
                hooks: swap.hooks,
            }
        }
    }

    impl From<ControllerPoolKey> for SwapPoolKey {
        fn from(manager: ControllerPoolKey) -> Self {
            SwapPoolKey {
                currency0: manager.currency0,
                currency1: manager.currency1,
                fee: manager.fee,
                tickSpacing: manager.tickSpacing,
                hooks: manager.hooks,
            }
        }
    }

    impl From<ModifyLiquidityPoolKey> for ManagerPoolKey {
        fn from(swap: ModifyLiquidityPoolKey) -> Self {
            ManagerPoolKey {
                currency0: swap.currency0,
                currency1: swap.currency1,
                fee: swap.fee,
                tickSpacing: swap.tickSpacing,
                hooks: swap.hooks,
            }
        }
    }

    impl From<ManagerPoolKey> for ModifyLiquidityPoolKey {
        fn from(manager: ManagerPoolKey) -> Self {
            ModifyLiquidityPoolKey {
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
    use alloy::{
        primitives::{FixedBytes, Signed, Uint, I256},
        providers::WalletProvider,
    };
    use async_trait::async_trait;
    use rug::{ops::Pow, Float};

    use super::*;
    use crate::{
        arena::{Arena, ArenaBuilder},
        config::Config,
        engine::{
            arbitrageur::{EmptyArbitrageur, FixedArbitrageur},
            inspector::EmptyInspector,
        },
        feed::OrnsteinUhlenbeck,
        strategy::Strategy,
        types::controller::ArenaController,
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
                    provider,
                )
                .await
                .unwrap();
        }
        async fn process(
            &self,
            provider: AnvilProvider,
            signal: Signal,
            _inspector: &mut Box<dyn Inspector<T>>,
            _engine: Engine,
        ) {
            let sqrt_price_x96 =
                Float::with_val(53, Float::parse(signal.sqrt_price_x96.to_string()).unwrap());
            let q96 = Float::with_val(53, 2).pow(96);
            let price = Float::with_val(53, sqrt_price_x96 / q96).pow(2);

            println!("price: {}", price);
        }
    }

    #[tokio::test]
    async fn test_arena() {
        let builder: ArenaBuilder<_> = ArenaBuilder::new();

        let mut arena: Arena<_> = builder
            .with_strategy(Box::new(StrategyMock))
            .with_feed(Box::new(OrnsteinUhlenbeck::new(1.0, 0.1, 1.0, 0.1, 0.1)))
            .with_inspector(Box::new(EmptyInspector {}))
            .with_arbitrageur(Box::new(FixedArbitrageur::default()))
            .build();

        arena
            .run(Config::new(Uint::from(5000), 10000))
            .await
            .unwrap();
    }
}
