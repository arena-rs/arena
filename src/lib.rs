use std::sync::Arc;

use alloy::{
    network::EthereumWallet,
    node_bindings::Anvil,
    primitives::{Address, Bytes, Signed, Uint, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use anyhow::{anyhow, Result};
use octane::{
    agent::Agent,
    world::World,
    machine::{Behavior, EventStream},
    messenger::{Messager, To},
    AnvilProvider,
};
use serde::{Deserialize, Serialize};

use crate::bindings::{
    arenatoken::ArenaToken,
    poolmanager::{PoolManager, PoolManager::PoolKey},
};

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
        let pool_manager = PoolManager::deploy(client.clone(), Uint::from(5000))
            .await
            .unwrap();

        let currency_0 = ArenaToken::deploy(
            client.clone(),
            String::from("ARN0"),
            String::from("ARN0"),
            18,
        )
        .await
        .unwrap();
        let currency_1 = ArenaToken::deploy(
            client.clone(),
            String::from("ARN1"),
            String::from("ARN1"),
            18,
        )
        .await
        .unwrap();

        let key = PoolKey {
            currency0: *currency_0.address(),
            currency1: *currency_1.address(),
            fee: 0,
            tickSpacing: 24,
            hooks: Address::default(),
        };

        let tx = pool_manager.initialize(key, Uint::from(1000), Bytes::default());

        let tx_hash = tx.send().await?.watch().await?;

        println!("pool deployed: {tx_hash}");

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_behaviour() {
        let messager = Messager::new();
        let anvil = Anvil::new().try_spawn().unwrap();

        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet = EthereumWallet::from(signer);

        let rpc_url = anvil.endpoint().parse().unwrap();
        let provider = ProviderBuilder::new().wallet(wallet).on_http(rpc_url);

        let agent = Agent::builder("deployer")
            .with_behavior(Deployer);
            // .build(Arc::new(provider), messager)
            // .unwrap();

        let mut world = World::new("id");

        world.add_agent(agent);

        world.run().await;
    }
}
