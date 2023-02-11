#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
use ink_storage::traits::SpreadAllocate;
use ink_env::call::FromAccountId;

pub use self::sign_transfer::{
    SignTransfer,
    SignTransferRef,
};

/// EVM ID (from astar runtime)
const EVM_ID: u8 = 0x0F;

#[ink::contract(env = xvm_environment::XvmDefaultEnvironment)]
pub mod sign_transfer {
    use ethabi::{
        ethereum_types::{
            H160,
            U256,
        },
        Token,
    };
    use hex_literal::hex;
    use ink_prelude::{
        string::{
            String,
        },
        vec::Vec,
    };

    const TRANSFER_FROM_SELECTOR: [u8; 4] = hex!["23b872dd"];

    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum STError {
        Custom(String),
    } 

    /// SignTransfer
    ///
    /// Used by Collateral for transfering ERC721 token back to original owner. 
    /// By calling this contract, collateral ensures this contract signs the transfer
    /// and not the caller of collateral.
    #[ink(storage)]
    pub struct SignTransfer {
        

    }


    impl SignTransfer {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {  }
        }

        #[ink(message, selector = 0x3128d61b)]
        pub fn transfer(&mut self, evm_address: [u8; 20], to: AccountId, id: u32, _data: Vec<u8>) -> Result<(), STError> {
            let caller = self.env().caller();
            let encoded_input = Self::transfer_from_encode(Self::h160(&caller), Self::h160(&to), id.into());
            self.env()
                .extension()
                .xvm_call(
                    super::EVM_ID,
                    Vec::from(evm_address.as_ref()),
                    encoded_input,
                )
                .map_err(|_| STError::Custom(String::from("transfer failed")))
        }

        fn transfer_from_encode(from: H160, to: H160, token_id: U256) -> Vec<u8> {
            let mut encoded = TRANSFER_FROM_SELECTOR.to_vec();
            let input = [
                Token::Address(from),
                Token::Address(to),
                Token::Uint(token_id),
            ];
            encoded.extend(&ethabi::encode(&input));
            encoded
        }

        fn h160(from: &AccountId) -> H160 {
            let mut dest: H160 = [0; 20].into();
            dest.as_bytes_mut()
                .copy_from_slice(&<ink_env::AccountId as AsRef<[u8]>>::as_ref(from)[..20]);
            dest
        }
    }

    
}

// https://github.com/paritytech/ink/issues/1149
impl SpreadAllocate for SignTransferRef {
    fn allocate_spread(_ptr: &mut ink_primitives::KeyPtr) -> Self {
        FromAccountId::from_account_id([0; 32].into())
    }
}