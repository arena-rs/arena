use super::*;
use crate::{config::Config, feed::Feed, strategy::Strategy};
use crate::types::PoolKey;

pub struct Arena {
    pub env: AnvilInstance,
    pub strategies: Vec<Box<dyn Strategy>>,
    pub pool: PoolKey,
    pub feed: Box<dyn Feed>,
}

impl Arena {
    pub fn run(&mut self, config: Config) {
        for i in self.strategies.iter_mut() {
            // i.init();
        }

        for step in 0..config.steps {
            for i in self.strategies.iter_mut() {
                // i.process();
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
}

impl ArenaBuilder {
    pub fn new() -> Self {
        ArenaBuilder {
            env: Anvil::default().spawn(),
            strategies: Vec::new(),
            pool: None,
            feed: None,
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
        Arena {
            env: self.env,
            strategies: self.strategies,
            pool: self.pool.unwrap(),
            feed: self.feed.unwrap(),
        }
    }
}
