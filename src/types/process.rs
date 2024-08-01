use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

pub trait StochasticProcess {
    fn current_value(&self) -> f64;

    fn step(&mut self) -> f64;
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
    fn current_value(&self) -> f64 {
        self.current_value
    }

    fn step(&mut self) -> f64 {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();

        let drift = self.theta * (self.mu - self.current_value) * self.dt;
        let randomness = self.sigma * self.dt.sqrt() * normal.sample(&mut rng);

        self.current_value += drift + randomness;
        self.current_value
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeometricBrownianMotion {
    initial_value: f64,
    current_value: f64,
    current_time: f64,
    mu: f64,
    sigma: f64,
    dt: f64,
}

impl GeometricBrownianMotion {
    pub fn new(initial_value: f64, mu: f64, sigma: f64, dt: f64) -> Self {
        GeometricBrownianMotion {
            initial_value,
            current_value: initial_value,
            current_time: 0.0,
            mu,
            sigma,
            dt,
        }
    }
}

impl StochasticProcess for GeometricBrownianMotion {
    fn current_value(&self) -> f64 {
        self.current_value
    }

    fn step(&mut self) -> f64 {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();

        let wiener_process = self.dt.sqrt() * normal.sample(&mut rng);
        let drift = self.mu * self.current_time;
        let volatility = self.sigma * wiener_process;

        let change = drift + volatility;

        self.current_time += self.dt;
        self.current_value = self.initial_value * change.exp();
        self.current_value
    }
}