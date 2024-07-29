# arena 🏟️

![visitors](https://visitor-badge.laobi.icu/badge?page_id=arena-rs.arena)
[![Twitter Badge](https://badgen.net/badge/icon/twitter?icon=twitter&label)](https://twitter.com/anthiasxyz)
![image](https://github.com/arena-rs/.github/blob/main/arena_banner.png)

Arena is a powerful and extensible framework for holistic economic modelling and simulation of Uniswap v4 strategies, hooks and pools.

Track how metrics evolve over time, and over various market conditions.

## Overview

Arena uses a modified version of [Arbiter](https://github.com/primitivefinance/arbiter) engine called [octane](https://github.com/arena-rs/octane), which alloys the usage of [alloy](https://alloy.rs/) and anvil for simulation. An agent-oriented architecture is utilized.

- [x] Deployer agent.
- [x] Pool admin agent.
- [ ] Arbitrageur agent.  

## Initial goals
- Simulate fee accumulation for a stable pool across various market volatilities using an Ornstein-Uhlenbeck process and agentic modelling.
- Geometric Brownian motion for a non-stable pool.
- Provide a set of reusable, extensible and modular types that allow any liquidity provision strategy to be defined, and any market condition.
- Command line interface for plug-and-play analytics and simulation.
