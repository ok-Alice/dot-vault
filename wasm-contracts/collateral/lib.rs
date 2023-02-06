#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod psp34_wrapper {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        traits::{
            Storage,
            String,
        },
    };
    use openbrush::contracts::psp34::{PSP34Error};
    use ethabi::ethereum_types::U256;
    use openbrush::contracts::psp34::Id;
    use xvm_helper::XvmErc721;

    #[derive(Default, SpreadAllocate, Storage)]
    #[ink(storage)]
    pub struct PSP34Wrapper {
//        #[storage_field]
        evm_address: [u8; 20],
        
    }

    pub enum TmpError {
        Error,
    }

    impl PSP34Wrapper {
        #[ink(constructor)]
        pub fn new(evm_contract_address: [u8; 20]) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {

                instance.evm_address = evm_contract_address;
             
            })
        }

        #[ink(message)]
        pub fn deposit(&mut self, id: Id) -> Result<(),PSP34Error> {
            let caller = self.env().caller();
            let contract = self.env().account_id();
            XvmErc721::transfer_from(self.evm_address, caller, contract, cast(id.clone()))
                .map_err(|_| PSP34Error::Custom(String::from("transfer failed")))
 
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