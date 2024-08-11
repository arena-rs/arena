use alloy_sol_macro::sol;

sol! {
    type Currency is address;

    #[derive(Debug)]
    struct PoolKey {
        Currency currency0;
        Currency currency1;
        uint24 fee;
        int24 tickSpacing;
        address hooks;
    }
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    PoolManager,
    "contracts/v4-core/out/PoolManager.sol/PoolManager.json"
}
