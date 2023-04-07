#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();



#[elrond_wasm::module]
pub trait Utils {

    fn to_wei(&self, amount: BigUint) -> BigUint {            
        amount * &BigUint::from(10 as u32).pow(18)
    }

    fn to_egld(&self, amount: BigUint) -> BigUint {
        amount / &BigUint::from(10 as u32).pow(18)
    }

}