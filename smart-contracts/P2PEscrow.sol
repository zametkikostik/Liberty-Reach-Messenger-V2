// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Liberty Messenger P2P Escrow (3% protocol fee)
contract P2PEscrow {
    address public feeRecipient;
    uint256 public constant FEE_BPS = 300;

    enum Status { None, Funded, Released, Refunded }

    struct Deal {
        address buyer;
        address seller;
        uint256 amount;
        Status status;
    }

    mapping(bytes32 => Deal) public deals;
    uint256 public dealCount;

    event DealCreated(bytes32 indexed id, address buyer, address seller, uint256 amount);
    event DealReleased(bytes32 indexed id);
    event DealRefunded(bytes32 indexed id);

    constructor(address _feeRecipient) {
        feeRecipient = _feeRecipient;
    }

    function createDeal(address seller) external payable returns (bytes32 id) {
        require(msg.value > 0, "no value");
        require(seller != address(0) && seller != msg.sender, "bad seller");
        id = keccak256(abi.encodePacked(msg.sender, seller, msg.value, dealCount++, block.timestamp));
        deals[id] = Deal(msg.sender, seller, msg.value, Status.Funded);
        emit DealCreated(id, msg.sender, seller, msg.value);
    }

    function release(bytes32 id) external {
        Deal storage d = deals[id];
        require(d.status == Status.Funded, "not funded");
        require(msg.sender == d.buyer || msg.sender == d.seller, "not party");
        d.status = Status.Released;
        uint256 fee = (d.amount * FEE_BPS) / 10_000;
        (bool ok1, ) = d.seller.call{value: d.amount - fee}("");
        require(ok1, "seller");
        (bool ok2, ) = feeRecipient.call{value: fee}("");
        require(ok2, "fee");
        emit DealReleased(id);
    }

    function refund(bytes32 id) external {
        Deal storage d = deals[id];
        require(d.status == Status.Funded, "not funded");
        require(msg.sender == d.seller, "only seller");
        d.status = Status.Refunded;
        (bool ok, ) = d.buyer.call{value: d.amount}("");
        require(ok, "refund");
        emit DealRefunded(id);
    }
}
