// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

/// This is an example of an problematic comment.
/// It should produce one error.
contract Test {}

/**
 * This is an example of a possible error:
 * these subsequent lines should not be considered a new sentence and should
 * produce no errors.
 */
library FooBar {}

/// Let's aadd a cuple spelling errors for good measure.
