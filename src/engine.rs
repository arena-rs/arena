use std::collections::HashMap;

use alloy::primitives::{Address, U256, Uint, Signed};
use crossbeam_channel::{unbounded, Receiver, Sender};
use revm::db::{emptydb::EmptyDB, in_memory_db::CacheDB};

use crate::agent::Agent;

pub struct Engine {
    pub env: CacheDB<EmptyDB>,
    pub socket: Receiver<Instruction>,
    pub agents: HashMap<String, Box<dyn Agent>>,

    send: Sender<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    CreateToken {
        name: String,
        symbol: String,
        decimals: u8,
    },

    CreatePool {
        token_0: Address,
        token_1: Address,
        fee: Uint<24, 1>,
        tick_spacing: Signed<24, 1>,
        sqrt_price_x96: Uint<160, 3>,
    },

    Message {
        id: String,
        message: String, 
    },

    Tick,
    Halt,
}

impl Engine {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<Instruction>();
        Self {
            env: CacheDB::new(EmptyDB::new()),
            socket: receiver,
            agents: HashMap::new(),
            send: sender,
        }
    }

    pub fn spawn(&self) -> Sender<Instruction> {
        self.send.clone()
    }

    pub fn run(&mut self) {
        while let Ok(instruction) = self.socket.recv() {
            for (_, agent) in &mut self.agents {
                agent.act(instruction.clone());
            }

            match instruction {
                Instruction::CreateToken {
                    name,
                    symbol,
                    decimals,
                } => {
                    println!("Creating {name} token with symbol {symbol} and decimals {decimals}");
                }
                Instruction::CreatePool {
                    token_0,
                    token_1,
                    fee,
                    tick_spacing,
                    sqrt_price_x96
                } => {
                    println!("Creating pool between {token_0} and {token_1} with fee {fee}");
                }
                Instruction::Tick => continue,
                Instruction::Message { .. } => continue,
                Instruction::Halt => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_sending() {
        let engine = Engine::new();
        let sender = engine.spawn();

        sender
            .send(Instruction::CreateToken {
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                decimals: 18,
            })
            .unwrap();

        sender.send(Instruction::Halt).unwrap();

        engine.run();
    }
}
