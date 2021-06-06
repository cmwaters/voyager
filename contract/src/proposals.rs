use std::collections::{HashMap};
use std::u128;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, WrappedTimestamp, U64};
use near_sdk::{AccountId, Balance, PromiseOrValue};

use crate::policy::{UserInfo, WeightKind};
use crate::types::{
    upgrade_remote, upgrade_self, Action, Config, BASE_TOKEN, GAS_FOR_FT_TRANSFER, ONE_YOCTO_NEAR,
};
use crate::*;

/// Proposal kind is a means of distinguishing between different types of 
/// proposals based on the kinds of instructions that are included in a proposal
/// The ability to categorize proposals helps define the purpose of roles and
/// allows for different vote policies.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
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

impl ProposalKind {
    pub fn match_proposal(&self, instructions: &Vec<Instruction>) -> bool {
        let instruction_kind: Vec<InstructionKind> = instructions.into_iter().map(|i| i.to_enum()).collect();
        for instr in self.required_instrs.iter() {
            if !instruction_kind.contains(instr) {
                return false
            }
        }
        true
    }
}

/// Status of a proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    InProgress,
    /// If quorum voted yes, one of the versions of the proposal was successfully approved.
    Approved{ version: u8 },
    /// If quorum voted no, this proposal is rejected. Bond is returned.
    Rejected,
    /// Expired after period of time.
    Expired,
    /// If proposal was moved to Hub or somewhere else.
    Moved,
}

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionCall {
    method_name: String,
    args: Base64VecU8,
    deposit: U128,
    gas: U64,
}

/// Instruction is an action that may be executed when a proposal is approved.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum Instruction {
    /// Change the DAO config.
    ChangeConfig { config: Config },
    /// Change the full policy.
    ChangePolicy { policy: VersionedPolicy },
    /// Add member to given role in the policy. This is short cut to updating the whole policy.
    AddMemberToRole { member_id: AccountId, role: String },
    /// Remove member to given role in the policy. This is short cut to updating the whole policy.
    RemoveMemberFromRole { member_id: AccountId, role: String },
    /// Calls `receiver_id` with list of method names in a single promise.
    /// Allows this contract to execute any arbitrary set of actions in other contracts.
    FunctionCall {
        receiver_id: AccountId,
        actions: Vec<ActionCall>,
    },
    /// Upgrade this contract with given hash from blob store.
    UpgradeSelf { hash: Base58CryptoHash },
    /// Upgrade another contract, by calling method with the code from given hash from blob store.
    UpgradeRemote {
        receiver_id: AccountId,
        method_name: String,
        hash: Base58CryptoHash,
    },
    /// Transfers given amount of `token_id` from this DAO to `receiver_id`.
    Transfer {
        token_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    },
    /// Sets staking contract. Can only be proposed if staking contract is not set yet.
    SetStakingContract { staking_id: AccountId },
    /// Add new bounty.
    AddBounty { bounty: Bounty },
    /// Indicates that given bounty is done by given user.
    BountyDone {
        bounty_id: u64,
        receiver_id: AccountId,
    },
    /// Just a signaling vote, with no execution.
    Vote,
}

pub type InstructionKind = u8;

impl Instruction {
    /// Returns label of policy for given type of proposal.
    pub fn to_enum(&self) -> InstructionKind {
        match self {
            Instruction::ChangeConfig { .. } => 0,
            Instruction::ChangePolicy { .. } => 1,
            Instruction::AddMemberToRole { .. } => 2,
            Instruction::RemoveMemberFromRole { .. } => 3,
            Instruction::FunctionCall { .. } => 4,
            Instruction::UpgradeSelf { .. } => 5,
            Instruction::UpgradeRemote { .. } => 6,
            Instruction::Transfer { .. } => 7,
            Instruction::SetStakingContract { .. } => 8,
            Instruction::AddBounty { .. } => 9,
            Instruction::BountyDone { .. } => 10,
            Instruction::Vote => 11,
        }
    }
}

