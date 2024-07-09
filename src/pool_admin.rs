use octane::{machine::ControlFlow, messenger::Message};

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolAdmin {
    #[serde(skip)]
    pub messager: Option<Messager>,

    #[serde(skip)]
    pub client: Option<Arc<AnvilProvider>>,
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
    hook_date: Bytes,
}

#[async_trait::async_trait]
impl Behavior<Message> for PoolAdmin {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<Message>>> {
        self.client = Some(client.clone());
        self.messager = Some(messager.clone());

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
            PoolAdminQuery::CreatePool(pool_creation) => Ok(ControlFlow::Continue),
        }
    }
}
