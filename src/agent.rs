use crate::engine::Instruction;

pub trait Agent {
    fn act(&mut self, event: Instruction) -> Option<Instruction>;
}
