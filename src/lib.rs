use std::{str::FromStr, sync::Arc};

use alloy::{
    primitives::{Address, Bytes, Uint, U256},
    sol,
};
use anyhow::Result;
use octane::{
    agent::Agent,
    machine::{Behavior, ControlFlow, EventStream},
    messenger::{Message, Messager, To},
    world::World,
    AnvilProvider,
};
use serde::{Deserialize, Serialize};

use crate::{
    bindings::{
        arenatoken::ArenaToken,
        liquidexchange::LiquidExchange,
        poolmanager::{PoolManager, PoolManager::PoolKey},
    },
    deployer::{DeploymentRequest, DeploymentResponse},
};

pub mod bindings;
pub mod deployer;
pub mod pool_admin;
