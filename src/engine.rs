use crossbeam_channel::{unbounded, Receiver, Sender};
use revm::{
    db::{emptydb::EmptyDB, in_memory_db::CacheDB},
};

pub struct Engine {
    pub env: CacheDB<EmptyDB>,
    pub socket: Receiver<Instruction>,

    send: Sender<Instruction>,
}

pub enum Instruction {
    CreateToken {
        name: String,
        symbol: String,
        decimals: u8,
    },

    Halt,
}

impl Engine {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<Instruction>();
        Self {
            env: CacheDB::new(EmptyDB::new()),
            socket: receiver,
            send: sender,
        }
    }

    pub fn spawn(&self) -> Sender<Instruction> {
        self.send.clone()
    }

    pub fn run(&self) {
        while let Ok(instruction) = self.socket.recv() {
            match instruction {
                Instruction::CreateToken { name, symbol, decimals } => {
                    println!("Creating {name} token with symbol {symbol} and decimals {decimals}");
                }
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

        sender.send(Instruction::CreateToken {
            name: "Test".to_string(),
            symbol: "TST".to_string(),
            decimals: 18,
        }).unwrap();

        sender.send(Instruction::Halt).unwrap();

        engine.run();
    }
}
