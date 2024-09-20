use async_trait::async_trait;

use super::*;
use crate::{
    types::{controller::ArenaController, swap::PoolSwapTest},
    AnvilProvider, Signal,
};

/// Generic trait allowing user defined arbitrage strategies.
#[async_trait]
pub trait Arbitrageur {
    /// Initialize arbitrageur agent.
    async fn init(&mut self, signal: &Signal, provider: AnvilProvider);

    /// Perform an arbitrage based on a [`Signal`].
    async fn arbitrage(&mut self, signal: &Signal, provider: AnvilProvider);
}

/// Default implementation of an [`Arbitrageur`] that uses the closed-form optimal swap amount to determine the optimal arbitrage.
#[derive(Default)]
pub struct FixedArbitrageur {
    swapper: Option<Address>,
}

#[async_trait]
impl Arbitrageur for FixedArbitrageur {
    async fn init(&mut self, signal: &Signal, provider: AnvilProvider) {
        let swapper = PoolSwapTest::deploy(provider.clone(), signal.manager)
            .await
            .unwrap();

        self.swapper = Some(*swapper.address());
    }

    async fn arbitrage(&mut self, signal: &Signal, provider: AnvilProvider) {
        let controller = ArenaController::new(signal.controller, provider.clone());

        controller
            .equalizePrice(Signed::try_from(100000).unwrap())
            .nonce(
                provider
                    .clone()
                    .get_transaction_count(provider.clone().default_signer_address())
                    .await
                    .unwrap(),
            )
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();

        println!("current: {}", signal.current_value);
    }
}

/// No-op implementation of an [`Arbitrageur`] for custom usecases.
pub struct EmptyArbitrageur;

#[async_trait]
impl Arbitrageur for EmptyArbitrageur {
    async fn init(&mut self, _signal: &Signal, _provider: AnvilProvider) {}
    async fn arbitrage(&mut self, _signal: &Signal, _provider: AnvilProvider) {}
}
