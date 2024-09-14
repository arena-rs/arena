pragma solidity ^0.8.10;

import {PoolManager} from "v4-core/PoolManager.sol";
import {PoolModifyLiquidityTest} from "v4-core/test/PoolModifyLiquidityTest.sol";
import {ArenaToken} from "./ArenaToken.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {Currency} from "v4-core/types/Currency.sol";
import {IHooks} from "v4-core/interfaces/IHooks.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";

contract ArenaController {
    PoolManager immutable poolManager;
    PoolModifyLiquidityTest immutable router;

    ArenaToken immutable currency0;
    ArenaToken immutable currency1;

    PoolKey public poolKey;

    constructor(uint256 fee) {
        poolManager = new PoolManager(fee);
        router = new PoolModifyLiquidityTest(poolManager);

        currency0 = new ArenaToken("currency0", "c0", 18);
        currency1 = new ArenaToken("currency1", "c1", 18);

        if (currency0 > currency1) {
            (currency0, currency1) = (currency1, currency0);
        }
    }

    function setPool(uint24 poolFee, int24 tickSpacing, IHooks hooks, uint160 sqrtPriceX96, bytes memory hookData) public {
        poolKey = PoolKey ({
            currency0: Currency.wrap(address(currency0)),
            currency1: Currency.wrap(address(currency1)),
            fee: poolFee,
            tickSpacing: tickSpacing,
            hooks: hooks
        });

        poolManager.initialize(poolKey, sqrtPriceX96, hookData);
    }

    function addLiquidity(int256 liquidityDelta, int24 tickLower, int24 tickUpper) public {
        if (liquidityDelta > 0) {
            require(currency0.mint(address(this), uint256(liquidityDelta)), "Minting currency0 failed");
            require(currency1.mint(address(this), uint256(liquidityDelta)), "Minting currency1 failed");
        }

        require(currency0.approve(address(router), type(uint256).max), "Approval for currency0 failed");
        require(currency1.approve(address(router), type(uint256).max), "Approval for currency1 failed");

        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams({
            tickLower: tickLower,
            tickUpper: tickUpper,
            liquidityDelta: liquidityDelta,
            salt: ""
        });

        router.modifyLiquidity(poolKey, params, "");
    }
}