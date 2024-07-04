use alloy::primitives::{Address, U256};
use revm::db::{emptydb::EmptyDB, in_memory_db::CacheDB};

pub struct Environment {
    pub db: CacheDB<EmptyDB>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            db: CacheDB::new(EmptyDB::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_creation() {
        let mut env = Environment::new();

        env.db
            .insert_account_storage(Address::default(), U256::from(0), U256::from(1000));

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
