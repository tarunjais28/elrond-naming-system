#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Eq, Clone, Debug)]
pub enum PriceType {
    Fixed,
    Dynamic,
}

#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, ManagedVecItem,
)]
pub struct PriceItem<M: ManagedTypeApi> {
    price: BigUint<M>,
    length: u8,
}

#[elrond_wasm::contract]
pub trait PriceOracleContract: common::Utils {
    #[init]
    fn init(&self) {
        self.current_type().set(&PriceType::Fixed);
        self.price_fixed().set(&BigUint::zero());
    }

    #[view]
    fn get_price(&self, length: u8) -> BigUint {
        let price_type = self.current_type().get();
        match price_type {
            PriceType::Fixed => self.price_fixed().get(),
            PriceType::Dynamic => {
                let first_item = self.price_mid().get(1);
                require!(
                    first_item.price > 0,
                    "First element less than zero"
                );
                let last_item = self.price_mid().get(self.price_mid().len());
                if first_item.length > length {
                    return self.price_less().get();
                }
                if last_item.length < length {
                    return self.price_more().get();
                }
                let found = self
                    .price_mid()
                    .iter()
                    .find(|item| item.length == length);
                if found.is_none() {
                    return self.price_more().get();
                } else {
                    return found.expect("unwrap found price").price;
                }
            }
            _ => BigUint::zero(),
        }
    }

    

    #[endpoint]
    fn set_price(
        &self,
        price_type: PriceType,
        price: BigUint,
        price_mid: ManagedVec<PriceItem<Self::Api>>,
        price_more: BigUint,
    ) {
        match price_type {
            PriceType::Fixed => {
                self.current_type().set(price_type);
                self.price_fixed().set(self.to_wei(price));
            }
            PriceType::Dynamic => {
                self.current_type().set(price_type);
                require!(price > 0, "Less price should be a positive number");
                self.price_less().set(price);
                require!(
                    !price_mid.is_empty(),
                    "Price Mid is empty!"
                );
                self.price_mid().clear();
                price_mid.iter().for_each(|item| {
                    let item = PriceItem {
                        length: item.length,
                        price: item.price.clone(),
                    };
                    
                    self.price_mid().push(&item);
                });

                self.price_more().set(price_more);
            }
        }
    }

    #[storage_mapper("price_less")]
    fn price_less(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("price_more")]
    fn price_more(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("price_mid")]
    fn price_mid(&self) -> VecMapper<PriceItem<Self::Api>>;

    #[storage_mapper("current_type")]
    fn current_type(&self) -> SingleValueMapper<PriceType>;

    #[storage_mapper("price_fixed")]
    fn price_fixed(&self) -> SingleValueMapper<BigUint>;
}
