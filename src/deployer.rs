use super::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Deployer {
    pub base: Base,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DeploymentRequest {
    Token {
        name: String,
        symbol: String,
        decimals: u8,
    },

    LiquidExchange {
        token_0: Address,
        token_1: Address,
        initial_price: f64,
    },

    Pool(PoolParams),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DeploymentResponse {
    Token(Address),
    LiquidExchange(Address),
    PoolManager(Address),

    // params pass through if deployment is successful
    Pool(PoolParams),
}

#[async_trait::async_trait]
impl Behavior<Message> for Deployer {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<Message>>> {
        let pool_manager = PoolManager::deploy(client.clone(), Uint::from(5000))
            .await
            .unwrap();

        messager
            .clone()
            .send(
                To::All,
                DeploymentResponse::PoolManager(*pool_manager.address()),
            )
            .await?;

        self.base.client = Some(client.clone());
        self.base.messager = Some(messager.clone());

        Ok(Some(messager.stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        println!("Received event: {:?}", event);
        let query: DeploymentRequest = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                println!("Failed to deserialize the event data into a DeploymentRequest");
                return Ok(ControlFlow::Continue);
            }
        };

        println!("Received deployment request: {:?}", query);

        match query {
            DeploymentRequest::Token {
                name,
                symbol,
                decimals,
            } => {
                let token =
                    ArenaToken::deploy(self.base.client.clone().unwrap(), name, symbol, decimals)
                        .await
                        .unwrap();

                println!("Token deployed at address: {:?}", token.address());

                self.base
                    .messager
                    .clone()
                    .unwrap()
                    .send(To::All, DeploymentResponse::Token(*token.address()))
                    .await?;

                Ok(ControlFlow::Continue)
            }
            DeploymentRequest::LiquidExchange {
                token_0,
                token_1,
                initial_price,
            } => {
                let lex = LiquidExchange::deploy(
                    self.base.client.clone().unwrap(),
                    token_0,
                    token_1,
                    U256::from((initial_price * 10f64.powf(18.0)) as u64),
                )
                .await
                .unwrap();

                self.base
                    .messager
                    .clone()
                    .unwrap()
                    .send(To::All, DeploymentResponse::LiquidExchange(*lex.address()))
                    .await?;

                Ok(ControlFlow::Continue)
            }
            _ => Ok(ControlFlow::Continue),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use octane::{agent::Agent, world::World};

//     use super::*;
//     use crate::deployer::Deployer;

//     #[derive(Debug, Serialize, Deserialize)]
//     pub struct MockDeployer {
//         #[serde(skip)]
//         pub messager: Option<Messager>,

//         #[serde(skip)]
//         pub client: Option<Arc<AnvilProvider>>,
//     }

//     #[async_trait::async_trait]
//     impl Behavior<Message> for MockDeployer {
//         async fn startup(
//             &mut self,
//             client: Arc<AnvilProvider>,
//             messager: Messager,
//         ) -> Result<Option<EventStream<Message>>> {
//             messager
//                 .send(
//                     To::Agent("deployer".to_string()),
//                     DeploymentRequest::Token {
//                         name: String::from("TEST0"),
//                         symbol: String::from("TST0"),
//                         decimals: 18,
//                     },
//                 )
//                 .await?;

//             self.client = Some(client.clone());
//             self.messager = Some(messager.clone());

//             Ok(Some(messager.clone().stream().unwrap()))
//         }

//         async fn process(&mut self, event: Message) -> Result<ControlFlow> {
//             let query: DeploymentResponse = match serde_json::from_str(&event.data) {
//                 Ok(query) => query,
//                 Err(_) => {
//                     eprintln!("Failed to deserialize the event data into a DeploymentResponse");
//                     return Ok(ControlFlow::Continue);
//                 }
//             };

//             match query {
//                 DeploymentResponse::Token(address) => {
//                     let tok = ArenaToken::new(address, self.client.clone().unwrap());

//                     assert_eq!(tok.name().call().await.unwrap()._0, "TEST");
//                     assert_eq!(tok.symbol().call().await.unwrap()._0, "TST");
//                     assert_eq!(tok.decimals().call().await.unwrap()._0, 18);
//                 }
//                 _ => {}
//             }

//             Ok(ControlFlow::Continue)
//         }
//     }

//     #[tokio::test]
//     async fn test_deployer() {
//         // env_logger::init();

//         let deployer = Agent::builder("deployer").with_behavior(Deployer {
//             messager: None,
//             client: None,
//         });
//         let mock_deployer = Agent::builder("mock_deployer").with_behavior(MockDeployer {
//             client: None,
//             messager: None,
//         });

//         let mut world = World::new("id");

//         world.add_agent(mock_deployer);
//         world.add_agent(deployer);

//         let _ = world.run().await;
//     }
// }
