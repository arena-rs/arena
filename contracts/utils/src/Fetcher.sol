pragma solidity ^0.8.10;

import {PoolId} from "v4-core/types/PoolId.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {Position} from "v4-core/libraries/Position.sol";

// This contract is a compact version of StateLibrary, which can be found below.
// https://github.com/Uniswap/v4-core/blob/799dd2cb980319a8d3b827b6a7aa59a606634553/src/libraries/StateLibrary.sol
contract Fetcher {
    bytes32 public constant POOLS_SLOT = bytes32(uint256(6));
    uint256 public constant TICKS_OFFSET = 4;

    function _getPoolStateSlot(PoolId poolId) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(PoolId.unwrap(poolId), POOLS_SLOT));
    }

    function toId(PoolKey memory poolKey) external pure returns (PoolId poolId) {
        assembly ("memory-safe") {
            poolId := keccak256(poolKey, mul(32, 5))
        }
    }

    function getTickInfo(IPoolManager manager, PoolId poolId, int24 tick)
        external
        view
        returns (
            uint128 liquidityGross,
            int128 liquidityNet,
            uint256 feeGrowthOutside0X128,
            uint256 feeGrowthOutside1X128
        )
    {
        bytes32 slot = _getTickInfoSlot(poolId, tick);

        // read all 3 words of the TickInfo struct
        bytes32[] memory data = manager.extsload(slot, 3);
        assembly ("memory-safe") {
            let firstWord := mload(add(data, 32))
            liquidityNet := sar(128, firstWord)
            liquidityGross := and(firstWord, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
            feeGrowthOutside0X128 := mload(add(data, 64))
            feeGrowthOutside1X128 := mload(add(data, 96))
        }
    }

    function getSlot0(IPoolManager manager, PoolId poolId)
        external
        view
        returns (uint160 sqrtPriceX96, int24 tick, uint24 protocolFee, uint24 lpFee)
    {
        // slot key of Pool.State value: `pools[poolId]`
        bytes32 stateSlot = _getPoolStateSlot(poolId);

        bytes32 data = manager.extsload(stateSlot);

        //   24 bits  |24bits|24bits      |24 bits|160 bits
        // 0x000000   |000bb8|000000      |ffff75 |0000000000000000fe3aa841ba359daa0ea9eff7
        // ---------- | fee  |protocolfee | tick  | sqrtPriceX96
        assembly ("memory-safe") {
            // bottom 160 bits of data
            sqrtPriceX96 := and(data, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
            // next 24 bits of data
            tick := signextend(2, shr(160, data))
            // next 24 bits of data
            protocolFee := and(shr(184, data), 0xFFFFFF)
            // last 24 bits of data
            lpFee := and(shr(208, data), 0xFFFFFF)
        }
    }

    function _getTickInfoSlot(PoolId poolId, int24 tick) internal pure returns (bytes32) {
        // slot key of Pool.State value: `pools[poolId]`
        bytes32 stateSlot = _getPoolStateSlot(poolId);

        // Pool.State: `mapping(int24 => TickInfo) ticks`
        bytes32 ticksMappingSlot = bytes32(uint256(stateSlot) + TICKS_OFFSET);

        // slot key of the tick key: `pools[poolId].ticks[tick]
        return keccak256(abi.encodePacked(int256(tick), ticksMappingSlot));
    }
}
