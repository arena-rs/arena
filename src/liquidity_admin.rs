use super::*;

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct LiquidityAdmin {
    pub base: Base,

    pub deployment: Option<Address>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AllocationRequest {
    #[serde(skip)]
    pub pool: PoolKey,

    #[serde(skip)]
    pub modification: ModifyLiquidityParams,
}

#[async_trait::async_trait]
impl Behavior<Message> for LiquidityAdmin {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<Message>>> {
        self.base.client = Some(client.clone());
        self.base.messager = Some(messager.clone());

        let mut stream = messager.clone().stream().unwrap();

        while let Some(event) = stream.next().await {
            let query: DeploymentResponse = match serde_json::from_str(&event.data) {
                Ok(query) => query,
                Err(_) => continue,
            };

            if let DeploymentResponse::PoolManager(address) = query {
                self.deployment = Some(address);
                break;
            }
        }

        Ok(Some(messager.clone().stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let query: AllocationRequest = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => return Ok(ControlFlow::Continue),
        };

        ArenaToken::new(query.pool.currency0, self.base.client.clone().unwrap())
            .approve(self.deployment.unwrap(), Uint::MAX)
            .send()
            .await?
            .watch()
            .await?;

        ArenaToken::new(query.pool.currency1, self.base.client.clone().unwrap())
            .approve(self.deployment.unwrap(), Uint::MAX)
            .send()
            .await?
            .watch()
            .await?;

        PoolManager::new(self.deployment.unwrap(), self.base.client.clone().unwrap())
            .modifyLiquidity(query.pool, query.modification, Bytes::default())
            .send()
            .await?
            .watch()
            .await?;

        return Ok(ControlFlow::Continue);
    }
}
