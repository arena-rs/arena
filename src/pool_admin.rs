use futures::stream::StreamExt;
use octane::{machine::ControlFlow, messenger::Message};

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolAdmin {
    pub base: Base,

    pub deployment: Option<Address>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PoolParams {
    #[serde(skip)]
    key: PoolKey,

    sqrt_price_x96: U256,
    hook_data: Bytes,
}

#[async_trait::async_trait]
impl Behavior<Message> for PoolAdmin {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<Message>>> {
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

        self.base.client = Some(client.clone());
        self.base.messager = Some(messager.clone());

        Ok(Some(messager.clone().stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let query: DeploymentRequest = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a PoolAdminQuery");
                return Ok(ControlFlow::Continue);
            }
        };

        match query {
            DeploymentRequest::Pool(pool_creation) => {
                let key = pool_creation.clone();

                // will never panic as is always Some
                let pool_manager =
                    PoolManager::new(self.deployment.unwrap(), self.base.client.clone().unwrap());

                let tx = pool_manager.initialize(
                    pool_creation.key,
                    pool_creation.sqrt_price_x96,
                    pool_creation.hook_data,
                );

                tx.send().await?.watch().await?;

                self.base
                    .messager
                    .clone()
                    .unwrap()
                    .send(To::All, DeploymentResponse::Pool(key))
                    .await?;

                Ok(ControlFlow::Continue)
            }
            _ => Ok(ControlFlow::Continue),
        }
    }
}
