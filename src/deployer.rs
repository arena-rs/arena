use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Deployer {
    #[serde(skip)]
    pub messager: Option<Messager>,

    #[serde(skip)]
    pub client: Option<Arc<AnvilProvider>>,
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

        messager.clone()
            .send(
                To::All,
                DeploymentResponse::PoolManager(*pool_manager.address()),
            )
            .await?;

        self.client = Some(client.clone());
        self.messager = Some(messager.clone());

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
                    ArenaToken::deploy(self.client.clone().unwrap(), name, symbol, decimals)
                        .await
                        .unwrap();

                println!("Token deployed at address: {:?}", token.address());

                self.messager
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
                    self.client.clone().unwrap(),
                    token_0,
                    token_1,
                    U256::from((initial_price * 10f64.powf(18.0)) as u64),
                )
                .await
                .unwrap();

                self.messager
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
