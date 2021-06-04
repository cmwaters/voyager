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

Say someone puts something forward and there is a disagreement between the payout amount. Two parties might discuss better payout amounts and even agree on a compromise - but this isn't able to be captured by the existing proposal framework. **Voyager** allows for counter proposals to be put forth, thus a proposal has several versions. When voters are ready, they can nominate a single version or reject all of them entirely. The first version to exceed the threshold then passes. 