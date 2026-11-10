// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract FeeSplitter {
    address public treasury;
    address public ops;
    uint256 public treasuryBps;

    event Split(uint256 amount, uint256 toTreasury, uint256 toOps);

    constructor(address _treasury, address _ops, uint256 _treasuryBps) {
        require(_treasuryBps <= 10_000, "bps");
        treasury = _treasury;
        ops = _ops;
        treasuryBps = _treasuryBps;
    }

    receive() external payable { _split(msg.value); }
    function split() external payable { _split(msg.value); }

    function _split(uint256 amount) internal {
        if (amount == 0) return;
        uint256 toTreasury = (amount * treasuryBps) / 10_000;
        (bool a, ) = treasury.call{value: toTreasury}("");
        require(a, "treasury");
        (bool b, ) = ops.call{value: amount - toTreasury}("");
        require(b, "ops");
        emit Split(amount, toTreasury, amount - toTreasury);
    }
}
