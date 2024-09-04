use std::collections::HashMap;

use alloy::{
    primitives::{Uint, U256},
    providers::{Provider, ProviderBuilder, WalletProvider},
    signers::local::PrivateKeySigner,
};

use super::*;
use crate::{
    config::Config,
    engine::{arbitrageur::Arbitrageur, inspector::Inspector},
    error::ArenaError,
    feed::Feed,
    strategy::Strategy,
    types::{
        fetcher::Fetcher,
        modify_liquidity::PoolModifyLiquidityTest,
        pool_manager::{PoolManager, PoolManager::PoolKey},
        ArenaToken, LiquidExchange,
    },
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

    /// The arbitrageur that is used to peg the pool.
    pub arbitrageur: Box<dyn Arbitrageur>,

    providers: HashMap<usize, AnvilProvider>,
}

#[allow(clippy::redundant_closure)]
impl<V> Arena<V> {
    /// Run all strategies in the simulation with a given configuration.
    pub async fn run(&mut self, config: Config) -> Result<(), ArenaError> {
        let admin_provider = self.providers[&0].clone();

        let pool_manager = PoolManager::deploy(admin_provider.clone(), U256::from(0))
            .await
            .map_err(ArenaError::ContractError)?;

        let fetcher = Fetcher::deploy(admin_provider.clone())
            .await
            .map_err(ArenaError::ContractError)?;

        let lp_manager =
            PoolModifyLiquidityTest::deploy(admin_provider.clone(), *pool_manager.address())
                .await
                .map_err(ArenaError::ContractError)?;

        let engine = Engine {
            pool: self.pool.clone().into(),
            provider: admin_provider.clone(),
            liquidity_manager: *lp_manager.address(),
        };

        let currency_0 = ArenaToken::deploy(
            admin_provider.clone(),
            String::from("Currency 0"),
            String::from("C0"),
            18,
        )
        .await
        .map_err(ArenaError::ContractError)?;

        let currency_1 = ArenaToken::deploy(
            admin_provider.clone(),
            String::from("Currency 1"),
            String::from("C1"),
            18,
        )
        .await
        .map_err(ArenaError::ContractError)?;

        let liquid_exchange = LiquidExchange::deploy(
            admin_provider.clone(),
            *currency_0.address(),
            *currency_1.address(),
            U256::from(1),
        )
        .await
        .map_err(ArenaError::ContractError)?;

        if *currency_1.address() > *currency_0.address() {
            (self.pool.currency0, self.pool.currency1) =
                (*currency_0.address(), *currency_1.address());
        }

        pool_manager
            .initialize(
                self.pool.clone(),
                Uint::from(79228162514264337593543950336_u128),
                Bytes::default(),
            )
            .nonce(
                admin_provider
                    .get_transaction_count(admin_provider.default_signer_address())
                    .await
                    .unwrap(),
            )
            .send()
            .await
            .map_err(ArenaError::ContractError)?
            .watch()
            .await
            .map_err(|e| ArenaError::PendingTransactionError(e))?;

        let mut signal = Signal::default();

        for (idx, strategy) in self.strategies.iter_mut().enumerate() {
            let id = fetcher
                .toId(self.pool.clone().into())
                .call()
                .await
                .map_err(ArenaError::ContractError)?;

            let slot = fetcher
                .getSlot0(*pool_manager.address(), id.poolId)
                .call()
                .await
                .map_err(ArenaError::ContractError)?;

            signal = Signal::new(
                *pool_manager.address(),
                *fetcher.address(),
                self.pool.clone(),
                self.feed.current_value(),
                None,
                slot.tick,
                slot.sqrtPriceX96,
            );

            strategy.init(
                self.providers[&(idx + 1)].clone(),
                signal.clone(),
                &mut self.inspector,
                engine.clone(),
            );
        }

        self.arbitrageur.init(&signal, admin_provider.clone()).await;

        for step in 0..config.steps {
            let id = fetcher
                .toId(self.pool.clone().into())
                .call()
                .await
                .map_err(ArenaError::ContractError)?;

            let slot = fetcher
                .getSlot0(*pool_manager.address(), id.poolId)
                .call()
                .await
                .map_err(ArenaError::ContractError)?;

            let signal = Signal::new(
                *pool_manager.address(),
                *fetcher.address(),
                self.pool.clone(),
                self.feed.current_value(),
                Some(step),
                slot.tick,
                slot.sqrtPriceX96,
            );

            liquid_exchange
                .setPrice(
                    alloy::primitives::utils::parse_ether(&self.feed.step().to_string())
                        .map_err(ArenaError::ConversionError)?,
                )
                .nonce(
                    admin_provider
                        .get_transaction_count(admin_provider.default_signer_address())
                        .await
                        .unwrap(),
                )
                .send()
                .await
                .map_err(ArenaError::ContractError)?
                .watch()
                .await
                .map_err(|e| ArenaError::PendingTransactionError(e))?;

            self.arbitrageur
                .arbitrage(&signal, admin_provider.clone())
                .await;

            for (idx, strategy) in self.strategies.iter_mut().enumerate() {
                strategy.process(
                    self.providers[&(idx + 1)].clone(),
                    signal.clone(),
                    &mut self.inspector,
                    engine.clone(),
                );
            }

            self.feed.step();
        }

        Ok(())
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

    /// [`Arena::arbitrageur`]
    pub arbitrageur: Option<Box<dyn Arbitrageur>>,
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
            inspector: None,
            arbitrageur: None,
        }
    }

    /// Add a strategy to the simulation.
    pub fn with_strategy(mut self, strategy: Box<dyn Strategy<V>>) -> Self {
        self.strategies.push(strategy);
        self
    }

    /// Set the pool fee.
    pub fn with_fee(mut self, fee: Uint<24, 1>) -> Self {
        let fee: Uint<24, 1> = fee;
        self.pool.fee = fee;
        self
    }

    /// Set the pool tick spacing.
    pub fn with_tick_spacing(mut self, tick_spacing: Signed<24, 1>) -> Self {
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

    /// Set the inspector that is used to evaluate the performance of the strategies.
    pub fn with_arbitrageur(mut self, arbitrageur: Box<dyn Arbitrageur>) -> Self {
        self.arbitrageur = Some(arbitrageur);
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
            arbitrageur: self.arbitrageur.unwrap(),
            providers,
        }
    }
}
