use alloy::primitives::U256;

use super::*;

/// Configuration for the simulation.
pub struct Config {
    /// Number of steps to run the simulation for.
    pub steps: usize,

    /// Pool manager fee.
    pub manager_fee: U256,

    /// Pool tick spacing.
    pub tick_spacing: Signed<24, 1>,

    /// Pool hook data.
    pub hook_data: Bytes,

    /// Pool sqrt price x96.
    pub sqrt_price_x96: Uint<160, 3>,

    /// Pool fee.
    pub pool_fee: Uint<24, 1>,

    /// Initial price.
    pub initial_price: U256,

    /// Pool hooks.
    pub hooks: Address,
}

impl Config {
    /// Public constructor function for a new [`Config`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        steps: usize,
        manager_fee: U256,
        tick_spacing: Signed<24, 1>,
        hook_data: Bytes,
        sqrt_price_x96: Uint<160, 3>,
        pool_fee: Uint<24, 1>,
        initial_price: U256,
        hooks: Address,
    ) -> Self {
        Self {
            steps,
            manager_fee,
            tick_spacing,
            hook_data,
            sqrt_price_x96,
            pool_fee,
            initial_price,
            hooks,
        }
    }
}
