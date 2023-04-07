#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod authority;
mod nft_module;
mod structs;

pub use crate::structs::*;
use elrond_wasm::types::heap::Vec;

#[elrond_wasm::contract]
pub trait NftMinter: nft_module::NftModule + authority::Authority {
    #[init]
    fn init(&self, params: State<Self::Api>) {
        self.state().set(params);
        self.admins().insert(self.blockchain().get_caller());
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::redundant_closure)]
    #[endpoint(createNft)]
    fn create_nft(
        &self,
        name: ManagedBuffer,
        uri: ManagedBuffer,
        selling_price: BigUint,
        opt_token_used_as_payment: OptionalValue<TokenIdentifier>,
        opt_token_used_as_payment_nonce: OptionalValue<u64>,
        params: MintParams<Self::Api>,
    ) {
        let token_used_as_payment = match opt_token_used_as_payment {
            OptionalValue::Some(token) => EgldOrEsdtTokenIdentifier::esdt(token),
            OptionalValue::None => EgldOrEsdtTokenIdentifier::egld(),
        };
        require!(
            token_used_as_payment.is_valid(),
            "Invalid token_used_as_payment arg, not a valid token ID"
        );

        let token_used_as_payment_nonce = if token_used_as_payment.is_egld() {
            0
        } else {
            match opt_token_used_as_payment_nonce {
                OptionalValue::Some(nonce) => nonce,
                OptionalValue::None => 0,
            }
        };

        let expiry = self
            .blockchain()
            .get_block_timestamp()
            .checked_add(params.duration)
            .expect("Error while adding duration to current timestamp!");

        let state = self.state().get();
        let caller = self.blockchain().get_caller();
        let token_id = params.token_id.clone();

        // Storing token details
        self.token_details().insert(
            token_id.clone(),
            TokenData::new(
                caller.clone(),
                expiry,
                state.grace,
                params.domain.clone(),
                state.royalty.clone(),
            ),
        );

        self.create_nft_with_attributes(
            name,
            state.royalty,
            token_id.clone(),
            params,
            uri,
            selling_price,
            token_used_as_payment,
            token_used_as_payment_nonce,
        );

        // Logging mint event
        self.mint_event(&token_id, &caller, &1);
    }

    /// Function to burn token.
    ///
    /// It rejects if:
    /// - Current Time is less than expiry + Grace Period
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::redundant_closure)]
    #[endpoint(burn)]
    fn burn(&self, token_id: TokenIdentifier<Self::Api>) {
        let slot_time = self.blockchain().get_block_timestamp();

        let token_data = self
            .token_details()
            .get(&token_id)
            .expect("Token Id does not exists!");

        // Checking token validation
        require!(
            token_data
                .expiry
                .checked_add(token_data.grace)
                .expect("Error while adding expiry and grace periods!")
                < slot_time,
            "Error Grace period must not be over!"
        );

        // Burning token data
        let token_data = self
            .token_details()
            .remove(&token_id)
            .expect("Error while burning token data!");

        // Logging burn event
        self.burn_event(&token_id, &token_data.owner, &1);
    }

    /// Execute a list of domain transfers, in the order of the list.
    ///
    /// It rejects if:
    /// - Any of the transfers fail to be executed, which could be if:
    ///     - The `token_id` does not exist.
    ///     - The caller is not the owner of the token, or an operator for this
    ///       specific `token_id` and `from` address.
    ///     - The token is not owned by the `from` address.
    /// - Fails to log event.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::redundant_closure)]
    #[endpoint(transfer)]
    fn transfer(&self, transfers: Vec<Transfer<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let slot_time = self.blockchain().get_block_timestamp();

        for transfer in transfers {
            // Check owner from storage
            let token_data = self
                .token_details()
                .get(&transfer.token_id)
                .expect("Token Id does not exists!");

            require!(
                token_data.owner.eq(&transfer.from) && token_data.owner.eq(&caller),
                "Error: caller must be the owner of the domain!"
            );

            // Check the transfer amount
            match transfer.amount {
                0 => continue,
                1 => (),
                _ => sc_panic!("Error: operation not permitted!"),
            }

            // Check token expiry.
            require!(
                token_data
                    .expiry
                    .checked_add(token_data.grace)
                    .expect("Error while adding expiry and grace periods!")
                    < slot_time,
                "Error Grace period must not be over!"
            );

            require!(
                token_data.expiry < slot_time,
                "Error expiry period must not be over!"
            );

            // Logging event
            // TODO: Move logging at the end of the owner update. Logging
            // before actual move to avoid rust's error[E0382] `use of
            // partially moved value in closure`.
            self.transfer_event(
                &transfer.token_id,
                &transfer.from,
                &transfer.to,
                &transfer.amount,
            );

            // Update token data
            let to_address = transfer.to;
            self.token_details()
                .entry(transfer.token_id)
                .and_modify(|token_data| token_data.owner = to_address);
        }
    }

    // The marketplace SC will send the funds directly to the initial caller, i.e. the owner
    // The caller has to know which tokens they have to claim,
    // by giving the correct token ID and token nonce
    #[only_owner]
    #[endpoint(claimRoyaltiesFromMarketplace)]
    fn claim_royalties_from_marketplace(
        &self,
        marketplace_address: ManagedAddress,
        token_id: TokenIdentifier,
        token_nonce: u64,
    ) {
        let caller = self.blockchain().get_caller();
        self.marketplace_proxy(marketplace_address)
            .claim_tokens(token_id, token_nonce, caller)
            .async_call()
            .call_and_exit()
    }

    /// Function to update internal values. This includes:
    /// - Royalty. Fee percentage for token sale. Gets assigned to a token on mint.
    /// - Beneficiary. Account address that receives the fee.
    ///
    ///  It rejects if:
    ///  - If caller is neither one of the admins nor one of the maintainers.
    #[endpoint(updateInternalValue)]
    fn update_internal_value(&self, update_params: UpdateInternalValueParams<Self::Api>) {
        let caller = self.blockchain().get_caller();

        require!(
            self.has_maintainer_rights(&caller),
            "Unauthorized maintainer rights by the caller address!"
        );

        match update_params {
            UpdateInternalValueParams::Royalty(percentage) => {
                self.state().update(|state| state.royalty = percentage)
            }
            UpdateInternalValueParams::Beneficiary(account) => {
                self.state().update(|state| state.benificiary = account)
            }
        }
    }

    #[proxy]
    fn marketplace_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> nft_marketplace_proxy::Proxy<Self::Api>;

    // storage

    #[view]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State<Self::Api>>;

    #[view]
    #[storage_mapper("tokenDetails")]
    fn token_details(&self) -> MapMapper<TokenIdentifier, TokenData<Self::Api>>;

    /// Function to get domain subscription status.
    #[view(getTokenSubscriptionStatus)]
    fn get_token_subscription_status(
        &self,
        token_id: TokenIdentifier,
    ) -> OptionalValue<TokenSubscriptionStatus<Self::Api>> {
        let slot_time = self.blockchain().get_block_timestamp();

        if let Some(token_data) = self.token_details().get(&token_id) {
            OptionalValue::Some(
                SubscriptionData {
                    owner: token_data.owner,
                    expiry: token_data.expiry,
                    grace: token_data.grace,
                }
                .into_status(slot_time),
            )
        } else {
            OptionalValue::None
        }
    }

    /// View token data owned by particular address by token_id.
    #[view(getTokenInfo)]
    fn get_token_info(&self, token_id: TokenIdentifier) -> OptionalValue<TokenInfo<Self::Api>> {
        if let Some(token_data) = self.token_details().get(&token_id) {
            OptionalValue::Some(
                SubscriptionData {
                    owner: token_data.owner,
                    expiry: token_data.expiry,
                    grace: token_data.grace,
                }
            )
        } else {
            OptionalValue::None
        }
    }

    // Events

    #[event("mint")]
    fn mint_event(
        &self,
        #[indexed] token_id: &TokenIdentifier,
        #[indexed] owner: &ManagedAddress,
        amount: &u64,
    );

    #[event("burn")]
    fn burn_event(
        &self,
        #[indexed] token_id: &TokenIdentifier,
        #[indexed] owner: &ManagedAddress,
        amount: &u64,
    );

    #[event("transfer")]
    fn transfer_event(
        &self,
        #[indexed] token_id: &TokenIdentifier,
        #[indexed] from: &ManagedAddress,
        #[indexed] to: &ManagedAddress,
        amount: &u64,
    );
}

mod nft_marketplace_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait NftMarketplace {
        #[endpoint(claimTokens)]
        fn claim_tokens(
            &self,
            token_id: TokenIdentifier,
            token_nonce: u64,
            claim_destination: ManagedAddress,
        );
    }
}
