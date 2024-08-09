// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {PoolManager} from "v4-core/PoolManager.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {LiquidityProvider} from "../src/LiquidityProvider.sol";
import {ArenaToken} from "../src/ArenaToken.sol";
import {IHooks} from "v4-core/interfaces/IHooks.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {CurrencyLibrary, Currency} from "v4-core/types/Currency.sol";
import "forge-std/Test.sol";

contract LiquidityProviderTest is Test {
    PoolManager public manager;
    LiquidityProvider public lpRouter;
    ArenaToken public token0;
    ArenaToken public token1;

    function test_createLiquidity() external {
        vm.startPrank(address(0x1));  // Set caller one's address

        manager = new PoolManager(5000);
        lpRouter = new LiquidityProvider(manager);

        token0 = new ArenaToken("Token0", "T0", 18);
        token1 = new ArenaToken("Token1", "T1", 18);

        if (address(token0) > address(token1)) {
            (token0, token1) = (token1, token0);
        }

        PoolKey memory pool = PoolKey({
            currency0: Currency.wrap(address(token0)),
            currency1: Currency.wrap(address(token1)),
            fee: 3000,
            tickSpacing: 60,
            hooks: IHooks(address(0x0))
        });

        manager.initialize(pool, 79228162514264337593543950336, "");

        vm.stopPrank();
        vm.startPrank(address(0x2));

        token0.mint(2 ** 255);
        token1.mint(2 ** 255);

        token0.approve(address(lpRouter), type(uint256).max);
        token1.approve(address(lpRouter), type(uint256).max);

        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams({
            tickLower: -120,
            tickUpper: 120,
            liquidityDelta: 1e18,
            salt: ""
        });

        lpRouter.modifyLiquidity(pool, params, "");

        vm.stopPrank();
    }
}