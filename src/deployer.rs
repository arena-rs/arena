use super::*;

#[derive(Debug, Serialize, Deserialize)]
struct Deployer;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeploymentParams {
    pub pool_manager: Address,
}

#[async_trait::async_trait]
impl Behavior<()> for Deployer {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<()>>> {
        let pool_manager = PoolManager::deploy(client.clone(), Uint::from(5000))
            .await
            .unwrap();

        messager
            .send(
                To::All,
                DeploymentParams {
                    pool_manager: *pool_manager.address(),
                },
            )
            .await?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_behaviour() {
        env_logger::init();

        let agent = Agent::builder("deployer").with_behavior(Deployer);

        let mut world = World::new("id");

        world.add_agent(agent);
        world.run().await;
    }
}
