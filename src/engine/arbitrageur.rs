use std::str::FromStr;

use alloy::primitives::{Address, Bytes, Signed};
use async_trait::async_trait;
use rug::{ops::Pow, Float};

use crate::{
    types::{
        fetcher::Fetcher,
        swap::{
            PoolSwapTest,
            PoolSwapTest::TestSettings,
            IPoolManager::SwapParams
        },
    },
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
pub struct DefaultArbitrageur {
    swapper: Option<Address>,
}

#[async_trait]
impl Arbitrageur for DefaultArbitrageur {
    async fn init(&mut self, signal: &Signal, provider: AnvilProvider) {
        let swapper = PoolSwapTest::deploy(provider.clone(), signal.manager)
            .await
            .unwrap();

        self.swapper = Some(*swapper.address());
    }

    async fn arbitrage(&mut self, signal: &Signal, provider: AnvilProvider) {
        // arbitrageur is always initialized before event loop starts, so unwrap should never fail.
        let swapper = PoolSwapTest::new(self.swapper.unwrap(), provider.clone());

        let base = Float::with_val(53, 1.0001);
        let price = Float::with_val(53, signal.current_value);

        let target_tick = price.log10() / base.log10();
        let current_tick = Float::with_val(53, signal.tick.as_i64());

        let (start, end) = if current_tick < target_tick {
            (current_tick.clone(), target_tick.clone())
        } else {
            (target_tick.clone(), current_tick.clone())
        };

        let (a, b) = self
            .get_tick_range_liquidity(
                signal,
                provider,
                start.to_i32_saturating().unwrap(),
                end.to_i32_saturating().unwrap(),
            )
            .await;

        let k = a.clone() * b.clone();

        // closed form optimal swap solution, ref: https://arxiv.org/pdf/1911.03380
        let fee: u64 = signal.pool.fee.to_string().parse().unwrap();
        let optimal_swap =
            Float::with_val(53, 0).max(&(a.clone() - (k / (fee * (a / b)))));

        let zero_for_one = current_tick > target_tick;

        let swap_params = SwapParams {
            amountSpecified: Signed::from_str(&optimal_swap.to_string()).unwrap(),
            zeroForOne: zero_for_one,
            sqrtPriceLimitX96: signal.sqrt_price_x96,
        };

        let test_settings = TestSettings {
            takeClaims: false,
            settleUsingBurn: false,
        };

        swapper
            .swap(
                signal.pool.clone().into(),
                swap_params,
                test_settings,
                Bytes::new(),
            )
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();
    }
}

impl DefaultArbitrageur {
    async fn get_tick_range_liquidity(
        &self,
        signal: &Signal,
        provider: AnvilProvider,
        start: i32,
        end: i32,
    ) -> (Float, Float) {
        let fetcher = Fetcher::new(signal.fetcher, provider.clone());

        let mut liquidity_a = Float::with_val(53, 0);
        let mut liquidity_b = Float::with_val(53, 0);

        for tick in start..end {
            let pool_id = fetcher
                .toId(signal.pool.clone().into())
                .call()
                .await
                .unwrap()
                .poolId;

            let tick_info = fetcher
                .getTickInfo(signal.manager, pool_id, Signed::from_str(&tick.to_string()).unwrap())
                .call()
                .await
                .unwrap();
            let sqrt_price = Float::with_val(53, Float::with_val(53, 1.0001).pow(tick / 2));

            let tick_liquidity = Float::with_val(53, tick_info.liquidityNet);

            liquidity_a += tick_liquidity.clone() / sqrt_price.clone();
            liquidity_b += tick_liquidity * sqrt_price;
        }

        (liquidity_a, liquidity_b)
    }
}

/// No-op implementation of an [`Arbitrageur`] for custom usecases.
pub struct EmptyArbitrageur;

#[async_trait]
impl Arbitrageur for EmptyArbitrageur {
    async fn init(&mut self, _signal: &Signal, _provider: AnvilProvider) {}
    async fn arbitrage(&mut self, _signal: &Signal, _provider: AnvilProvider) {}
}
