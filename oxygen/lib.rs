#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(PSP22, PSP22Mintable, Ownable)]
#[openbrush::contract]
mod oxygen {
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Storage, Default)]
    pub struct Oxygen {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        ownable: ownable::Data,
    }

    impl Oxygen {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let mut instance = Self::default();
            let caller = Self::env().caller();
            psp22::Internal::_mint_to(&mut instance, caller, initial_supply).expect("Should mint");
            ownable::Internal::_init_with_owner(&mut instance, caller);
            instance
        }
    }
}
