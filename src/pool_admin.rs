use octane::{machine::ControlFlow, messenger::Message};

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolAdmin {
    #[serde(skip)]
    pub messager: Option<Messager>,

    #[serde(skip)]
    pub client: Option<Arc<AnvilProvider>>,

    pub deployment: Option<Address>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PoolAdminQuery {
    /// Deploy request.
    CreatePool(PoolParams),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolParams {
    #[serde(skip)]
    key: PoolKey,

    sqrt_price_x96: U256,
    hook_data: Bytes,
}

use futures::stream::StreamExt;

#[async_trait::async_trait]
impl Behavior<Message> for PoolAdmin {
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

            if let DeploymentResponse::PoolManager(address) = query {
                self.deployment = Some(address);
                break;
            }
        }

        Ok(Some(messager.clone().stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let query: PoolAdminQuery = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a PoolAdminQuery");
                return Ok(ControlFlow::Continue);
            }
        };

        match query {
            PoolAdminQuery::CreatePool(pool_creation) => {
                // will never panic as is always Some
                let pool_manager = PoolManager::new(
                    self.deployment.clone().unwrap(),
                    self.client.clone().unwrap(),
                );

                let tx = pool_manager.initialize(
                    pool_creation.key,
                    pool_creation.sqrt_price_x96,
                    pool_creation.hook_data,
                );

                tx.send().await?.watch().await?;

                Ok(ControlFlow::Continue)
            }
        }
    }
}
