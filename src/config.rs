pub struct Config {
    pub steps: usize,
}

impl Config {
    pub fn new(steps: usize) -> Self {
        Config { steps }
    }
}
