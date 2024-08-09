use std::{cmp::Ordering, fmt::Debug, str::FromStr, sync::Arc};

use alloy::{
    primitives::{Address, Bytes, I256, Uint, U256},
    providers::WalletProvider,
};
use alloy_sol_types::{sol_data::FixedBytes, SolType};
use anyhow::Result;
use futures::stream::StreamExt;
use octane::{
    machine::{Behavior, ControlFlow, EventStream},
    messenger::{Message, Messager, To},
    AnvilProvider,
};
use serde::{Deserialize, Serialize};
use alloy::providers::Provider;
use crate::{
    arbitrageur::Arbitrageur,
    bindings::{
        arenatoken::ArenaToken,
        fetcher::{Fetcher, Fetcher::PoolKey as FetcherPoolKey},
        liquidexchange::LiquidExchange,
        liquidityprovider::{LiquidityProvider, LiquidityProvider::PoolKey as LPoolKey, LiquidityProvider::ModifyLiquidityParams, LiquidityProvider::Currency},
        poolmanager::{
            PoolManager,
            PoolManager::{ModifyLiquidityParams as X, PoolKey},
        },
    },
    deployer::{DeploymentRequest, DeploymentResponse, PoolParams},
    liquidity_admin::{AllocationRequest, LiquidityAdmin},
    price_changer::{PriceChanger, PriceUpdate, Signal},
    types::process::{OrnsteinUhlenbeck, StochasticProcess},
};

pub mod arbitrageur;
pub mod bindings;
pub mod deployer;
pub mod liquidity_admin;
pub mod price_changer;
pub mod types;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Base {
    #[serde(skip)]
    pub messager: Option<Messager>,

    #[serde(skip)]
    pub client: Option<Arc<AnvilProvider>>,
}

#[cfg(test)]
mod tests {
    use octane::{agent::Agent, world::World};

    use super::*;
    use crate::deployer::Deployer;

    #[derive(Debug, Serialize, Deserialize, Default)]
    struct Harness;

    #[async_trait::async_trait]
    impl Behavior<Message> for Harness {
        async fn startup(
            &mut self,
            client: Arc<AnvilProvider>,
            messager: Messager,
        ) -> Result<Option<EventStream<Message>>> {
            let pool_manager = PoolManager::deploy(client.clone(), Uint::from(5000)).await.unwrap();

            // Deploy tokens
            let currency0 = ArenaToken::deploy(client.clone(), "ARENA0".to_string(), "ARENA0".to_string(), 18).await?;
            let currency1 = ArenaToken::deploy(client.clone(), "ARENA1".to_string(), "ARENA1".to_string(), 18).await?;
        
            // Mint tokens
            currency0.mint(Uint::from(2).pow(Uint::from(255))).send().await?.watch().await?;
            currency1.mint(Uint::from(2).pow(Uint::from(255))).send().await?.watch().await?;
        
            // Ensure the token addresses are ordered
            let (currency0, currency1) = if currency0.address() > currency1.address() {
                (currency1, currency0)
            } else {
                (currency0, currency1)
            };
        
            // Create PoolKey
            let key = PoolKey {
                currency0: *currency0.address(),
                currency1: *currency1.address(),
                fee: 2000,
                tickSpacing: 60,
                hooks: Address::default(),
            };

            let lp_key = LPoolKey {
                currency0: key.currency0,
                currency1: key.currency1,
                fee: key.fee,
                tickSpacing: key.tickSpacing,
                hooks: key.hooks,
            };
        
            // Initialize pool
            pool_manager.initialize(key.clone(), U256::from(79228162514264337593543950336_u128), Bytes::default()).send().await?.watch().await?;
        
            // Deploy LiquidityProvider
            let liquidity_provider = LiquidityProvider::deploy(client.clone(), *pool_manager.address()).await.unwrap();
        
            // Approve tokens for LiquidityProvider
            currency0.approve(*liquidity_provider.address(), Uint::MAX).send().await?.watch().await?;
            currency1.approve(*liquidity_provider.address(), Uint::MAX).send().await?.watch().await?;
        
            // Create ModifyLiquidityParams
            let modification = ModifyLiquidityParams {
                tickLower: -120,
                tickUpper: 120,
                liquidityDelta: I256::from_str("100000000000000000").unwrap(),
                salt: <FixedBytes<32> as SolType>::abi_decode(&[0u8; 32], true).unwrap(),
            };
        
            // Modify liquidity
            let tx = liquidity_provider.modifyLiquidity_1(lp_key, modification, Bytes::default());
            let send_result = tx.send().await;
        
            println!("send result: {:#?}", send_result);
            Ok(Some(messager.stream().unwrap()))
        }
    }

    #[tokio::test]
    async fn test_lp() {
        let harness = Agent::builder("harness").with_behavior(Harness::default());

        let mut world = World::new("id");

        world.add_agent(harness);

        let _ = world.run().await;
    }
}
