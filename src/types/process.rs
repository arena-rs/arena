use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

pub trait StochasticProcess {
    type Value;

    fn current_value(&self) -> Self::Value;

    fn step(&mut self) -> Self::Value;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrnsteinUhlenbeck {
    current_value: f64,

    /// Mean reversion rate
    theta: f64,

    /// Long-term mean
    mu: f64,

    // Volatility
    sigma: f64,

    // Time step
    dt: f64,
}

impl OrnsteinUhlenbeck {
    pub fn new(initial_value: f64, theta: f64, mu: f64, sigma: f64, dt: f64) -> Self {
        OrnsteinUhlenbeck {
            current_value: initial_value,
            theta,
            mu,
            sigma,
            dt,
        }
    }
}

impl StochasticProcess for OrnsteinUhlenbeck {
    type Value = f64;

    fn current_value(&self) -> Self::Value {
        self.current_value
    }

    fn step(&mut self) -> Self::Value {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();

        let drift = self.theta * (self.mu - self.current_value) * self.dt;
        let randomness = self.sigma * self.dt.sqrt() * normal.sample(&mut rng);

        self.current_value += drift + randomness;
        self.current_value
    }
}
