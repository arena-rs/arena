use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use alloy::primitives::{Address, Bytes, Uint, U256};
use anyhow::Result;
use futures::stream::StreamExt;
use octane::{
    machine::{Behavior, ControlFlow, EventStream},
    messenger::{Message, Messager, To},
    AnvilProvider,
};
use serde::{Deserialize, Serialize};

use crate::{
    bindings::{
        arenatoken::ArenaToken,
        fetcher::{Fetcher, Fetcher::PoolKey as FetcherPoolKey},
        liquidexchange::LiquidExchange,
        poolmanager::{
            PoolManager,
            PoolManager::{ModifyLiquidityParams, PoolKey},
        },
    },
    deployer::{DeploymentResponse, PoolParams},
    price_changer::Signal,
    types::process::StochasticProcess,
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

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TokenDeployer {
        #[serde(skip)]
        pub messager: Option<Messager>,

        #[serde(skip)]
        pub client: Option<Arc<AnvilProvider>>,
    }

    #[async_trait::async_trait]
    impl Behavior<Message> for TokenDeployer {
        async fn startup(
            &mut self,
            client: Arc<AnvilProvider>,
            messager: Messager,
        ) -> Result<Option<EventStream<Message>>> {
            self.client = Some(client.clone());
            self.messager = Some(messager.clone());

            messager
                .send(
                    To::Agent("deployer".to_string()),
                    DeploymentRequest::Token {
                        name: String::from("TEST0"),
                        symbol: String::from("TST0"),
                        decimals: 18,
                        initial_mint: 1000000,
                        receiver: client.clone().default_signer_address(),
                    },
                )
                .await?;

            messager
                .send(
                    To::Agent("deployer".to_string()),
                    DeploymentRequest::Token {
                        name: String::from("TEST1"),
                        symbol: String::from("TST1"),
                        decimals: 18,
                        initial_mint: 1000000,
                        receiver: client.clone().default_signer_address(),
                    },
                )
                .await?;

            Ok(Some(messager.clone().stream().unwrap()))
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MockOrchestrator {
        #[serde(skip)]
        pub messager: Option<Messager>,

        #[serde(skip)]
        pub client: Option<Arc<AnvilProvider>>,

        pub tokens: Vec<Address>,
    }

    #[async_trait::async_trait]
    impl Behavior<Message> for MockOrchestrator {
        async fn startup(
            &mut self,
            client: Arc<AnvilProvider>,
            messager: Messager,
        ) -> Result<Option<EventStream<Message>>> {
            let mut stream = messager.clone().stream().unwrap();

            while let Some(event) = stream.next().await {
                let query: DeploymentResponse = match serde_json::from_str(&event.data) {
                    Ok(query) => query,
                    Err(_) => {
                        eprintln!("Failed to deserialize the event data into a DeploymentResponse");
                        continue;
                    }
                };

                if let DeploymentResponse::Token(address) = query {
                    self.tokens.push(address);
                }

                if self.tokens.len() == 2 {
                    println!("Tokens: {:?}", self.tokens);
                    break;
                }
            }

            messager
                .send(
                    To::Agent("deployer".to_string()),
                    DeploymentRequest::LiquidExchange {
                        token_0: self.tokens[0].clone(),
                        token_1: self.tokens[1].clone(),
                        initial_price: 1.0,
                    },
                )
                .await?;

            let key = PoolKey {
                currency0: self.tokens[0],
                currency1: self.tokens[1],
                fee: 3000,
                tickSpacing: 60,
                hooks: Address::default(),
            };

            messager
                .send(
                    To::All,
                    DeploymentRequest::Pool(PoolParams {
                        key: key.clone(),
                        sqrt_price_x96: U256::from_str("79228162514264337593543950336").unwrap(),
                        hook_data: Bytes::default(),
                    }),
                )
                .await?;

            messager
                .send(
                    To::All,
                    AllocationRequest {
                        pool: key.clone(),
                        modification: ModifyLiquidityParams {
                            tickLower: -10,
                            tickUpper: 10,
                            liquidityDelta: Signed::from_str("1000").unwrap(),
                            salt: <FixedBytes<32> as SolType>::abi_decode(&[0u8; 32], true)
                                .unwrap(),
                        },
                    },
                )
                .await?;

            use tokio::time::{sleep, Duration};

            while let Some(event) = stream.next().await {
                let query: DeploymentResponse = match serde_json::from_str(&event.data) {
                    Ok(query) => query,
                    Err(_) => {
                        // eprintln!("Failed to deserialize the event datfa into a DeploymentResponse");
                        continue;
                    }
                };

                if let DeploymentResponse::LiquidExchange(_) = query {
                    sleep(Duration::from_millis(3000)).await;

                    for _ in 0..100 {
                        messager.send(To::All, PriceUpdate).await?;
                    }
                }
            }

            self.client = Some(client.clone());
            self.messager = Some(messager.clone());

            Ok(Some(messager.clone().stream().unwrap()))
        }
    }

    #[tokio::test]
    async fn test_price_changer() {
        env_logger::init();

        let token_deployer = Agent::builder("tdeployer")
            .with_behavior(TokenDeployer {
                messager: None,
                client: None,
            })
            .with_behavior(LiquidityAdmin::default());

        let deployer = Agent::builder("deployer").with_behavior(Deployer::default());

        let mock_deployer = Agent::builder("mock_deployer").with_behavior(MockOrchestrator {
            client: None,
            messager: None,
            tokens: vec![],
        });

        let changer =
            Agent::builder("pricechanger").with_behavior(PriceChanger::<OrnsteinUhlenbeck>::new(
                OrnsteinUhlenbeck::new(1.0, 0.15, 0.0, 0.3, 1.0 / 252.0),
            ));

        let arbitrageur = Agent::builder("arbitrageur").with_behavior(Arbitrageur::default());

        let mut world = World::new("id");

        world.add_agent(changer);
        world.add_agent(mock_deployer);
        world.add_agent(deployer);
        world.add_agent(token_deployer);
        world.add_agent(arbitrageur);

        let _ = world.run().await;
    }
}
