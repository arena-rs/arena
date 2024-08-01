use super::*;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum IterationType {
    Definite(usize), // How many loops should occur
    Indefinite, 
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum OrchestratorRequest {
    Terminate, // Terminate the Indefinite/Definite loop
    Restart, // Restart the Indefinite/Definite loop
    ChangeTo(IterationType), // Change the type of loop (Indefinite to Definite and vice versa)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Orchestrator {
    pub base: Base, // Base
    pub timesteps: f64, // How frequently the loop should occur
    pub iteration_type: IterationType, // The type of iteration which should happen
    pub iterations: usize, // The amount of iterations passed
    pub state: bool, // Whether the iterations have 
}

impl Orchestrator {
    fn new(timesteps: f64, iteration_type: IterationType) -> Self {
        Self {
            base: Base::default(),
            timesteps,
            iteration_type,
            iterations: 0,
            state: true,
        }
    }
}

#[async_trait::async_trait]
impl Behavior<Message> for Orchestrator {
    async fn startup(
        &mut self, 
        client: Arc<AnvilProvider>, 
        messenger: Messager
    ) -> Result<Option<EventStream<Message>>> {
        loop {
            sleep(Duration::from_secs_f64(self.timesteps)).await;
            if self.state {
                match self.iteration_type { 
                    IterationType::Indefinite => {
                        messenger.send(To::All, PriceUpdate).await;
                    }, 

                    IterationType::Definite(iteration_reps) => {
                        if iteration_reps >= self.iterations {
                            messenger.send(To::All, PriceUpdate).await;
                            self.iterations += 1;
                        }
                    },
                }
            }
        }
    }

    async fn process(&mut self, event: Message) -> Result<ControlFlow>{
        let query: OrchestratorRequest = match serde_json::from_str(&event.data) {
            Ok(query) => {
                query
            },
            Err(_) => {
                println!("Failed to deserialise the message");
                return Ok(ControlFlow::Continue);
            },
        };

        match query {
            OrchestratorRequest::Terminate => {
                self.state = false;
                Ok(ControlFlow::Continue)
            }

            OrchestratorRequest::Restart => {
                match self.iteration_type { 
                    IterationType::Indefinite => {
                        self.state = true;
                    }, 

                    IterationType::Definite(iteration_reps) => {
                        self.iterations = 0;
                        self.state = true;
                    },
                }
                Ok(ControlFlow::Continue)
            }

            OrchestratorRequest::ChangeTo(iteration_type) => {
                match iteration_type {
                    IterationType::Indefinite => {
                        self.iteration_type = iteration_type;
                    }, 

                    IterationType::Definite(iteration_reps) => {
                        self.iteration_type = iteration_type;
                    },

                    _ => {
                        println!("Attempted to use the wrong type of input");
                        return Ok(ControlFlow::Continue);
                    }
                }
                Ok(ControlFlow::Continue)
            }

            _ => {
                println!("Failed to find message");
                Ok(ControlFlow::Continue)
            },
        } 
    }
}