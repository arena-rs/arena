use std::sync::Arc;

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
        liquidexchange::LiquidExchange,
        poolmanager::{
            PoolManager,
            PoolManager::{ModifyLiquidityParams, PoolKey},
        },
    },
    deployer::{DeploymentRequest, DeploymentResponse},
    pool_admin::PoolParams,
    price_changer::PriceUpdate,
    types::process::{OrnsteinUhlenbeck, StochasticProcess},
};
use crate::price_changer::PriceChanger;

pub mod arbitrageur;
pub mod bindings;
pub mod deployer;
pub mod liquidity_admin;
pub mod pool_admin;
pub mod price_changer;
pub mod types;


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
                    break;
                }
            }

            messager
                .send(
                    To::Agent("deployer".to_string()),
                    DeploymentRequest::LiquidExchange {
                        token_0: self.tokens[0],
                        token_1: self.tokens[1],
                        initial_price: 1.0,
                    },
                )
                .await?;

            println!("Tokens: {:?}", self.tokens);

            for i in 0..100 {
                messager
                    .send(
                        To::Agent("pricechanger".to_string()),
                        PriceUpdate,
                    )
                    .await?;
            }

            self.client = Some(client.clone());
            self.messager = Some(messager.clone());

            Ok(Some(messager.clone().stream().unwrap()))
        }
    }

    #[tokio::test]
    async fn asdfasdf() {
        env_logger::init();

        let token_deployer = Agent::builder("tdeployer").with_behavior(TokenDeployer {
            messager: None,
            client: None,
        });

        let deployer = Agent::builder("deployer").with_behavior(Deployer {
            messager: None,
            client: None,
        });

        let mock_deployer = Agent::builder("mock_deployer").with_behavior(MockOrchestrator {
            client: None,
            messager: None,
            tokens: vec![],
        });

        let changer = Agent::builder("pricechanger").with_behavior(PriceChanger::new(OrnsteinUhlenbeck::new(0.0, 0.1, 0.1, 0.1)));

        let mut world = World::new("id");

        world.add_agent(mock_deployer);
        world.add_agent(deployer);
        world.add_agent(token_deployer);
        world.add_agent(changer);

        let _ = world.run().await;
    }
}