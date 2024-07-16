use super::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct PriceChanger {
    #[serde(skip)]
    pub messager: Option<Messager>,

    #[serde(skip)]
    pub client: Option<Arc<AnvilProvider>>,

    pub process: OrnsteinUhlenbeck,

    pub lex: Option<Address>,
}

impl PriceChanger {
    pub fn new(process: OrnsteinUhlenbeck) -> Self {
        Self {
            messager: None,
            client: None,
            process,
            lex: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceUpdate;

#[async_trait::async_trait]
impl Behavior<Message> for PriceChanger {
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

            if let DeploymentResponse::LiquidExchange(address) = query {
                self.lex = Some(address);
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

        let liquid_exchange =
            LiquidExchange::new(self.lex.unwrap(), self.client.clone().unwrap());

        let tx = liquid_exchange.setPrice(alloy::primitives::utils::parse_ether(
            &self.process.step().to_string(),
        )?);

        tx.send().await?.watch().await?;

        Ok(ControlFlow::Continue)
    }
}
