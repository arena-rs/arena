// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {PoolModifyLiquidityTest} from "v4-core/test/PoolModifyLiquidityTest.sol";

contract LiquidityProvider {
    PoolModifyLiquidityTest immutable lpRouter;

    constructor(IPoolManager _manager) {
        lpRouter = new PoolModifyLiquidityTest(_manager);
    }

    function createLiquidity(
        PoolKey memory poolKey,
        int24 tickLower,
        int24 tickUpper,
        int256 liquidity,
        bytes calldata hookData
    ) external {
        // if 0 < liquidity: add liquidity -- otherwise remove liquidity
        lpRouter.modifyLiquidity(
            poolKey,
            IPoolManager.ModifyLiquidityParams({
                tickLower: tickLower,
                tickUpper: tickUpper,
                liquidityDelta: liquidity,
                salt: ""
            }),
            hookData
        );
    }
}