/// Votes recorded in the proposal. Votes can be for any proposal within a
/// proposal topic. A vote of 0
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Vote {
    pub choice: u8,
    pub weight: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct RemoveVote {
    pub account_id: AccountId,
    pub version: u8,
}

/// Proposal that are sent to this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    /// Kind of proposal.
    pub kind: String,
    /// Lists all proposals within the same topic
    pub versions: Vec<ProposalVersion>,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of approvals per proposal version.
    pub approve_count: Vec<Balance>,
    /// Count of rejections over the entire proposal.
    pub reject_count: Balance,
    /// Count of votes to remove a proposal version
    pub remove_count: Vec<Balance>,
    /// Flag to indicate the removal of a proposal
    pub remove_flag: Vec<bool>,
    /// Map of who voted to prevent multiple voting
    pub votes: HashMap<AccountId, Vote>,
    /// Hashset of remove votes to prevent multiple voting
    pub remove_votes: Vec<RemoveVote>,
    /// Submission time (for voting period).
    pub submission_time: WrappedTimestamp,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalVersion {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Instructions to be executed if proposal is approved.
    pub instructions: Vec<Instruction>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedProposal {
    Default(Proposal),
}

impl From<VersionedProposal> for Proposal {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::Default(p) => p,
        }
    }
}

impl Proposal {
    /// Adds vote of the given user with given `amount` of weight. If user already voted, fails.
    pub fn update_votes(
        &mut self,
        account_id: &AccountId,
        vote: Vote,
        vote_policy: &VotePolicy,
        threshold: u128
    ) -> ProposalStatus {
        // add vote to tally and check previous votes
        let old_vote = self.votes.insert(account_id.clone(), vote.clone());
        // if the voter previously voted then revert it
        let mut weight = vote.weight;
        if old_vote.is_some() {
            let v = old_vote.unwrap();
            if v.choice == 0 {
                self.reject_count -= v.weight
            } else {
                self.approve_count[(v.choice - 1) as usize] += v.weight
            }
            // set the weight to what it was when the user first voted
            weight = v.weight
        }

        assert!(!self.remove_flag[(vote.choice - 1) as usize], "ERR_PROPOSAL_REMOVED");

        // version should have already been vetted
        if vote.choice == 0 {
            self.reject_count -= weight;
        } else {
            self.approve_count[(vote.choice - 1) as usize] += weight;
        }

        if self.reject_count >= threshold {
            return ProposalStatus::Rejected
        }
        
        if self.approve_count[(vote.choice - 1) as usize] >= threshold {
            return ProposalStatus::Approved{ version: (vote.choice - 1) }
        }

        ProposalStatus::InProgress
    }

    pub fn update_remove_votes(
        &mut self, 
        vote: RemoveVote, 
        weight: u128, 
        threshold: u128
    ) -> bool {
        assert!(vote.version < self.versions.len() as u8, "ERR_NO_PROPOSAL_VERSION");
        for v in self.remove_votes.iter() {
            assert!(v != &vote, "ERR_ALREADY_VOTED")
        }
        self.remove_count[vote.version as usize] += weight;
        if self.remove_count[vote.version as usize] >= threshold {
            self.remove_votes.push(vote);
            return true
        }
        self.remove_votes.push(vote);
        false
    }

    pub fn create_vote(&self, 
        vote_policy: &VotePolicy, 
        choice: u8, 
        user_weight: Balance
    ) -> Vote {
        assert!(choice <= self.versions.len() as u8, "ERR_NO_PROPOSAL_VERSION");
        // calculate the weight of the vote
        let weight = match vote_policy.weight_kind {
            WeightKind::TokenWeight => user_weight,
            WeightKind::RoleWeight => 1,
        };
        Vote {
            choice,
            weight,
        }
    }
}

impl Contract {
    /// Execute payout of given token to given user.
    pub(crate) fn internal_payout(
        &mut self,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
    ) -> PromiseOrValue<()> {
        if token_id == BASE_TOKEN {
            Promise::new(receiver_id.clone()).transfer(amount).into()
        } else {
            ext_fungible_token::ft_transfer(
                receiver_id.clone(),
                U128(amount),
                None,
                &token_id,
                ONE_YOCTO_NEAR,
                GAS_FOR_FT_TRANSFER,
            )
            .into()
        }
    }

