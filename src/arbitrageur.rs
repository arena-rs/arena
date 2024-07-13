use super::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Arbitrageur {
    #[serde(skip)]
    pub messager: Option<Messager>,

    #[serde(skip)]
    pub client: Option<Arc<AnvilProvider>>,

    pub deployment: Option<Address>,

    pub pool: Option<Address>,
}

#[async_trait::async_trait]
impl Behavior<Message> for Arbitrageur {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<Message>>> {
        self.client = Some(client.clone());
        self.messager = Some(messager.clone());

        let mut stream = messager.clone().stream().unwrap();

        while let Some(event) = stream.next().await {
            let query: DeploymentResponse = match serde_json::from_str(&event.data) {
                Ok(query) => query,
                Err(_) => {
                    eprintln!("Failed to deserialize the event data into a DeploymentResponse");
                    continue;
                }
            };

            match query {
                DeploymentResponse::PoolManager(address) => self.deployment = Some(address),
                // DeploymentResponse::Pool(address) => self.pool = Some(address),
                _ => {}
            }

            if self.pool.is_some() && self.deployment.is_some() {
                break;
            }
        }

        Ok(Some(messager.clone().stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let query: PriceUpdate = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a PoolAdminQuery");
                return Ok(ControlFlow::Continue);
            }
        };

        return Ok(ControlFlow::Continue);
    }
}
