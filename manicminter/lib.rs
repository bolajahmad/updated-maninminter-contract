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

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let manicminter = Manicminter::default();
            assert_eq!(manicminter.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut manicminter = Manicminter::new(false);
            assert_eq!(manicminter.get(), false);
            manicminter.flip();
            assert_eq!(manicminter.get(), true);
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = ManicminterRef::default();

            // When
            let contract_account_id = client
                .instantiate("manicminter", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<ManicminterRef>(contract_account_id.clone())
                .call(|manicminter| manicminter.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = ManicminterRef::new(false);
            let contract_account_id = client
                .instantiate("manicminter", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<ManicminterRef>(contract_account_id.clone())
                .call(|manicminter| manicminter.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<ManicminterRef>(contract_account_id.clone())
                .call(|manicminter| manicminter.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<ManicminterRef>(contract_account_id.clone())
                .call(|manicminter| manicminter.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
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
