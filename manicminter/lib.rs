#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod manicminter {
    use crate::ensure;
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Manicminter {
        /// Stores a single `bool` value on the storage.
        owner: AccountId,
        token_contract: AccountId,
        price: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadMintValue,
        ContractNotSet,
        NotOwner,
        Overflow,
        TransactionFailed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink::trait_definition]
    pub trait Minting {
        #[ink(message, payable)]
        fn manic_mint(&mut self, amount: Balance) -> Result<()>;

        #[ink(message)]
        fn set_price(&mut self, new_price: Balance) -> Result<()>;

        #[ink(message)]
        fn get_price(&self) -> Balance;
    }

    impl Manicminter {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(contract_account: AccountId) -> Self {
            Self {
                owner: Self::env().caller(),
                token_contract: contract_account,
                price: 0,
            }
        }
    }

    impl Minting for Manicminter {
        #[ink(message, payable)]
        fn manic_mint(&mut self, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            ensure!(self.token_contract != zero_address(), Error::ContractNotSet);

            match self.price.checked_mul(amount) {
                Some(value) => {
                    let transferred_value = self.env().transferred_value();
                    ensure!(transferred_value >= value, Error::TransactionFailed);
                }
                None => {
                    return Err(Error::Overflow);
                }
            }

            let mint_result = build_call::<DefaultEnvironment>()
                .call(self.token_contract)
                .gas_limit(5000000000)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22Mintable::mint")))
                        .push_arg(caller)
                        .push_arg(amount),
                )
                .returns::<()>()
                .try_invoke();

            match mint_result {
                Ok(Ok(_)) => Ok(()),
                _ => Err(Error::TransactionFailed),
            }
        }

        #[ink(message)]
        fn set_price(&mut self, new_price: Balance) -> Result<()> {
            ensure!(self.env().caller() == self.owner, Error::NotOwner);
            self.price = new_price;
            Ok(())
        }

        #[ink(message)]
        fn get_price(&self) -> Balance {
            self.price
        }
    }

    fn zero_address() -> AccountId {
        AccountId::from([0x0; 32])
    }
}

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into());
        }
    }};
}
