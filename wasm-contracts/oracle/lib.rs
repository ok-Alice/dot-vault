#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod oracle {
    use ink_storage::{
        traits::{SpreadAllocate},
        Mapping,
    };
    use ink_prelude::string::String;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct Oracle {
        values: Mapping<String, String>,
        owner: AccountId,
    }

    impl Oracle {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.owner = Self::env().caller();
            })
        }

        #[ink(message)]
        pub fn get(&self, key: String) -> String {
            return self.values.get(key).unwrap_or(String::from(""));
        }

        #[ink(message)]
        pub fn set(&mut self, key: String, value: String) {
            let caller = self.env().caller();
            assert_eq!(caller, self.owner);
            self.values.insert(
                key,
                &value,
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn sample_test() {
            let mut oracle = Oracle::new();
            assert_eq!(oracle.get(String::from("foo")), String::from(""));

            oracle.set(String::from("foo"), String::from("bar"));
            assert_eq!(oracle.get(String::from("foo")), String::from("bar"));
        }
    }
}
