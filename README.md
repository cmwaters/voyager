# Voyager DAO

Voyager is an extension of the [Sputnik DAO](https://github.com/near-daos/sputnik-dao-contract) repository. It is a participant of the [Open Web Governance Challenge](https://metagov.github.io/open-web-challenge/) and the codebase for the [Improve Sputnik Framework Proposal](https://gov.near.org/t/proposal-improve-sputnik-framework/2202). 

## Documentation

For documentation on the changes made and a walkthough of how to use them visit the [docs](https://cmwaters.github.io/voyager/docs/introduction.html)

## Changelog

This section records changes made between [SputnikDAO](https://github.com/near-daos/sputnik-dao-contract) and Voyager (this repository).

- Allows for a proposer to withdraw a proposal so long as no other member has voted for it yet. 
- Any valid proposer can propose an alternative version of an existing proposal so long as it remains within the same `ProposalKind`. Voters can thus choose which of the proposals they want to vote on.
- `ProposalKind` has been renamed to `Instruction`. A proposal can (in most cases) consist of multiple `Instructions`'s. The DAO's policy dicates how proposals get categorized. This is done by defining many `ProposalKind`s and for each one state the required messages that a proposal must have to match that particular kind.
- Voting is no longer tallied per each role that has voting rights to that proposal. Instead, for weighted proposals, there is a simple counter that counts the approvals given to each version. In role-based elections, we add a single vote for each role that a user is a member of that is also valid based from the proposal.