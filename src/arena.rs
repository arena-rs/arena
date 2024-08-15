use std::collections::HashMap;

use alloy::{primitives::U256, providers::ProviderBuilder, signers::local::PrivateKeySigner};

use super::*;
use crate::{
    config::Config,
    feed::Feed,
    inspector::Inspector,
    strategy::Strategy,
    types::{ArenaToken, LiquidExchange, PoolManager, PoolManager::PoolKey},
};

/// Represents an [`Arena`] that can be used to run a simulation and execute strategies.
pub struct Arena<V> {
    /// The underlying Anvil execution environment.
    pub env: AnvilInstance,

    /// The strategies that are to be run in the simulation.
    pub strategies: Vec<Box<dyn Strategy<V>>>,

    /// The pool that the strategies are to be run against, and the arbitrageur to peg.
    pub pool: PoolKey,

    /// The feed that provides the current, theoretical value of the pool.
    pub feed: Box<dyn Feed>,

    /// The inspector that is used to evaluate the performance of the strategies.
    pub inspector: Box<dyn Inspector<V>>,

    providers: HashMap<usize, AnvilProvider>,
}

impl<V> Arena<V> {
    /// Run all strategies in the simulation with a given configuration.
    pub async fn run(&mut self, config: Config) {
        let admin_provider = self.providers[&0].clone();

        let pool_manager = PoolManager::deploy(admin_provider.clone(), U256::from(0))
            .await
            .unwrap();

        let currency_0 = ArenaToken::deploy(
            admin_provider.clone(),
            String::from("Currency 0"),
            String::from("C0"),
            18,
        )
        .await
        .unwrap();

        let currency_1 = ArenaToken::deploy(
            admin_provider.clone(),
            String::from("Currency 1"),
            String::from("C1"),
            18,
        )
        .await
        .unwrap();

        let liquid_exchange = LiquidExchange::deploy(
            admin_provider.clone(),
            *currency_0.address(),
            *currency_1.address(),
            U256::from(1),
        )
        .await
        .unwrap();

        if *currency_1.address() < *currency_0.address() {
            self.pool.currency0 = *currency_1.address();
            self.pool.currency1 = *currency_0.address();
        }

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
            strategy.init(
                self.providers[&(idx + 1)].clone(),
                Signal::new(
                    *pool_manager.address(),
                    self.pool.clone(),
                    self.feed.current_value(),
                    None,
                ),
                &mut self.inspector,
            );
        }

        for step in 0..config.steps {
            liquid_exchange
                .setPrice(
                    alloy::primitives::utils::parse_ether(&self.feed.step().to_string())
                        .unwrap(),
                )
                .send()
                .await
                .unwrap()
                .watch()
                .await
                .unwrap();
                
            for (idx, strategy) in self.strategies.iter_mut().enumerate() {
                strategy.process(
                    self.providers[&(idx + 1)].clone(),
                    Signal::new(
                        *pool_manager.address(),
                        self.pool.clone(),
                        self.feed.current_value(),
                        Some(step),
                    ),
                    &mut self.inspector,
                );
            }

            self.feed.step();
        }
    }
}

/// A builder for an [`Arena`] that can be used to configure the simulation.
pub struct ArenaBuilder<V> {
    /// [`Arena::env`]
    pub env: AnvilInstance,

    /// [`Arena::strategies`]
    pub strategies: Vec<Box<dyn Strategy<V>>>,

    /// [`Arena::pool`]
    pub pool: PoolKey,

    /// [`Arena::feed`]
    pub feed: Option<Box<dyn Feed>>,

    /// [`Arena::inspector`]
    pub inspector: Option<Box<dyn Inspector<V>>>,

    providers: Option<HashMap<usize, AnvilProvider>>,
}

impl<V> Default for ArenaBuilder<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> ArenaBuilder<V> {
    /// Public constructor function for a new [`ArenaBuilder`].
    pub fn new() -> Self {
        ArenaBuilder {
            env: Anvil::default().spawn(),
            strategies: Vec::new(),
            pool: PoolKey::default(),
            feed: None,
            providers: None,
            inspector: None,
        }
    }

    /// Add a strategy to the simulation.
    pub fn with_strategy(mut self, strategy: Box<dyn Strategy<V>>) -> Self {
        self.strategies.push(strategy);
        self
    }

    /// Set the pool fee.
    pub fn with_fee(mut self, fee: u32) -> Self {
        self.pool.fee = fee;
        self
    }

    /// Set the pool tick spacing.
    pub fn with_tick_spacing(mut self, tick_spacing: i32) -> Self {
        self.pool.tickSpacing = tick_spacing;
        self
    }

    /// Set the pool hooks.
    pub fn with_hooks(mut self, hooks: Address) -> Self {
        self.pool.hooks = hooks;
        self
    }

    /// Set the feed that provides the current, theoretical value of the pool.
    pub fn with_feed(mut self, feed: Box<dyn Feed>) -> Self {
        self.feed = Some(feed);
        self
    }

    /// Set the inspector that is used to evaluate the performance of the strategies.
    pub fn with_inspector(mut self, inspector: Box<dyn Inspector<V>>) -> Self {
        self.inspector = Some(inspector);
        self
    }

    /// Build the [`Arena`] with the given configuration.
    pub fn build(self) -> Arena<V> {
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
            pool: self.pool,
            feed: self.feed.unwrap(),
            inspector: self.inspector.unwrap(),
            providers,
        }
    }
}
