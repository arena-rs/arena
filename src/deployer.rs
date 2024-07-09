use super::*;

#[derive(Debug, Serialize, Deserialize)]
struct Deployer;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeploymentParams {
    pool_manager: Address,
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

        let currency_0 = ArenaToken::deploy(
            client.clone(),
            String::from("ARN0"),
            String::from("ARN0"),
            18,
        )
        .await
        .unwrap();

        let currency_1 = ArenaToken::deploy(
            client.clone(),
            String::from("ARN1"),
            String::from("ARN1"),
            18,
        )
        .await
        .unwrap();

        let key = PoolKey {
            currency0: *currency_0.address(),
            currency1: *currency_1.address(),
            fee: 10,
            tickSpacing: 2,
            hooks: Address::default(),
        };

        let tx = pool_manager.initialize(
            key,
            U256::from_str("42951287310").unwrap(),
            Bytes::default(),
        );

        let tx = tx.send().await?.watch().await?;

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
