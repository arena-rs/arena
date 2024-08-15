use crate::{AnvilProvider, Signal};

/// Generic trait allowing user defined arbitrage strategies.
pub trait Arbitrageur {
    /// Perform an arbitrage based on a [`Signal`].
    fn arbitrage(&self, signal: &Signal, provider: AnvilProvider);
}
