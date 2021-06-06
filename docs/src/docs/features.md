# Features

This section describes some of the features introduced in Voyager and attempts to explain the reasoning behind them.

## Multiple Proposals / Non-Binary Voting

The proposal flow in the Sputnik DAO follows a rather simple pattern. An authorized proposer submits a proposal to their respective DAO. Voters then decide whether to approve it or reject it (or in rare occasions mark it as spam). Where this scheme falls apart is in the discussion phase. In many practical governance systems we see particpants, when a proposal is put forward, debate for and against there positions. After a while the proposal will be revised and put forth again. This negotiation helps to achieve proposals that a larger group align to.

Bringing this back to the Sputnik DAO, let's take for example a proposal to add a bounty. Bounty's look like the following:

```rust
pub struct Bounty {
    /// Description of the bounty.
    pub description: String,
    /// Token the bounty will be paid out.
    pub token: AccountId,
    /// Amount to be paid out.
    pub amount: U128,
    /// How many times this bounty can be done.
    pub times: u32,
    /// Max deadline from claim that can be spend on this bounty.
    pub max_deadline: WrappedDuration,
}
```

Say someone puts something forward and there is a disagreement between the payout amount. Two parties might discuss better payout amounts and even agree on a compromise - but this isn't able to be captured by the existing proposal framework. **Voyager** allows for counter proposals to be put forth, thus a proposal has several versions. 

```rust
    /// Adds a counter proposal to an existing one. Voters can only vote for one of these versions
    pub fn add_counter_proposal(&mut self, id: u64, proposal_input: ProposalInput) -> u8 {
```

When voters are ready, they can nominate a single version or reject all of them entirely. The first version to exceed the threshold then passes. 

```rust
pub struct Proposal {
    ...
    /// Count of approvals per proposal version.
    pub approve_count: Vec<Balance>,
    /// Count of rejections over the entire proposal.
    pub reject_count: Balance,
    /// Count of votes to remove a proposal version
    pub remove_count: Vec<Balance>,
    ...
}
```

Prior, tallies were kept for each role and the 

Counter proposals must be of the same `ProposalKind` (more information on how proposals are categorized in the next section). Votes are tracked 

## Repeat Votes

If we allow members to counter propose, to express their own perspective on the issue, then how does this affect voting. It is conceivable that a member votes on a proposal and then a later proposal is submitted that better captures the voters preference. To solve this issue we either create two separate phases, for proposing and voting or we allow a voter to then change their mind. While the former constrains how quick a proposal can be executed the later not only avoids that but gives greater flexbility generally to how a proposal may evolve from the discussions that surround it. Because of this, the VoyagerDAO has been modified to allow voters to vote again and again if need be. 

## Multi-messaged Proposals

The SputnikDAO categorises the possible typs of actions that can be executed as the `ProposalKind`. This can be, for example, to change a member or policy, add a bounty or execute any other contract. There may be situations however where a members wants to propose a set of actions. To accomodate this, VoyagerDAO uses an array to combine and types of `ProposalKind` together. Allowing to mix and match proposals, whilst offering greater flexibility introduces it's own set of challenges, namely how do we deicde what the actual `ProposalKind` is. To solve this, VoyagerDAO breaks down the prior concept into two. `Instruction`'s which are an array of actions that get executed if the proposal is accepted and `ProposalKind` which encapsulates how we treat an array of instructions.

```rust
pub fn propose(&mut self, description: String, instructions: Vec<Instruction>) -> u64 {
    let kind = self.internal_check_proposal(&instructions);
    ...
}
```

We thus define the `ProposalKind` as the following:

```rust
pub struct  ProposalKind {
    /// All proposals fall under this kind
    pub name: String,
    /// Proposal must have all of the following instructions within it to be considered
    /// part of this proposal kind. This information is thus used to decide whether a proposal
    /// matches this proposal kind
    required_instrs: Vec<InstructionKind>,
    /// the vote policy that get's associated
    pub vote_policy: VotePolicy,
}
```

An array of `ProposalKind` is set within the policy. The order of the array is important and must be from most restrictive to least restrictive. For a set of instructions to fall under that `ProposalKind`, all of the required instructions must be a subset of the instructions in the proposal. If no `ProposalKind` is matched, we fallback to the default `VotePolicy`

## Withdrawing and Amending Proposals

The final feature is designed to provide better user experience. There is always the possibility that throughout the proposal process that mistakes are made. VoyagerDAO adds the ability to withdraw proposals that at a later point seem unfit for the DAO or to amend proposals that have mistakes in the way that they were structured.  

In order to provide some degree of continuity there are a set of rules that come with withdrawing and amending proposal.
- Only the proposer can withdraw or amend proposals
- The proposer must have `WithdrawProposal` or `AmendProposal` permissions.
- The proposer can only perform these actions if no one has voted on the proposal yet.