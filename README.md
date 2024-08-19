# arena 🏟️

![visitors](https://visitor-badge.laobi.icu/badge?page_id=arena-rs.arena)
[![Twitter Badge](https://badgen.net/badge/icon/twitter?icon=twitter&label)](https://twitter.com/anthiasxyz)
![image](https://github.com/arena-rs/.github/blob/main/arena_banner.png)

> *Arena is a powerful and extensible framework for holistic economic modelling and simulation of Uniswap v4 strategies, hooks and pools.*

Track how metrics evolve over time, and over various market conditions.

## Overview

Arena introduces a novel approach to LP simululation through a highly-configurable event-driven runtime. Each event consists of integral market information for a strategy, from which the actor can derive insight from.

Arena is an [alloy](https://alloy.rs) native project, utilizing many crate-native features such as the `sol!` procedural macro, and the `Anvil` testnet node implementation.

## Technical details

Every LP strategy must implement the `Strategy` trait. This contains two key methods:
- `init()` is called upon initialization of the Arena runtime.
- `process()` is called each discrete timestep of the simulation.

Additionally, each LP strategy accepts an `Inspector`. An `Inspector` allows custom behavior to be defined for performance analysis of strategy and continuous telemetry. Arena will provide default `Inspector` implementations for CSV output, graph plotting and JSON output. 

The runtime can hold multiple strategies in paralell.

The price of the simulation's underlying pool is set via a price process that implements the `Feed` trait. Currently, Arena implements an Ornstein-Uhlenbeck price process using a Euler-Maryama discretization scheme for stable pool simulation. The price is set on an infinitely liquid exchange (sometimes referred to as a "lex"), and tied to the v4 pool using an `Arbitrageur`.

## Usage

To use Arena, the Rust programming language alongside the Foundry framework must be installed on your machine. This is commonly achieved using [`rustup`](https://rustup.rs/), and [`foundryup`](https://book.getfoundry.sh/getting-started/installation)

Arena can be added to your library or binary with 
```
cargo add arena-core
```

If you wish to build from source, the project can be cloned with:
```
git clone https://github.com/arena-rs/arena.git
cd arena
git submodule update --init --recursive
```