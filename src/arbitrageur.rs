use super::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Arbitrageur {
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
                    DeploymentResponse::Pool(params) => self.pool = Some(params),
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

        let id = keccak256(&self.pool.clone().unwrap().key.encode());

        let slot0 = fetcher
            .getSlot0(*manager.address(), id)
            .send()
            .await?
            .watch()
            .await?;

        return Ok(ControlFlow::Continue);
    }
}
