/// Configuration for the simulation.
pub struct Config {
    /// Number of steps to run the simulation for.
    pub steps: usize,
}

impl Config {
    /// Public constructor function for a new [`Config`].
    pub fn new(steps: usize) -> Self {
        Config { steps }
    }
}
