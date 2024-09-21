pragma solidity ^0.8.10;

import {PoolManager} from "v4-core/PoolManager.sol";
import {PoolModifyLiquidityTest} from "v4-core/test/PoolModifyLiquidityTest.sol";
import {ArenaToken} from "./ArenaToken.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {Currency} from "v4-core/types/Currency.sol";
import {IHooks} from "v4-core/interfaces/IHooks.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {LiquidExchange} from "./LiquidExchange.sol";
import {Fetcher} from "./Fetcher.sol";
import {FullMath} from "v4-core/libraries/FullMath.sol";
import {SqrtPriceMath} from "v4-core/libraries/SqrtPriceMath.sol";
import {PoolSwapTest} from "v4-core/test/PoolSwapTest.sol";
import {TickMath} from "v4-core/libraries/TickMath.sol";
import {PoolId} from "v4-core/types/PoolId.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {Position} from "v4-core/libraries/Position.sol";

contract ArenaController {
    PoolManager immutable poolManager;
    PoolModifyLiquidityTest immutable router;
    PoolSwapTest immutable swapRouter;
    LiquidExchange immutable lex;
    Fetcher immutable fetcher;

    ArenaToken immutable currency0;
    ArenaToken immutable currency1;

    PoolKey public poolKey;

    uint256 internal constant MAX_SWAP_FEE = 1e6;

    uint160 public constant MIN_PRICE_LIMIT = TickMath.MIN_SQRT_PRICE + 1;
    uint160 public constant MAX_PRICE_LIMIT = TickMath.MAX_SQRT_PRICE - 1;

    bytes32 public constant POOLS_SLOT = bytes32(uint256(6));
    uint256 public constant POSITIONS_OFFSET = 6;

    struct Signal {
        int24 currentTick;
        uint160 sqrtPriceX96;
        address manager;
        uint256 lexPrice;
        PoolKey pool;
        address fetcher;
    }

    constructor(uint256 fee, uint256 initialPrice) {
        poolManager = new PoolManager(fee);
        router = new PoolModifyLiquidityTest(poolManager);
        swapRouter = new PoolSwapTest(poolManager);
        fetcher = new Fetcher();

        currency0 = new ArenaToken("currency0", "c0", 18);
        currency1 = new ArenaToken("currency1", "c1", 18);

        if (currency0 > currency1) {
            (currency0, currency1) = (currency1, currency0);
        }

        lex = new LiquidExchange(address(currency0), address(currency1), initialPrice);

        require(currency0.mint(address(this), 10000000000000000000), "Minting currency0 to liquid exchange failed");
        require(currency1.mint(address(this), 10000000000000000000), "Minting currency1 to liquid exchange failed");
    }

    function getRouter() external view returns (address) {
        return address(router);
    }

    function constructSignal() public view returns (Signal memory) {
        (uint160 sqrtPriceX96, int24 tick,,) = fetcher.getSlot0(poolManager, fetcher.toId(poolKey));

        return Signal({
            currentTick: tick,
            sqrtPriceX96: sqrtPriceX96,
            manager: address(poolManager),
            lexPrice: lex.price(),
            pool: poolKey,
            fetcher: address(fetcher)
        });
    }

    function setPrice(uint256 price) public {
        lex.setPrice(price);
    }

    function swapOnLex(address tokenIn, uint256 amountIn) public {
        lex.swap(tokenIn, amountIn);
    }

    function equalizePrice(int256 depth) public {
        require(currency0.approve(address(swapRouter), type(uint256).max), "Approval for currency0 failed");
        require(currency1.approve(address(swapRouter), type(uint256).max), "Approval for currency1 failed");

        (uint160 sqrtPriceX96,,,) = fetcher.getSlot0(poolManager, fetcher.toId(poolKey));
        
        uint256 uniswapPrice = FullMath.mulDiv(uint256(sqrtPriceX96) * 10**18, uint256(sqrtPriceX96), 1 << 192);
        uint256 lexPrice = lex.price();

        if (uniswapPrice > lexPrice) {
            bool zeroForOne = true;

            IPoolManager.SwapParams memory params = IPoolManager.SwapParams({
                zeroForOne: zeroForOne,
                amountSpecified: depth,
                sqrtPriceLimitX96: zeroForOne ? MIN_PRICE_LIMIT : MAX_PRICE_LIMIT // unlimited impact
            });

            PoolSwapTest.TestSettings memory testSettings =
                PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});

            swapRouter.swap(poolKey, params, testSettings, "");

        } else if (uniswapPrice < lexPrice) {
            bool zeroForOne = false;

            IPoolManager.SwapParams memory params = IPoolManager.SwapParams({
                zeroForOne: zeroForOne,
                amountSpecified: depth,
                sqrtPriceLimitX96: zeroForOne ? MIN_PRICE_LIMIT : MAX_PRICE_LIMIT // unlimited impact
            });

            PoolSwapTest.TestSettings memory testSettings =
                PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});

            swapRouter.swap(poolKey, params, testSettings, "");
        }
    }

    function setPool(uint24 poolFee, int24 tickSpacing, IHooks hooks, uint160 sqrtPriceX96, bytes memory hookData)
        public
    {
        poolKey = PoolKey({
            currency0: Currency.wrap(address(currency0)),
            currency1: Currency.wrap(address(currency1)),
            fee: poolFee,
            tickSpacing: tickSpacing,
            hooks: hooks
        });

        poolManager.initialize(poolKey, sqrtPriceX96, hookData);
    }

    function addLiquidity(int256 liquidityDelta, int24 tickLower, int24 tickUpper, bytes memory hookData) public {
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

        router.modifyLiquidity(poolKey, params, hookData);
    }
}
