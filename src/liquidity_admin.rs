use super::*;

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct LiquidityAdmin {
    pub base: Base,
    pub deployment: Option<Address>,
    pub lp: Option<Address>,
    pub pool: Option<PoolParams>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
                    DeploymentResponse::Pool(pool_params) => self.pool = Some(pool_params),
                    _ => {}
                }
            }

            if self.deployment.is_some() && self.pool.is_some() {
                break;
            }
        }

        Ok(Some(messager.stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let query: AllocationRequest = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => return Ok(ControlFlow::Continue),
        };

        println!("duck");


        let liquidity_provider = LiquidityProvider::deploy(self.base.client.clone().unwrap(), self.deployment.unwrap())
            .await
            .unwrap();  

        let currency0 = ArenaToken::new(query.pool.currency0, self.base.client.clone().unwrap());
        let currency1 = ArenaToken::new(query.pool.currency1, self.base.client.clone().unwrap());

        currency0.mint(Uint::MAX).send().await?.watch().await?;
        currency1.mint(Uint::MAX).send().await?.watch().await?;

        currency0
            .approve(*liquidity_provider.address(), Uint::MAX)
            .send()
            .await?
            .watch()
            .await?;

        currency1
            .approve(*liquidity_provider.address(), Uint::MAX)
            .send()
            .await?
            .watch()
            .await?;

        let key = query.pool.clone();

        let lp_key = LPoolKey {
            currency0: key.currency0,
            currency1: key.currency1,
            fee: key.fee,
            tickSpacing: key.tickSpacing,
            hooks: key.hooks,
        };

        println!("do we get here");

        println!("lp_key: {:#?}", lp_key);
        println!("query.modification.tickLower: {:#?}", query.modification.tickLower);

        let tx = liquidity_provider.createLiquidity(
            lp_key,
            query.modification.tickLower,
            query.modification.tickUpper,
            query.modification.liquidityDelta,
            query.hook_data,
        );

        let send_result = tx.call().await;

        println!("send result: {:#?}", send_result);
        
        // let watch_result = send_result?.watch().await;

        // println!("watch result: {:#?}", watch_result);

        Ok(ControlFlow::Continue)
    }
}