    /// Executes given proposal and updates the contract's state.
    fn internal_execute_proposal(
        &mut self,
        policy: &Policy,
        proposal: &Proposal,
        version: &ProposalVersion,
    ) {
        // Return the proposal bond to all proposers.
        for p in proposal.versions.iter() {
            Promise::new(p.proposer.clone()).transfer(policy.proposal_bond.0);
        }
        // execute instructions in order of proposal
        for instr in &version.instructions {
            match instr {
                Instruction::ChangeConfig { config } => {
                    self.config.set(config);
                }
                Instruction::ChangePolicy { policy } => {
                    self.policy.set(policy);
                }
                Instruction::AddMemberToRole { member_id, role } => {
                    let mut new_policy = policy.clone();
                    new_policy.add_member_to_role(role, member_id);
                    self.policy.set(&VersionedPolicy::Current(new_policy));
                }
                Instruction::RemoveMemberFromRole { member_id, role } => {
                    let mut new_policy = policy.clone();
                    new_policy.remove_member_from_role(role, member_id);
                    self.policy.set(&VersionedPolicy::Current(new_policy));
                }
                Instruction::FunctionCall {
                    receiver_id,
                    actions,
                } => {
                    let mut promise = Promise::new(receiver_id.clone());
                    for action in actions {
                        promise = promise.function_call(
                            action.method_name.clone().into_bytes(),
                            action.args.clone().into(),
                            action.deposit.0,
                            action.gas.0,
                        )
                    }
                }
                Instruction::UpgradeSelf { hash } => {
                    upgrade_self(&CryptoHash::from(hash.clone()));
                }
                Instruction::UpgradeRemote {
                    receiver_id,
                    method_name,
                    hash,
                } => {
                    upgrade_remote(receiver_id, method_name, &CryptoHash::from(hash.clone()));
                }
                Instruction::Transfer {
                    token_id,
                    receiver_id,
                    amount,
                } => {
                    self.internal_payout(token_id, receiver_id, amount.0);
                },
                Instruction::SetStakingContract { staking_id } => {
                    assert!(self.staking_id.is_none(), "ERR_INVALID_STAKING_CHANGE");
                    self.staking_id = Some(staking_id.clone());
                }
                Instruction::AddBounty { bounty } => {
                    self.internal_add_bounty(bounty);
                }
                Instruction::BountyDone {
                    bounty_id,
                    receiver_id,
                } => {
                    self.internal_execute_bounty_payout(*bounty_id, receiver_id, true);
                },
                Instruction::Vote => {}
            }
        }
    }

