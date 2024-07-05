use revm::db::{emptydb::EmptyDB, in_memory_db::CacheDB};

use crate::agent::Agent;

pub struct Environment {
    pub db: CacheDB<EmptyDB>,

    pub agent: Agent,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            db: CacheDB::new(EmptyDB::new()),
            agent: Agent,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{Address, U256};

    use super::*;

    #[test]
    fn test_environment_creation() {
        let mut env = Environment::new();

        env.db
            .insert_account_storage(Address::default(), U256::from(0), U256::from(1000)).unwrap();

        let value = env
            .db
            .load_account(Address::default())
            .unwrap()
            .storage
            .get(&U256::from(0))
            .unwrap();

        assert_eq!(value, &U256::from(1000));
    }
}
