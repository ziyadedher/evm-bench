// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.17;

contract TenThousandHashes {
    function Benchmark() external pure {
        for (uint256 i = 0; i < 5000; i++) {
            keccak256(abi.encodePacked(i));
        }
        for (uint256 i = 0; i < 2500; i++) {
            ripemd160(abi.encodePacked(i));
        }
        for (uint256 i = 0; i < 2500; i++) {
            sha256(abi.encodePacked(i));
        }
    }
}
