use std::{fmt, str::FromStr, sync::Arc};

use alloy::{
    primitives::{Address, Bytes, Uint, U256},
    sol,
};
use anyhow::Result;
use futures::stream::StreamExt;
use octane::{
    agent::Agent,
    machine::{Behavior, ControlFlow, EventStream},
    messenger::{Message, Messager, To},
    world::World,
    AnvilProvider,
};
use serde::{Deserialize, Serialize};
use RustQuant::{
    models::*,
    stochastics::{process::Trajectories, *},
};

use crate::{
    bindings::{
        arenatoken::ArenaToken,
        liquidexchange::LiquidExchange,
        poolmanager::{PoolManager, PoolManager::PoolKey},
    },
    deployer::{DeploymentRequest, DeploymentResponse},
    pool_admin::PoolParams,
    price_changer::PriceUpdate,
};

pub mod arbitrageur;
pub mod bindings;
pub mod deployer;
pub mod pool_admin;
pub mod price_changer;
