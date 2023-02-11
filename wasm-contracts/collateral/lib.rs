#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod collateral {
    use ink_storage::traits::SpreadAllocate;
    use ink_prelude::vec::Vec;
    use openbrush::{
        //storage::Mapping,
        traits::{
            Storage,
            String,
        },
    };

    //use openbrush::contracts::psp34::{PSP34Error};
    use sign_transfer::sign_transfer::SignTransferRef;

    use ethabi::ethereum_types::U256;
    use openbrush::contracts::psp34::Id;
    use xvm_helper::XvmErc721;

    type EvmContractAddress = [u8; 20];

    #[derive(SpreadAllocate, Storage)]
    #[ink(storage)]
    pub struct Collateral {
        //#[storage_field]
        collections: Vec<EvmContractAddress>,
        risk_factor: u32,
        sign_transfer: SignTransferRef,
    }

    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CollateralError {
        Custom(String),
    } 

    impl Collateral {

        #[ink(constructor)]
        pub fn new(version: u32, sign_transfer_hash: Hash, risk_factor: u32) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.risk_factor = risk_factor;

                let salt = version.to_le_bytes();
                let sign_transfer = SignTransferRef::new()
                    .endowment(0)
                    .code_hash(sign_transfer_hash)
                    .salt_bytes(salt)
                    .instantiate()
                    .unwrap_or_else(|error| {
                        panic!("failed at instantiating the Sign Transfer contract: {:?}", error)
                    });

                instance.sign_transfer = sign_transfer;
            })
        }

        #[ink(message)]
        pub fn deposit(&mut self, evm_address: EvmContractAddress, id: Id) -> Result<(),CollateralError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();
            
            //TODO: Check contract is allowed

            XvmErc721::transfer_from(evm_address, caller, contract, cast(id.clone()))
                .map_err(|_| CollateralError::Custom(String::from("transfer failed")))
        }


        // #[ink(message)]
        // pub fn withdraw(&mut self, id: Id) -> Result<(),PSP34Error> {
        //     let caller = self.env().caller();
        //     PSP34Ref::transfer(&mut self.psp34_controller, caller, id, Vec::new())
        // }
    }

    fn cast(id: Id) -> U256 {
        return match id {
            Id::U8(v) => U256::from(v),
            Id::U16(v) => U256::from(v),
            Id::U32(v) => U256::from(v),
            Id::U64(v) => U256::from(v),
            Id::U128(v) => U256::from(v),
            Id::Bytes(v) => U256::from(v.as_slice()),
        }
    }
}