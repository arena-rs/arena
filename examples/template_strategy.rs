use alloy::primitives::{Address, Bytes, Signed, Uint, I256};
use arena_core::{
    arena::{Arena, ArenaBuilder},
    config::Config,
    engine::{
        arbitrageur::FixedArbitrageur,
        inspector::{EmptyInspector, Inspector},
        Engine,
    },
    feed::OrnsteinUhlenbeck,
    strategy::Strategy,
    AnvilProvider, Signal,
};
use async_trait::async_trait;

struct TemplateStrategy;

#[async_trait]
impl<T> Strategy<T> for TemplateStrategy {
    async fn init(
        &self,
        provider: AnvilProvider,
        _signal: Signal,
        _inspector: &mut Box<dyn Inspector<T>>,
        engine: Engine,
    ) {
        // provide a fixed amount of liquidity upon runtime initialization to the pool across the full tick range.
        engine
            .modify_liquidity(
                I256::try_from(10000000).unwrap(),
                Signed::try_from(-887272).unwrap(),
                Signed::try_from(887272).unwrap(),
                Bytes::new(),
                provider,
            )
            .await
            .unwrap();
    }
    async fn process(
        &self,
        _provider: AnvilProvider,
        _signal: Signal,
        _inspector: &mut Box<dyn Inspector<T>>,
        _engine: Engine,
    ) {
    }
}

#[tokio::main]
async fn main() {
    let builder: ArenaBuilder<_> = ArenaBuilder::new();

    let mut arena: Arena<_> = builder
        .with_strategy(Box::new(TemplateStrategy))
        .with_feed(Box::new(OrnsteinUhlenbeck::new(1.0, 0.1, 1.0, 0.1, 0.1)))
        .with_inspector(Box::new(EmptyInspector {}))
        .with_arbitrageur(Box::new(FixedArbitrageur {
            depth: Signed::try_from(10000).unwrap(),
        }))
        .build();

    arena
        .run(Config::new(
            // timesteps to run for
            10,
            // manager fee
            Uint::from(0),
            // pool tick spacing
            Signed::try_from(2).unwrap(),
            // hook data
            Bytes::new(),
            // sqrtpricex96
            Uint::from(79228162514264337593543950336_u128),
            // pool fee
            Uint::from(0),
            // initial price
            Uint::from(1),
            // hook contract
            Address::ZERO,
        ))
        .await
        .unwrap();
}
