use std::collections::HashMap;

use alloy::{providers::ProviderBuilder, signers::local::PrivateKeySigner};

use super::*;
use crate::{config::Config, feed::Feed, strategy::Strategy, types::PoolKey};

pub struct Arena {
    pub env: AnvilInstance,
    pub strategies: Vec<Box<dyn Strategy>>,
    pub pool: PoolKey,
    pub feed: Box<dyn Feed>,

    providers: HashMap<usize, AnvilProvider>,
}

impl Arena {
    pub fn run(&mut self, config: Config) {
        for (idx, strategy) in self.strategies.iter_mut().enumerate() {
            strategy.init(self.providers[&idx].clone());
        }

        for step in 0..config.steps {
            for (idx, strategy) in self.strategies.iter_mut().enumerate() {
                strategy.process(self.providers[&idx].clone());
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

        for i in 0..10 {
            let signer: PrivateKeySigner = self.env.keys()[i].clone().into();
            let wallet = EthereumWallet::from(signer);

            let rpc_url = self.env.endpoint().parse().unwrap();

            let provider = ProviderBuilder::new()
                .with_recommended_fillers()
                .wallet(wallet)
                .on_http(rpc_url);

            providers.insert(i, provider).unwrap();
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
