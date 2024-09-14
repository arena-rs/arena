pragma solidity ^0.8.10;

import {PoolManager} from "v4-core/PoolManager.sol";
import {PoolModifyLiquidityTest} from "v4-core/test/PoolModifyLiquidityTest.sol";
import {ArenaToken} from "src/ArenaToken.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {Currency} from "v4-core/types/Currency.sol";
import {IHooks} from "v4-core/interfaces/IHooks.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";

contract Test {
    function test_liquidityAdd() public {
        PoolManager poolManager = new PoolManager(0);
        PoolModifyLiquidityTest router = new PoolModifyLiquidityTest(poolManager);

        ArenaToken currency0 = new ArenaToken("currency0", "c0", 18);
        ArenaToken currency1 = new ArenaToken("currency1", "c1", 18);

        currency0.mint(address(this), type(uint256).max);
        currency1.mint(address(this), type(uint256).max);

        currency0.approve(address(router), type(uint256).max);
        currency1.approve(address(router), type(uint256).max);

        if (currency0 > currency1) {
            (currency0, currency1) = (currency1, currency0);
        }

        PoolKey memory poolKey = PoolKey ({
            currency0: Currency.wrap(address(currency0)),
            currency1: Currency.wrap(address(currency1)),
            fee: 4000,
            tickSpacing: 2,
            hooks: IHooks(address(0))
        });

        // Represents a 1:1 ratio of assets in the pool.
        poolManager.initialize(poolKey, 7922816251426433543950336, "");

        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams ({
            tickLower: -20,
            tickUpper: 20,
            liquidityDelta: 10000000000,
            salt: ""
        });

        router.modifyLiquidity(poolKey, params, "");
    }
}