use async_trait::async_trait;

use super::*;

/// Represents a strategy that can be run in an [`Arena`].
#[async_trait]
pub trait Strategy<V> {
    /// Initialization function for ths strategy to be run upon simulation startup.
    async fn init(
        &self,
        provider: AnvilProvider,
        signal: Signal,
        inspector: &mut Box<dyn Inspector<V>>,
        engine: Engine,
    );

    /// Processing function for the strategy to be run each simulation step.
    async fn process(
        &self,
        provider: AnvilProvider,
        signal: Signal,
        inspector: &mut Box<dyn Inspector<V>>,
        engine: Engine,
    );
}
