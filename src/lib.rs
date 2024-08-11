pub mod arena;
pub mod config;
pub mod feed;
pub mod strategy;
pub mod types;
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

use crate::{
    arena::ArenaBuilder, config::Config, feed::OrnsteinUhlenbeck, strategy::Strategy,
    types::PoolManager::PoolKey,
};

pub type AnvilProvider = FillProvider<
    JoinFill<
        JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

#[cfg(test)]
mod tests {
    use super::*;

    struct StrategyMock;

    impl Strategy for StrategyMock {
        fn init(&self, _provider: AnvilProvider) {}
        fn process(&self, _provider: AnvilProvider) {}
    }

    #[tokio::test]
    async fn test_arena() {
        let builder = ArenaBuilder::new();

        let mut arena = builder
            .with_strategy(Box::new(StrategyMock {}))
            .with_pool(PoolKey {
                currency0: Address::default(),
                currency1: Address::repeat_byte(1),
                fee: 4000,
                tickSpacing: 2,
                hooks: Address::default(),
            })
            .with_feed(Box::new(OrnsteinUhlenbeck::new(0.1, 0.1, 0.1, 0.1, 0.1)))
            .build();

        arena.run(Config::new(0)).await;
    }
}
