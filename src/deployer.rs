use super::*;

#[derive(Debug, Serialize, Deserialize)]
struct Deployer {
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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DeploymentResponse {
    Token(Address),
    LiquidExchange(Address),
    PoolManager(Address),
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
            .send(
                To::All,
                DeploymentResponse::PoolManager(*pool_manager.address()),
            )
            .await?;

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
            DeploymentRequest::Token {
                name,
                symbol,
                decimals,
            } => {
                let token =
                    ArenaToken::deploy(self.client.clone().unwrap(), name, symbol, decimals)
                        .await
                        .unwrap();

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_behaviour() {
        env_logger::init();

        // messager, client are initialized later
        let agent = Agent::builder("deployer").with_behavior(Deployer { messager: None, client: None });

        let mut world = World::new("id");

        world.add_agent(agent);
        world.run().await;
    }
}
