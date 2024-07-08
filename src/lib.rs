use std::sync::Arc;

use alloy::{
    network::EthereumWallet, node_bindings::Anvil, primitives::U256, providers::ProviderBuilder,
    signers::local::PrivateKeySigner, sol,
};
use anyhow::{anyhow, Result};
use octane::{
    agent::Agent,
    machine::{Behavior, EventStream},
    messenger::{Messager, To},
    AnvilProvider,
};
use alloy::primitives::Uint;
use serde::{Deserialize, Serialize};
use crate::bindings::poolmanager::PoolManager;

pub mod bindings;

#[derive(Debug, Serialize, Deserialize)]
struct Deployer;

#[async_trait::async_trait]
impl Behavior<()> for Deployer {
    async fn startup(
        &mut self,
        client: Arc<AnvilProvider>,
        messager: Messager,
    ) -> Result<Option<EventStream<()>>> {
        let pool_manager = PoolManager::deploy(client.clone(), Uint::from(5000));
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behaviour() {
        let messager = Messager::new();
        let anvil = Anvil::new().try_spawn().unwrap();

        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet = EthereumWallet::from(signer);

        let rpc_url = anvil.endpoint().parse().unwrap();
        let provider = ProviderBuilder::new().wallet(wallet).on_http(rpc_url);

        let agent = Agent::builder("deployer")
            .with_behavior(Deployer)
            .build(Arc::new(provider), messager);
    }
}
