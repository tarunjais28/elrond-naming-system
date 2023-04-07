use super::*;

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct State<M: ManagedTypeApi> {
    pub grace: u64,
    pub benificiary: ManagedAddress<M>,
    pub royalty: BigUint<M>,
}

impl<M: ManagedTypeApi> State<M> {
    pub fn store(&mut self, state: State<M>) {
        self.grace = state.grace;
        self.benificiary = state.benificiary;
        self.royalty = state.royalty;
    }
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum AuthorityField {
    Maintainer,
    Admin,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum UpdateKind {
    Remove,
    Add,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct AuthorityUpdateParams<M: ManagedTypeApi> {
    pub field: AuthorityField,
    pub kind: UpdateKind,
    pub address: ManagedAddress<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum UpdateInternalValueParams<M: ManagedTypeApi> {
    Royalty(BigUint<M>),
    Beneficiary(ManagedAddress<M>),
}

/// Minting Data.
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct MintParams<M: ManagedTypeApi> {
    /// TokenId to mint
    pub token_id: TokenIdentifier<M>,
    /// Token domain name
    pub domain: ManagedBuffer<M>,
    /// The account address of owner.
    pub owner: ManagedAddress<M>,
    /// Initial subscription duration of the CNS Domain.
    pub duration: u64,
}

/// Token Data
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct TokenData<M: ManagedTypeApi> {
    /// The account address of owner.
    pub owner: ManagedAddress<M>,
    /// The expiry timestamp.
    pub expiry: u64,
    /// The grace period.
    pub grace: u64,
    /// Token domain name
    pub domain: ManagedBuffer<M>,
    /// Royalty.
    pub royalty: BigUint<M>,
}

impl<M: ManagedTypeApi> TokenData<M> {
    pub fn new(
        owner: ManagedAddress<M>,
        expiry: u64,
        grace: u64,
        domain: ManagedBuffer<M>,
        royalty: BigUint<M>,
    ) -> Self {
        Self {
            owner,
            expiry,
            grace,
            domain,
            royalty,
        }
    }
}

/// Transfer Data.
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Transfer<M: ManagedTypeApi> {
    /// The ID of the token being transferred.
    pub token_id: TokenIdentifier<M>,
    /// The amount of tokens being transferred.
    pub amount: u64,
    /// The address owning the tokens being transferred.
    pub from: ManagedAddress<M>,
    /// The address receiving the tokens being transferred.
    pub to: ManagedAddress<M>,
    /// Additional data to include in the transfer.
    /// Can be used for additional arguments.
    pub data: Vec<u8>,
}

// Subscription Data
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct SubscriptionData<M: ManagedTypeApi> {
    pub owner: ManagedAddress<M>,
    pub expiry: u64,
    pub grace: u64,
}

impl<M: ManagedTypeApi> SubscriptionData<M> {
    pub fn into_status(self, slot_time: u64) -> TokenSubscriptionStatus<M> {
        let grace_period = self
            .expiry
            .checked_add(self.grace)
            .expect("Error while adding expiry and grace periods!");

        let status = if self.expiry >= slot_time {
            SubscriptionExpiryStatus::Owned(self.expiry)
        } else if grace_period >= slot_time {
            SubscriptionExpiryStatus::Grace(grace_period)
        } else {
            SubscriptionExpiryStatus::Expired
        };

        TokenSubscriptionStatus {
            owner: self.owner,
            status,
        }
    }
}

// Subscription Expiry Status
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum SubscriptionExpiryStatus {
    Owned(u64),
    Grace(u64),
    Expired,
}

// Token Subscription Status
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct TokenSubscriptionStatus<M: ManagedTypeApi> {
    pub owner: ManagedAddress<M>,
    pub status: SubscriptionExpiryStatus,
}

// Token Info
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct TokenInfo<M: ManagedTypeApi> {
    pub domain: ManagedBuffer<M>,
    pub royalty: BigUint<M>,
}
