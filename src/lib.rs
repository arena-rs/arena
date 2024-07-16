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
        poolmanager::{PoolManager, PoolManager::PoolKey},
    },
    deployer::{DeploymentRequest, DeploymentResponse},
    pool_admin::PoolParams,
    price_changer::PriceUpdate,
    types::process::{OrnsteinUhlenbeck, StochasticProcess},
};

pub mod arbitrageur;
pub mod bindings;
pub mod deployer;
pub mod pool_admin;
pub mod price_changer;
pub mod types;

#[cfg(test)]
mod tests {
    use octane::{agent::Agent, world::World};

    use super::*;
    use crate::deployer::Deployer;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MockDeployer {
        #[serde(skip)]
        pub messager: Option<Messager>,

        #[serde(skip)]
        pub client: Option<Arc<AnvilProvider>>,
    }

    #[async_trait::async_trait]
    impl Behavior<Message> for MockDeployer {
        async fn startup(
            &mut self,
            client: Arc<AnvilProvider>,
            messager: Messager,
        ) -> Result<Option<EventStream<Message>>> {
            messager
                .send(
                    To::Agent("deployer".to_string()),
                    DeploymentRequest::Token {
                        name: String::from("TEST"),
                        symbol: String::from("TST"),
                        decimals: 18,
                    },
                )
                .await?;

            self.client = Some(client.clone());
            self.messager = Some(messager.clone());

            Ok(Some(messager.clone().stream().unwrap()))
        }

        async fn process(&mut self, event: Message) -> Result<ControlFlow> {
            let query: DeploymentResponse = match serde_json::from_str(&event.data) {
                Ok(query) => query,
                Err(_) => {
                    eprintln!("Failed to deserialize the event data into a DeploymentResponse");
                    return Ok(ControlFlow::Continue);
                }
            };

            match query {
                DeploymentResponse::Token(address) => {
                    let tok = ArenaToken::new(address, self.client.clone().unwrap());

                    assert_eq!(tok.name().call().await.unwrap()._0, "TEST");
                    assert_eq!(tok.symbol().call().await.unwrap()._0, "TST");
                    assert_eq!(tok.decimals().call().await.unwrap()._0, 18);
                }
                _ => {}
            }

            Ok(ControlFlow::Continue)
        }
    }

    #[tokio::test]
    async fn test_deployer() {
        env_logger::init();

        let deployer = Agent::builder("deployer").with_behavior(Deployer {
            messager: None,
            client: None,
        });
        let mock_deployer = Agent::builder("mock_deployer").with_behavior(MockDeployer {
            client: None,
            messager: None,
        });

        let mut world = World::new("id");

        world.add_agent(mock_deployer);
        world.add_agent(deployer);

        let _ = world.run().await;
    }
}
