use super::*;

#[derive(Serialize, Deserialize)]
pub struct PriceChanger {
    mu: f64,
    sigma: f64,
    theta: f64,

    seed: u64,

    #[serde(skip)]
    #[serde(default = "trajectory_default")]
    pub current_chunk: Trajectories,

    #[serde(skip)]
    pub client: Option<Arc<AnvilProvider>>,

    pub liquid_exchange: Option<Address>,

    cursor: usize,
    value: f64,
}

fn trajectory_default() -> Trajectories {
    Trajectories {
        times: Vec::new(),
        paths: Vec::new(),
    }
}

impl fmt::Debug for PriceChanger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceUpdate;

impl PriceChanger {
    /// Public constructor function to create a [`PriceChanger`] behaviour.
    pub fn new(initial_value: f64, mu: f64, sigma: f64, theta: f64, seed: u64) -> Self {
        let ou = OrnsteinUhlenbeck::new(mu, sigma, theta);

        // Chunk our price trajectory over 100 price points.
        let current_chunk =
            ou.seedable_euler_maruyama(initial_value, 0.0, 100.0, 100_usize, 1_usize, false, seed);

        Self {
            mu,
            sigma,
            theta,
            seed,
            current_chunk,
            cursor: 0,
            client: None,
            liquid_exchange: None,
            value: initial_value,
        }
    }
}

#[async_trait::async_trait]
impl Behavior<Message> for PriceChanger {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<Message>>> {
        self.client = Some(client);

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
                self.liquid_exchange = Some(address);
                break;
            }
        }

        Ok(Some(messager.clone().stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let ou = OrnsteinUhlenbeck::new(self.mu, self.sigma, self.theta);

        let _query: PriceUpdate = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a PriceUpdate");
                return Ok(ControlFlow::Continue);
            }
        };

        if self.cursor >= 99 {
            self.cursor = 0;
            self.value = self.current_chunk.paths.clone()[0][self.cursor];
            self.current_chunk = ou.seedable_euler_maruyama(
                self.value, 0.0, 100.0, 100_usize, 1_usize, false, self.seed,
            );
        }

        let liquid_exchange =
            LiquidExchange::new(self.liquid_exchange.unwrap(), self.client.clone().unwrap());

        let price = self.current_chunk.paths.clone()[0][self.cursor];

        let tx = liquid_exchange
            .setPrice(alloy::primitives::utils::parse_ether(&price.to_string())?);

        tx.send().await?.watch().await?;

        self.cursor += 1;

        Ok(ControlFlow::Continue)
    }
}