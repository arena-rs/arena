use crate::{AnvilProvider, Signal};

/// Generic trait allowing user defined arbitrage strategies.
pub trait Arbitrageur {
    /// Perform an arbitrage based on a [`Signal`].
    fn arbitrage(&self, signal: &Signal, provider: AnvilProvider);
}

/// No-op implementation of an [`Arbitrageur`] for custom usecases.
pub struct EmptyArbitrageur;

impl Arbitrageur for EmptyArbitrageur {
    fn arbitrage(&self, _signal: &Signal, _provider: AnvilProvider) {}
}
