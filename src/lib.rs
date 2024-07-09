use std::{str::FromStr, sync::Arc};

use alloy::{
    primitives::{Address, Bytes, Uint, U256},
    sol,
};
use anyhow::Result;
use octane::{
    agent::Agent,
    machine::{Behavior, EventStream},
    messenger::{Messager, To},
    world::World,
    AnvilProvider,
};
use serde::{Deserialize, Serialize};

use crate::{
    bindings::{
        arenatoken::ArenaToken,
        poolmanager::{PoolManager, PoolManager::PoolKey},
    },
    deployer::DeploymentParams,
};

pub mod bindings;
pub mod deployer;
pub mod pool_admin;
