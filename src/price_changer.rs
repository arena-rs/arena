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

use serde::{ser::SerializeStruct, Deserializer, Serializer};

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceUpdate;

// impl Serialize for PriceUpdate {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut state = serializer.serialize_struct("PriceUpdate", 1)?;
//         state.serialize_field("type", "PriceUpdate")?;
//         state.end()
//     }
// }

// impl<'de> Deserialize<'de> for PriceUpdate {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         serde::de::Deserialize::deserialize(deserializer)?;
//         Ok(PriceUpdate)
//     }
// }

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
                Ok(query) => {
                    println!("heresd");
                    query
                }
                Err(_) => {
                    // eprintln!("Failed to deserialize the event datfa into a DeploymentResponse");
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
        let query: PriceUpdate = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a PriceUpdate");
                return Ok(ControlFlow::Continue);
            }
        };

        let liquid_exchange = LiquidExchange::new(self.lex.unwrap(), self.client.clone().unwrap());

        let tx = liquid_exchange.setPrice(alloy::primitives::utils::parse_ether(
            &self.process.step().to_string(),
        )?);

        tx.send().await?.watch().await?;

        println!("Price updated to: {}", self.process.current_value());

        Ok(ControlFlow::Continue)
    }
}
