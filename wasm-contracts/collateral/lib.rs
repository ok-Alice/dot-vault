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
    use sign_transfer::sign_transfer::{
        SignTransferRef,
        CollateralError};

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
        pub fn deposit_nft(&mut self, evm_address: EvmContractAddress, id: u32) -> Result<(),CollateralError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();
            
            //TODO: Check contract is allowed

            XvmErc721::transfer_from(evm_address, caller, contract, U256::from(id))
                .map_err(|_| CollateralError::Custom(String::from("transfer failed")))

            //TODO: 
        }


        #[ink(message)]
        pub fn withdraw_nft(&mut self, evm_address: EvmContractAddress, id: u32) -> Result<(),CollateralError> {
            let caller = self.env().caller();
            //PSP34Ref::transfer(&mut self.psp34_controller, caller, id, Vec::new())

            //TODO: check user holds this NFT as collateral
            //TODO: check user balance allows this

            SignTransferRef::transfer(&mut self.sign_transfer, evm_address, caller, id)

            //TODO: modify user load balance
        }

        
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