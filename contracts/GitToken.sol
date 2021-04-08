// SPDX-License-Identifier: MIT
pragma solidity >=0.4.25 <0.7.0;

import "./ConvertLib.sol";

// This is just a simple example of a coin-like contract.
// It is not standards compatible and cannot be expected to talk to other
// coin/token contracts. If you want to create a standards-compliant
// token, see: https://github.com/ConsenSys/Tokens. Cheers!

contract GitToken {
    mapping(address => uint256) balances;
    address[] public users;

    address public owner;
    uint256 public totalSupply;

    event Transfer(address indexed _from, address indexed _to, uint256 _value);

    constructor(uint256 _initialAmount) public {
        balances[msg.sender] = _initialAmount;
        owner = msg.sender;
        totalSupply = _initialAmount;
        users.push(tx.origin);
    }

    function givePoint(address receiver, uint256 amount)
        public
        returns (bool sufficient)
    {
        if (balances[msg.sender] < amount) return false;
        balances[msg.sender] -= amount;
        balances[receiver] += amount;
        emit Transfer(msg.sender, receiver, amount);
        return true;
    }

    function getBalance(address addr) public view returns (uint256) {
        return balances[addr];
    }

    function addUser(address newUser) public {
        require(msg.sender == owner, "only owners can reset");
        fraction = balances[msg.sender] * 0.01;
        balances[msg.sender] -= fraction;
        balances[newUser] += fraction;
        emit Transfer(msg.sender, addr, equal_amount);
    }
}