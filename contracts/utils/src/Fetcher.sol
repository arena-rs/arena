pragma solidity ^0.8.10;

import {PoolId} from "v4-core/types/PoolId.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {Position} from "v4-core/libraries/Position.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";

// This contract is a compact version of StateLibrary, which can be found below.
// https://github.com/Uniswap/v4-core/blob/799dd2cb980319a8d3b827b6a7aa59a606634553/src/libraries/StateLibrary.sol
contract Fetcher {
    bytes32 public constant POOLS_SLOT = bytes32(uint256(6));

    function _getPoolStateSlot(PoolId poolId) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(PoolId.unwrap(poolId), POOLS_SLOT));
    }

    function toId(PoolKey memory poolKey) external pure returns (PoolId poolId) {
        assembly ("memory-safe") {
            poolId := keccak256(poolKey, mul(32, 5))
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
}