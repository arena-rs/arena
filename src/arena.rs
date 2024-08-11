use std::collections::HashMap;

use alloy::{providers::ProviderBuilder, signers::local::PrivateKeySigner};

use super::*;
use crate::{
    config::Config,
    feed::Feed,
    strategy::Strategy,
    types::{PoolManager, PoolManager::PoolKey},
};

pub struct Arena {
    pub env: AnvilInstance,
    pub strategies: Vec<Box<dyn Strategy>>,
    pub pool: PoolKey,
    pub feed: Box<dyn Feed>,

    providers: HashMap<usize, AnvilProvider>,
}

impl Arena {
    pub async fn run(&mut self, config: Config) {
        let admin_provider = self.providers[&0].clone();

        let pool_manager = PoolManager::deploy(admin_provider.clone(), U256::from(0))
            .await
            .unwrap();

        pool_manager
            .initialize(
                self.pool.clone(),
                U256::from(79228162514264337593543950336_u128),
                Bytes::default(),
            )
            .call()
            .await
            .unwrap();

        for (idx, strategy) in self.strategies.iter_mut().enumerate() {
            strategy.init(self.providers[&(idx + 1)].clone());
        }

        for step in 0..config.steps {
            for (idx, strategy) in self.strategies.iter_mut().enumerate() {
                strategy.process(self.providers[&(idx + 1)].clone());
            }

            self.feed.step();
        }
    }
}

pub struct ArenaBuilder {
    pub env: AnvilInstance,
    pub strategies: Vec<Box<dyn Strategy>>,
    pub pool: Option<PoolKey>,
    pub feed: Option<Box<dyn Feed>>,

    providers: Option<HashMap<usize, AnvilProvider>>,
}

impl ArenaBuilder {
    pub fn new() -> Self {
        ArenaBuilder {
            env: Anvil::default().spawn(),
            strategies: Vec::new(),
            pool: None,
            feed: None,
            providers: None,
        }
    }

    pub fn with_strategy(mut self, strategy: Box<dyn Strategy>) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub fn with_pool(mut self, pool: PoolKey) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn with_feed(mut self, feed: Box<dyn Feed>) -> Self {
        self.feed = Some(feed);
        self
    }

    pub fn build(self) -> Arena {
        let mut providers = HashMap::new();

        for i in 0..9 {
            let signer: PrivateKeySigner = self.env.keys()[i].clone().into();
            let wallet = EthereumWallet::from(signer);

            let rpc_url = self.env.endpoint().parse().unwrap();

            let provider = ProviderBuilder::new()
                .with_recommended_fillers()
                .wallet(wallet)
                .on_http(rpc_url);

            providers.insert(i, provider);
        }

        Arena {
            env: self.env,
            strategies: self.strategies,
            pool: self.pool.unwrap(),
            feed: self.feed.unwrap(),
            providers,
        }
    }
}
