use super::*;

pub enum IterationType {
    Definite(f64),
    Indefinite,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Orchestrator {
    pub base: Base,
    pub timesteps: f64,
    pub iterations: IterationType,
}

impl Orchestrator {
    fn new(timesteps: f64, iterations: usize) -> Self {
        Self {
            Base::default(),
            timesteps,
            iterations,
        }
    }
}


#[async_trait::async_trait]
impl Behavior<Message> for Orchestrator {
    async fn startup(&mut self, client, messenger) {
        messenger.send(To::All, PriceUpdate);
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow>{
        let query: = match serde_json::from_str()
    }
}