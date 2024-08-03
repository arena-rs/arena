use std::{cmp::Ordering, fmt::Debug, str::FromStr, sync::Arc};

use alloy::{
    primitives::{Address, Bytes, Signed, Uint, U256},
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

use crate::{
    arbitrageur::Arbitrageur,
    bindings::{
        arenatoken::ArenaToken,
        fetcher::{Fetcher, Fetcher::PoolKey as FetcherPoolKey},
        liquidexchange::LiquidExchange,
        liquidityprovider::{LiquidityProvider, LiquidityProvider::PoolKey as LPoolKey},
        poolmanager::{
            PoolManager,
            PoolManager::{ModifyLiquidityParams, PoolKey},
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
            let mut tokens = Vec::new();
            let mut stream = messager.clone().stream().unwrap();

            messager.send(To::All, DeploymentRequest::Token {
                name: "Arena Token 0".to_string(),
                symbol: "ARENA0".to_string(),
                decimals: 18,
            }).await?;

            messager.send(To::All, DeploymentRequest::Token {
                name: "Arena Token 1".to_string(),
                symbol: "ARENA1".to_string(),
                decimals: 18,
            }).await?;

            while let Some(event) = stream.next().await {
                let query: DeploymentResponse = match serde_json::from_str(&event.data) {
                    Ok(query) => query,
                    Err(_) => continue,
                };

                if let DeploymentResponse::Token(address) = query {
                    tokens.push(address);
                }

                if tokens.len() == 2 {
                    break;
                }
            }

            if tokens[0] > tokens[1] {
                tokens.swap(0, 1);
            }

            let key = PoolKey {
                currency0: tokens[0],
                currency1: tokens[1],
                fee: 1000,
                tickSpacing: 60,
                hooks: Address::default()
            };

            messager.send(To::All, DeploymentRequest::Pool(PoolParams {
                key: key.clone(),
                sqrt_price_x96: U256::from(79228162514264337593543950336_u128),
                hook_data: Bytes::default(),
            })).await?;

            messager.send(To::All, AllocationRequest {
                pool: key,
                modification: ModifyLiquidityParams {
                    tickLower: -600,
                    tickUpper: 600,
                    liquidityDelta: Signed::from_str("1000").unwrap(),
                    salt: <FixedBytes<32> as SolType>::abi_decode(&[0u8; 32], true)
                        .unwrap(),
                },
                hook_data: Bytes::default(),
            }).await?;

            Ok(Some(stream))
        }
    }

    #[tokio::test]
    async fn test_lp() {
        env_logger::init();

        let harness = Agent::builder("harness").with_behavior(Harness::default());
        let deployer = Agent::builder("deployer")
            .with_behavior(Deployer::default());

        let liq_admin = Agent::builder("admin")
            .with_behavior(LiquidityAdmin::default());

        let mut world = World::new("id");

        world.add_agent(deployer);
        world.add_agent(liq_admin);
        world.add_agent(harness);

        let _ = world.run().await;
    }
}
