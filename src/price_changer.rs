use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceUpdate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Signal;

#[derive(Serialize, Deserialize, Debug)]
pub struct PriceChanger<T>
where
    T: StochasticProcess,
{
    pub base: Base,
    pub process: T,
    pub lex: Option<Address>,
}

impl<T: StochasticProcess> PriceChanger<T> {
    pub fn new(process: T) -> Self {
        Self {
            base: Base::default(),
            process,
            lex: None,
        }
    }
}

#[async_trait::async_trait]
impl<T> Behavior<Message> for PriceChanger<T>
where
    T: Debug + Send + Sync + for<'a> Deserialize<'a> + Serialize + StochasticProcess + 'static,
{
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

            if let DeploymentResponse::LiquidExchange(address) = query {
                self.lex = Some(address);
                break;
            }
        }

        Ok(Some(messager.stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let query: PriceUpdate = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a PriceUpdate");
                return Ok(ControlFlow::Continue);
            }
        };

        let liquid_exchange =
            LiquidExchange::new(self.lex.unwrap(), self.base.client.clone().unwrap());

        let tx = liquid_exchange.setPrice(alloy::primitives::utils::parse_ether(
            &self.process.step().to_string(),
        )?);

        tx.send().await?.watch().await?;

        self.base.messager.clone().unwrap()
            .send(
                To::All,
                Signal,
            )
            .await?;

        Ok(ControlFlow::Continue)
    }
}
