use alloy::primitives::U256;

/// Configuration for the simulation.
pub struct Config {
    /// Number of steps to run the simulation for.
    pub steps: usize,

    /// Pool manager fee.
    pub fee: U256,
}

impl Config {
    /// Public constructor function for a new [`Config`].
    pub fn new(fee: U256, steps: usize) -> Self {
        Config { steps, fee }
    }
}
