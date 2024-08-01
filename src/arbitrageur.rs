use super::*;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Arbitrageur {
    pub base: Base,
    pub deployment: Option<Address>,
    pub pool: Option<PoolParams>,
    pub fetcher: Option<Address>,
    pub liquid_exchange: Option<Address>,
}

impl Arbitrageur {
    pub fn profit(dy: f64, p_ext: f64, p_uni: f64, fee: f64, liquidity: f64) -> (f64, f64) {
        let sqrt_p_uni = p_uni.sqrt();
        let delta_p = dy / (liquidity * sqrt_p_uni);
        let new_price = p_uni + delta_p;

        let dx = dy / ((1.0 - fee) * new_price);

        let revenue = dy * p_ext;

        let profit = revenue - dx;
        (profit, new_price)
    }

    pub async fn optimal_swap(&self, p_ext: f64, p_uni: f64, fee: f64, initial_tick: f64) -> f64 {
        let (mut a, mut b) = (0.0, 1f64.powi(6));

        let mut current_tick = initial_tick;

        let mut next_tick_liquidity = self.get_next_tick_liquidity(current_tick as i32).await;
        let tol = 1f64.powi(-6);

        while b - a > tol {
            let mid = (a + b) / 2.0;

            let (profit_mid, new_price_mid) =
                Arbitrageur::profit(mid, p_ext, p_uni, fee, next_tick_liquidity);
            let (profit_next, _new_price_next) =
                Arbitrageur::profit(mid + tol, p_ext, p_uni, fee, next_tick_liquidity);

            if profit_mid > profit_next {
                b = mid;
            } else {
                a = mid;
            }

            if new_price_mid >= Arbitrageur::get_price_at_tick(current_tick + 1.0) {
                current_tick += 1.0;
                next_tick_liquidity = self.get_next_tick_liquidity(current_tick as i32).await;
            } else if new_price_mid <= Arbitrageur::get_price_at_tick(current_tick - 1.0) {
                current_tick -= 1.0;
                next_tick_liquidity = self.get_next_tick_liquidity(current_tick as i32).await;
            }
        }

        (a + b) / 2.0
    }

    pub fn get_price_at_tick(tick: f64) -> f64 {
        let sqrt = 1.0001f64.powf(tick);
        sqrt * sqrt
    }

    pub async fn get_next_tick_liquidity(&self, tick: i32) -> f64 {
        let fetcher_key = FetcherPoolKey {
            currency0: self.pool.clone().unwrap().key.currency0,
            currency1: self.pool.clone().unwrap().key.currency1,
            fee: self.pool.clone().unwrap().key.fee,
            tickSpacing: self.pool.clone().unwrap().key.tickSpacing,
            hooks: self.pool.clone().unwrap().key.hooks,
        };

        let fetcher = Fetcher::new(self.fetcher.unwrap(), self.base.client.clone().unwrap());
        let id = fetcher.toId(fetcher_key).call().await.unwrap().poolId;

        fetcher
            .getTickLiquidity(self.deployment.unwrap(), id, tick)
            .call()
            .await
            .unwrap()
            .liquidityGross as f64
    }
}

#[async_trait::async_trait]
impl Behavior<Message> for Arbitrageur {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<Message>>> {
        self.base.client = Some(client.clone());
        self.base.messager = Some(messager.clone());

        let mut stream = messager.clone().stream().unwrap();

        while let Some(event) = stream.next().await {
            if let Ok(query) = serde_json::from_str::<DeploymentResponse>(&event.data) {
                match query {
                    DeploymentResponse::PoolManager(address) => self.deployment = Some(address),
                    DeploymentResponse::Pool(params) => self.pool = Some(params),
                    DeploymentResponse::LiquidExchange(address) => {
                        self.liquid_exchange = Some(address)
                    }
                    DeploymentResponse::Fetcher(address) => self.fetcher = Some(address),
                    _ => {}
                }
            }

            if self.pool.is_some()
                && self.deployment.is_some()
                && self.fetcher.is_some()
                && self.liquid_exchange.is_some()
            {
                break;
            }
        }

        Ok(Some(messager.clone().stream().unwrap()))
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow> {
        let _query: Signal = match serde_json::from_str(&event.data) {
            Ok(query) => query,
            Err(_) => {
                eprintln!("Failed to deserialize the event data into a Signal");
                return Ok(ControlFlow::Continue);
            }
        };

        let manager = PoolManager::new(self.deployment.unwrap(), self.base.client.clone().unwrap());
        let fetcher = Fetcher::new(self.fetcher.unwrap(), self.base.client.clone().unwrap());
        let liquid_exchange = LiquidExchange::new(
            self.liquid_exchange.unwrap(),
            self.base.client.clone().unwrap(),
        );

        let pool = self.pool.clone().unwrap();

        let fetcher_key = FetcherPoolKey {
            currency0: pool.key.currency0,
            currency1: pool.key.currency1,
            fee: pool.key.fee,
            tickSpacing: pool.key.tickSpacing,
            hooks: pool.key.hooks,
        };

        let id = fetcher.toId(fetcher_key).call().await?.poolId;

        let get_slot0_return = fetcher.getSlot0(*manager.address(), id).call().await?;

        let pricex192 = get_slot0_return.sqrtPriceX96.pow(U256::from(2));
        let two_pow_192 = U256::from(1u128) << 192;

        let scaled_price: U256 = (pricex192 * U256::from(10u128).pow(U256::from(18))) / two_pow_192;

        let lex_price = liquid_exchange.price().call().await?._0;

        let _diff = scaled_price.abs_diff(lex_price);

        let p_uni = f64::from(scaled_price) / 10f64.powi(18);
        let p_ext = f64::from(lex_price) / 10f64.powi(18);

        match p_uni.partial_cmp(&p_ext) {
            Some(Ordering::Greater) => {
                let swap_amount = self
                    .optimal_swap(p_ext, p_uni, 0.003, get_slot0_return.tick as f64)
                    .await;

                println!("greater: Optimal swap amount: {}", swap_amount);
            }
            Some(Ordering::Less) => {
                let swap_amount = self
                    .optimal_swap(p_uni, p_ext, 0.003, get_slot0_return.tick as f64)
                    .await;
                println!("lesser: Optimal swap amount: {}", swap_amount);
            }
            Some(Ordering::Equal) => return Ok(ControlFlow::Continue),
            None => panic!(),
        }

        println!("tick: {}", get_slot0_return.tick);
        println!("price: {}", p_ext);

        Ok(ControlFlow::Continue)
    }
}
