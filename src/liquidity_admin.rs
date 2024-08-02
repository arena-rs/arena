use super::*;

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct LiquidityAdmin {
    pub base: Base,
    pub deployment: Option<Address>,
    pub lp: Option<Address>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AllocationRequest {
    pub pool: PoolKey,
    pub modification: ModifyLiquidityParams,
    pub hook_data: Bytes,
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
            if let Ok(query) = serde_json::from_str::<DeploymentResponse>(&event.data) {
                match query {
                    DeploymentResponse::PoolManager(address) => self.deployment = Some(address),
                    DeploymentResponse::LiquidityProvider(address) => self.lp = Some(address),
                    _ => {}
                }
            }

            if self.lp.is_some() && self.deployment.is_some() {
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
            .approve(self.lp.unwrap(), Uint::from(100000))
            .send()
            .await?
            .watch()
            .await?;

        ArenaToken::new(query.pool.currency1, self.base.client.clone().unwrap())
            .approve(self.lp.unwrap(), Uint::from(100000))
            .send()
            .await?
            .watch()
            .await?;

        println!("deployment of manager: {:#?}", self.deployment.unwrap());

        let liquidity_provider =
            LiquidityProvider::new(self.lp.unwrap(), self.base.client.clone().unwrap());

        let key = query.pool.clone();

        let lp_key = LPoolKey {
            currency0: key.currency0,
            currency1: key.currency1,
            fee: key.fee,
            tickSpacing: key.tickSpacing,
            hooks: key.hooks,
        };

        liquidity_provider
            .createLiquidity(
                lp_key,
                query.modification.tickLower,
                query.modification.tickUpper,
                query.modification.liquidityDelta,
                query.hook_data,
            )
            .send()
            .await?
            .watch()
            .await?;

        Ok(ControlFlow::Continue)
    }
}
