use std::{fmt, str::FromStr, sync::Arc};

use alloy::{
    primitives::{Address, Bytes, Uint, U256},
    sol,
};
use anyhow::Result;
use futures::stream::StreamExt;
use octane::{
    agent::Agent,
    machine::{Behavior, ControlFlow, EventStream},
    messenger::{Message, Messager, To},
    world::World,
    AnvilProvider,
};
use serde::{Deserialize, Serialize};
// use RustQuant::{
//     models::*,
//     stochastics::{process::Trajectories, *},
// };

use crate::{
    bindings::{
        arenatoken::ArenaToken,
        liquidexchange::LiquidExchange,
        poolmanager::{PoolManager, PoolManager::PoolKey},
    },
    deployer::{Deployer, DeploymentRequest, DeploymentResponse},
    pool_admin::PoolParams,
    // price_changer::PriceUpdate,
};

// pub mod arbitrageur;
pub mod bindings;
pub mod deployer;
pub mod pool_admin;
// pub mod price_changer;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MockDeployer {}

    #[async_trait::async_trait]
    impl Behavior<()> for MockDeployer {
        async fn startup(
            &mut self,
            client: Arc<AnvilProvider>,
            messager: Messager,
        ) -> Result<Option<EventStream<()>>> {
            messager
                .send(
                    To::Agent("deployer".to_string()),
                    DeploymentRequest::Token { name: String::from("TEST"), symbol: String::from("TST"), decimals: 18 },
                )
                .await?;

            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_deployer() {
        env_logger::init();

        let deployer = Agent::builder("deployer").with_behavior(Deployer {
            messager: None,
            client: None,
        });
        let mock_deployer = Agent::builder("mock_deployer").with_behavior(MockDeployer {});

        let mut world = World::new("id");

        world.add_agent(mock_deployer);

        world.add_agent(deployer);

        world.run().await;
    }
}
