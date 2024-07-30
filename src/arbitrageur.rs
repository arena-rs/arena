use super::*;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Arbitrageur {
    pub base: Base,
    pub deployment: Option<Address>,
    pub pool: Option<PoolParams>,
    pub fetcher: Option<Address>,
    pub liquid_exchange: Option<Address>,
}

#[async_trait::async_trait]
impl Behavior<Message> for Arbitrageur {
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
                    DeploymentResponse::Pool(params) => self.pool = Some(params),
                    DeploymentResponse::LiquidExchange(address) => {
                        self.liquid_exchange = Some(address)
                    }
                    DeploymentResponse::Fetcher(address) => self.fetcher = Some(address),
                    _ => {}
                }
            }

            if self.pool.is_some()
                && self.deployment.is_some()
                && self.fetcher.is_some()
                && self.liquid_exchange.is_some()
            {
                break;
            }
        }

        Ok(Some(messager.clone().stream().unwrap()))
    }
    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let _query: Signal = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a Signal");
                return Ok(ControlFlow::Continue);
            }
        };

        let manager = PoolManager::new(self.deployment.unwrap(), self.base.client.clone().unwrap());
        let fetcher = Fetcher::new(self.fetcher.unwrap(), self.base.client.clone().unwrap());
        let liquid_exchange =
            LiquidExchange::new(self.liquid_exchange.unwrap(), self.base.client.clone().unwrap());


        let fetcher_key = FetcherPoolKey {
            currency0: self.pool.clone().unwrap().key.currency0,
            currency1: self.pool.clone().unwrap().key.currency1,
            fee: self.pool.clone().unwrap().key.fee,
            tickSpacing: self.pool.clone().unwrap().key.tickSpacing,
            hooks: self.pool.clone().unwrap().key.hooks,
        };

        let id = fetcher.toId(fetcher_key).call().await?.poolId;

        let get_slot0_return = fetcher
            .getSlot0(manager.address().clone(), id)
            .call()
            .await?;

        let sqrt_price_x96 = get_slot0_return.sqrtPriceX96;

        let pricex192 = sqrt_price_x96.pow(U256::from(2));

        let two_pow_192 = U256::from(1u128) << 192;
        
        let scaled_price: U256 = (pricex192 * U256::from(10u128).pow(U256::from(18))) / two_pow_192;
        
        let lex_price = liquid_exchange
            .price()
            .call()
            .await?._0;

        let diff = scaled_price.abs_diff(lex_price);

        println!("diff: {}", diff);        

        Ok(ControlFlow::Continue)
    }
}
