# Instructions Specific to the Language Feature

## Objective

We are working on this PR: Adds comprehensive multi-language support to harper, enabling grammar and spell checking for German and Portuguese in addition to English. Introduces a modular language architecture with compressed dictionaries, dialect detection, and language-specific linters.

## How to Work:

- You can use the `gh` command line tool to access information on GitHub.
- The PR is number 3402. The issue that outlines the roadmap is 2654. You need only check that, if I tell you to. 
- Don't comment on issues or submit PRs, etc. using `gh` unless I specifically tell you to. 
- Ensure that `just check-rust` will work, after you changed something. 

## Git History

- I want that the work is based on the idea of the `portuges` branch, that was submitted by `dante-e`.
  The main concern was that we should not just use a giant enum for all Dialects, but use the more structured approach with Language enum. 
- I want everything to be based on the master branch (that's why I said "based on the _idea_ of the `portuges` branch").
  The portuges branch idea is already in one commit after the current master. 