    /// Process rejecting proposal.
    fn internal_reject_proposal(
        &mut self,
        policy: &Policy,
        proposal: &Proposal
    ) {
        for p in proposal.versions.iter() {
            // Return bond to all proposers.
            Promise::new(p.proposer.clone()).transfer(policy.proposal_bond.0);
            for instr in p.instructions.iter() {
                match instr {
                    Instruction::BountyDone {
                        bounty_id,
                        receiver_id,
                    } => {
                        self.internal_execute_bounty_payout(*bounty_id, receiver_id, false);
                    },
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn internal_user_info(&self) -> UserInfo {
        let account_id = env::predecessor_account_id();
        UserInfo {
            amount: self.get_user_weight(&account_id),
            account_id,
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Add proposal to this DAO.
    #[payable]
    pub fn propose(&mut self, description: String, instructions: Vec<Instruction>) -> u64 {
        let kind = self.internal_check_proposal(&instructions);

        let p = Proposal {
            versions: vec![
                ProposalVersion {
                    proposer: env::predecessor_account_id(),
                    instructions: instructions,
                    description: description,
                }
            ],
            kind: kind.clone(),
            status: ProposalStatus::InProgress,
            approve_count: vec![0],
            reject_count: 0,
            remove_count: vec![0],
            remove_flag: vec![false],
            votes: HashMap::new(),
            remove_votes: Vec::new(),
            submission_time: WrappedTimestamp::from(env::block_timestamp())
        };

        let id = self.last_proposal_id;
        self.proposals
            .insert(&id, &VersionedProposal::Default(p.into()));
        self.last_proposal_id += 1;
        id
    }

    /// Adds a counter proposal to an existing one. Voters can only vote for one of these versions
    #[payable]
    pub fn counter_propose(&mut self, id: u64, description: String, instructions: Vec<Instruction>) -> u8 {
        let mut p: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        
        // the new proposal must be of the same proposal_kind
        let kind = self.internal_check_proposal(&instructions);
        assert_eq!(kind, p.kind, "ERR_DIFFERENT_PROPOSAL_KIND");

        // add the new proposal version, update the tallies and return the corresponding version
        p.versions.push(ProposalVersion{
            proposer: env::predecessor_account_id(),
            instructions,
            description,
        });
        p.approve_count.push(0);
        p.remove_count.push(0);
        p.remove_flag.push(false);
        
        let version: u8 = (p.versions.len() - 1) as u8;
        self.proposals.insert(&id, &VersionedProposal::Default(p));
        version
    }

    // Approve a proposal
    pub fn approve(&mut self, id: u64, version: u8) {
        self.handle_vote(id, version + 1)
    }

    // Reject a proposal
    pub fn reject(&mut self, id: u64) {
        self.handle_vote(id, 0)
    }

    pub fn withdraw(&mut self, id: u64, version: u8) {
        let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let policy = self.policy.get().unwrap().to_policy();

        // Check permissions for the given action
        let allowed = policy.can_execute_action(
            self.internal_user_info(), 
            &proposal.kind, 
            &Action::WithdrawProposal
        );
        assert!(allowed, "ERR_PERMISSION_DENIED");
        assert_eq!(
            proposal.status,
            ProposalStatus::InProgress,
            "ERR_PROPOSAL_NOT_IN_PROGRESS"
        );

        // Only the proposer can withdraw a proposal
        assert!(version < proposal.versions.len() as u8, "ERR_NO_PROPOSAL_VERSION");
        assert!(!proposal.remove_flag[version as usize], "ERR_ALREADY_REMOVED");
        assert_eq!(
            proposal.versions[version as usize].proposer,
            env::predecessor_account_id(),
            "ERR_UNAUTHORIZED_WITHDRAW"
        );

        // No one should have voted on the proposal yet
        assert_eq!(proposal.approve_count[version as usize], 0, "ERR_VOTING_BEGUN");
        assert_eq!(proposal.remove_count[version as usize], 0, "ERR_VOTING_BEGUN");
        proposal.remove_flag[version as usize] = true;

        self.proposals
                .insert(&id, &VersionedProposal::Default(proposal));
    }

    pub fn veto(&mut self, id: u64, version: u8) {
        let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let policy = self.policy.get().unwrap().to_policy();

        // Check permissions for the given action
        let allowed = policy.can_execute_action(self.internal_user_info(), &proposal.kind, &Action::VoteRemove);
        assert!(allowed, "ERR_PERMISSION_DENIED");
        assert_eq!(
            proposal.status,
            ProposalStatus::InProgress,
            "ERR_PROPOSAL_NOT_IN_PROGRESS"
        );
        let vote_policy = policy
            .get_vote_policy(&proposal.kind)
            .unwrap_or(&policy.default_vote_policy);
        let sender_id = env::predecessor_account_id();
        
        let threshold = policy.get_threshold(
            vote_policy,
            self.total_delegation_amount,
            &proposal.kind,
        );
        let weight = self.get_user_weight(&sender_id);
        let removeVote = RemoveVote {
            account_id: sender_id,
            version: version,
        };

        proposal.remove_flag[version as usize] = proposal.update_remove_votes(
            removeVote,
            weight,
            threshold,
        );

        self.proposals
                .insert(&id, &VersionedProposal::Default(proposal));
    }

    pub fn remove(&mut self, id: u64) {
        let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let policy = self.policy.get().unwrap().to_policy();

        // Check permissions for the given action
        let allowed = policy.can_execute_action(
            self.internal_user_info(),
            &proposal.kind, 
            &Action::RemoveProposal
        );
        assert!(allowed, "ERR_PERMISSION_DENIED");
        assert_eq!(
            proposal.status,
            ProposalStatus::InProgress,
            "ERR_PROPOSAL_NOT_IN_PROGRESS"
        );

        self.proposals.remove(&id);
    }

    pub fn finalize(&mut self, id: u64) {
        let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let policy = self.policy.get().unwrap().to_policy();

        let allowed = policy.can_execute_action(
            self.internal_user_info(), 
            &proposal.kind, 
            &Action::Finalize
        );
        assert!(allowed, "ERR_PERMISSION_DENIED");
        assert_eq!(
            proposal.status,
            ProposalStatus::InProgress,
            "ERR_PROPOSAL_NOT_IN_PROGRESS"
        );

        proposal.status = policy.proposal_status(
            &proposal,
            self.total_delegation_amount,
        );
        assert_eq!(
            proposal.status,
            ProposalStatus::Expired,
            "ERR_PROPOSAL_NOT_EXPIRED"
        );
        self.internal_reject_proposal(&policy, &proposal);
    }

    pub fn amend(&mut self, id: u64, version: u8, description: String, instructions: Vec<Instruction>) {
        let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let policy = self.policy.get().unwrap().to_policy();

        // Check permissions for the given action
        let allowed = policy.can_execute_action(
            self.internal_user_info(), 
            &proposal.kind, 
            &Action::AmendProposal
        );
        assert!(allowed, "ERR_PERMISSION_DENIED");
        assert_eq!(
            proposal.status,
            ProposalStatus::InProgress,
            "ERR_PROPOSAL_NOT_IN_PROGRESS"
        );

        // the new proposal must be of the same proposal_kind
        let kind = self.internal_check_proposal(&instructions);
        assert_eq!(kind, proposal.kind, "ERR_DIFFERENT_PROPOSAL_KIND");

        assert!(version < proposal.versions.len() as u8, "ERR_NO_PROPOSAL_VERSION");
        assert_eq!(proposal.versions[version as usize].proposer, env::predecessor_account_id(), "ERR_UNAUTHORIZED_AMEND");
        // No one should have voted on the proposal yet
        assert_eq!(proposal.approve_count[version as usize], 0, "ERR_VOTING_BEGUN");
        assert_eq!(proposal.remove_count[version as usize], 0, "ERR_VOTING_BEGUN");
        
        proposal.versions[version as usize] = ProposalVersion {
            proposer: env::predecessor_account_id(),
            description,
            instructions
        };
        self.proposals
                .insert(&id, &VersionedProposal::Default(proposal));
    }

    fn handle_vote(&mut self, id: u64, choice: u8) {
        let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let policy = self.policy.get().unwrap().to_policy();
        assert!(choice <= proposal.versions.len() as u8, "ERR_INVALID_CHOICE");

        // Check permissions for the given action
        let mut action = Action::VoteApprove;
        if choice == 0 {
            action = Action::VoteReject;
        }
        let allowed = policy.can_execute_action(self.internal_user_info(), &proposal.kind, &action);
        assert!(allowed, "ERR_PERMISSION_DENIED");
        assert_eq!(
            proposal.status,
            ProposalStatus::InProgress,
            "ERR_PROPOSAL_NOT_IN_PROGRESS"
        );
        let vote_policy = policy
            .get_vote_policy(&proposal.kind)
            .unwrap_or(&policy.default_vote_policy);
        let sender_id = env::predecessor_account_id();

        // create the vote - this also checks that the proposal version exists
        let vote = proposal.create_vote(
            &vote_policy,
            choice, 
            self.get_user_weight(&sender_id)
        );

        let threshold = policy.get_threshold(
            &vote_policy,
            self.total_delegation_amount, 
            &proposal.kind,
        );

        // update the votes and check if the status of the proposal has changed
        proposal.status = proposal.update_votes(
            &sender_id,
            vote,
            &vote_policy,
            threshold
        );
        match proposal.status {
            ProposalStatus::Approved{ version } => { 
                // success, now execute the proposal
                self.internal_execute_proposal(&policy, &proposal, &proposal.versions[version as usize]);
            },
            ProposalStatus::Rejected => {
                // defeated, return the bond
                self.internal_reject_proposal(&policy, &proposal)
            }
            _ => {}, 
        };

        // update the proposal
        self.proposals
                .insert(&id, &VersionedProposal::Default(proposal));
    }

    fn internal_check_proposal(&mut self, instructions: &Vec<Instruction>) -> String {
        let policy = self.policy.get().unwrap().to_policy();
        assert!(
            env::attached_deposit() >= policy.proposal_bond.0,
            "ERR_MIN_BOND"
        );

        // 1. validate proposal.
        assert!(instructions.len() > 0, "ERR_EMPTY_INSTRUCTION_SET");
        assert!(self.is_valid_instruction_set(&instructions), "ERR_INVALID_INSTRUCTION_SET");
        match instructions[0] {
            Instruction::SetStakingContract { .. } => assert!(
                self.staking_id.is_none(),
                "ERR_STAKING_CONTRACT_CANT_CHANGE"
            ),
            // TODO: add more verifications.
            _ => {}
        };

        // 2. check permission of caller to add proposal.
        let kind = policy.match_proposal_kind(&instructions);
        assert!(
            policy
                .can_execute_action(
                    self.internal_user_info(),
                    &kind,
                    &Action::AddProposal
                ),
            "ERR_PERMISSION_DENIED"
        );
        // 3. return the proposal kind
        kind
    }

    fn is_valid_instruction_set(&self, instructions: &Vec<Instruction>) -> bool {
        if instructions.len() > 1 {
            for instr in instructions.iter() {
                match instr {
                    // these instructions must be put in a standalone proposal
                    Instruction::SetStakingContract{ .. } | 
                    Instruction::UpgradeSelf{ .. } |
                    Instruction::Vote |
                    Instruction::BountyDone{ .. } => return false,
                    // TODO: add more cases
                    _ => {},
                }
            }
        }
        true
    }
}
