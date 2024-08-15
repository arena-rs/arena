// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "solmate/tokens/ERC20.sol";
import "solmate/utils/FixedPointMathLib.sol";
import "./ArenaToken.sol";

/**
 * @dev Implementation of the test interface for Arbiter writing contracts.
 */
contract LiquidExchange {
    using FixedPointMathLib for int256;
    using FixedPointMathLib for uint256;

    address public arenaTokenX;
    address public arenaTokenY;

    uint256 public price;
    uint256 constant WAD = 10 ** 18;

    constructor(address arenaTokenX_, address arenaTokenY_, uint256 price_) {
        arenaTokenX = arenaTokenX_;
        arenaTokenY = arenaTokenY_;

        price = price_;
    }

    event PriceChange(uint256 price);
    event Swap(address tokenIn, address tokenOut, uint256 amountIn, uint256 amountOut, address to);

    function setPrice(uint256 _price) public {
        price = _price;
        emit PriceChange(price);
    }

    function swap(address tokenIn, uint256 amountIn) public {
        uint256 amountOut;
        address tokenOut;

        if (tokenIn == arenaTokenX) {
            tokenOut = arenaTokenY;
            amountOut = FixedPointMathLib.mulWadDown(amountIn, price);
        } else if (tokenIn == arenaTokenY) {
            tokenOut = arenaTokenX;
            amountOut = FixedPointMathLib.divWadDown(amountIn, price);
        } else {
            revert("Invalid token");
        }

        require(ERC20(tokenIn).transferFrom(msg.sender, address(this), amountIn), "Transfer failed");
        require(ERC20(tokenOut).transfer(msg.sender, amountOut), "Transfer failed");

        emit Swap(tokenIn, tokenOut, amountIn, amountOut, msg.sender);
    }
}
