pragma solidity ^0.8.10;

import "solmate/tokens/ERC20.sol";

contract ArenaToken is ERC20 {
    constructor(string memory name, string memory symbol, uint8 decimals) ERC20(name, symbol, decimals) {}

    function mint(address receiver, uint256 amount) public returns (bool) {
        _mint(receiver, amount);
        return true;
    }
}
