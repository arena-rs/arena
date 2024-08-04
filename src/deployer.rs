use super::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Deployer {
    pub base: Base,
    pub manager: Option<Address>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PoolParams {
    pub key: PoolKey,

    pub sqrt_price_x96: U256,
    pub hook_data: Bytes,
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
    Fetcher(Address),
    LiquidityProvider(Address),

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


        let fetcher = Fetcher::deploy(client.clone()).await.unwrap();

        self.manager = Some(*pool_manager.address());

        messager
            .clone()
            .send(
                To::All,
                DeploymentResponse::PoolManager(*pool_manager.address()),
            )
            .await?;

        messager
            .clone()
            .send(To::All, DeploymentResponse::Fetcher(*fetcher.address()))
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
            DeploymentRequest::Pool(pool_creation) => {
                let key = pool_creation.clone();

                // will never panic as is always Some
                let pool_manager =
                    PoolManager::new(self.manager.unwrap(), self.base.client.clone().unwrap());

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
        }
    }
}
