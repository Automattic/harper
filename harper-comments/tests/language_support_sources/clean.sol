// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

/// This is an example Solidity file that should produce no Harper lints.
contract TestContract {
    /// This is a test function.
    /// It has a [link](https://example.com) embedded inside
    function testFunction() external {}

    /**
     * @notice This is another test function.
     * @dev It has another [link](https://example.com) embedded inside
     * @param p This is a parameter
     */
    function testFunction2(uint256 p) external {}

    // This is some gibberish to try to trigger a lint for sentences that continue for too long
    //
    // This is some gibberish to try to trigger a lint for sentences that continue for too long
    //
    // This is some gibberish to try to trigger a lint for sentences that continue for too long
    //
    // This is some gibberish to try to trigger a lint for sentences that continue for too long
    //
    // This is some gibberish to try to trigger a lint for sentences that continue for too long
}
