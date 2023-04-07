use super::*;

#[elrond_wasm::module]
pub trait Authority {
    fn has_maintainer_rights(&self, address: &ManagedAddress<Self::Api>) -> bool {
        self.maintainers().contains(address) || self.admins().contains(address)
    }

    fn has_admin_rights(&self, address: &ManagedAddress<Self::Api>) -> bool {
        self.admins().contains(address)
    }

    /// Function to manage addresses that are allowed to maintain and modify the state of the contract.
    ///
    ///  It rejects if:
    ///  - If caller is neither one of the admins nor one of the maintainers.
    #[endpoint(updateAuthority)]
    fn update_authority(&self, update_params: AuthorityUpdateParams<Self::Api>) {
        let caller = self.blockchain().get_caller();

        match update_params.field {
            AuthorityField::Maintainer => {
                require!(
                    self.has_maintainer_rights(&caller),
                    "Unauthorized maintainer rights by the caller address!"
                );

                // TODO: Return mutable SetMapper address and update
                match update_params.kind {
                    UpdateKind::Remove => {
                        self.maintainers().remove(&update_params.address);
                    }
                    UpdateKind::Add => {
                        self.maintainers().insert(update_params.address);
                    }
                }
            }
            AuthorityField::Admin => {
                require!(
                    self.has_admin_rights(&caller),
                    "Unauthorized admin rights by the caller address!"
                );

                // TODO: Return mutable SetMapper address and update
                match update_params.kind {
                    UpdateKind::Remove => {
                        self.admins().remove(&update_params.address);
                    }
                    UpdateKind::Add => {
                        self.admins().insert(update_params.address);
                    }
                }
            }
        };
    }

    // storage

    #[view]
    #[storage_mapper("admins")]
    fn admins(&self) -> SetMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("maintainers")]
    fn maintainers(&self) -> SetMapper<ManagedAddress>;
}
