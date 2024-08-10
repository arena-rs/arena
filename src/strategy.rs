use super::*;

pub trait Strategy {
    fn init(&self, provider: AnvilProvider);
    fn process(&self, provider: AnvilProvider);
}
