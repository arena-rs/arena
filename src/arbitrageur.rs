use super::*;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Arbitrageur {
    pub base: Base,
    pub deployment: Option<Address>,
    pub pool: Option<PoolParams>,
    pub fetcher: Option<Address>,
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
                    DeploymentResponse::Pool(params) => {
                        println!("FINALLY GOT HERE");
                        self.pool = Some(params)
                    }
                    DeploymentResponse::Fetcher(address) => self.fetcher = Some(address),
                    _ => {}
                }
            }

            if self.pool.is_some() && self.deployment.is_some() && self.fetcher.is_some() {
                break;
            }
        }

        Ok(Some(messager.clone().stream().unwrap()))
    }
    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let _query: PriceUpdate = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a PriceUpdate");
                return Ok(ControlFlow::Continue);
            }
        };

        let manager = PoolManager::new(self.deployment.unwrap(), self.base.client.clone().unwrap());
        let fetcher = Fetcher::new(self.fetcher.unwrap(), self.base.client.clone().unwrap());

        let fetcher_key = FetcherPoolKey {
            currency0: self.pool.clone().unwrap().key.currency0,
            currency1: self.pool.clone().unwrap().key.currency1,
            fee: self.pool.clone().unwrap().key.fee,
            tickSpacing: self.pool.clone().unwrap().key.tickSpacing,
            hooks: self.pool.clone().unwrap().key.hooks,
        };

        println!("key: {:#?}", fetcher_key);

        let id = fetcher.toId(fetcher_key).call().await?.poolId;

        // println!("id: {:?}", id);

        let get_slot0_return = fetcher
            .getSlot0(manager.address().clone(), id)
            .call()
            .await?;

        println!("price: {:?}", get_slot0_return);

        return Ok(ControlFlow::Continue);
    }
}
