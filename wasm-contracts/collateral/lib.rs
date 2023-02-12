#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod collateral {
    use ink_storage::{
        Mapping,
        traits::SpreadAllocate
    };
    use openbrush::{
        //storage::Mapping,
        traits::{
            Storage,
            String,
        },
    };

    use openbrush::{
        contracts::ownable::*,
        modifiers,
    };


    use sign_transfer::sign_transfer::{
        SignTransferRef,
        CollateralError};

    use ethabi::ethereum_types::U256;
    use xvm_helper::XvmErc721;

    type EvmContractAddress = [u8; 20];
    type RiskFactor = u32;
    type CollateralFactor = u32;

    type LoanLimit = Balance;
    type LoanOpen = Balance;
    type LoanLastChange = BlockNumber;
    type InterestRate = u32; // Interast rate / block (/1_000_000)

    #[derive(SpreadAllocate, Storage)]
    #[ink(storage)]
    pub struct Collateral {
        collections: Mapping<EvmContractAddress, (RiskFactor, CollateralFactor)>,
        loans: Mapping<AccountId, (LoanLimit, LoanOpen, LoanLastChange)>,
        sign_transfer: SignTransferRef,
        interest_rate: InterestRate,
        #[storage_field]
        ownable: ownable::Data,
    }

    impl Ownable for Collateral {}

    impl Collateral {

        /// Constructor:
        ///  - Needs the hash of previously deployed sign-transfer contract 
        #[ink(constructor)]
        pub fn new(version: u32, sign_transfer_hash: Hash, interest_rate: Option<InterestRate>) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_owner(caller);

                instance.interest_rate = interest_rate.unwrap_or(15);

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

        /// Allows a user to deposit an NFT as collateral to increase it's loan limit
        #[ink(message)]
        pub fn deposit_nft(&mut self, evm_address: EvmContractAddress, id: u32) -> Result<(),CollateralError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();
            
            //TODO: Check contract is allowed

            XvmErc721::transfer_from(evm_address, caller, contract, U256::from(id))
                .map_err(|_| CollateralError::Custom(String::from("transfer failed")))

            //TODO: query oracle
            //TODO: modify user loan balance
        }


        /// Allows a user to reclaim NFT to decrease it's loan balance (as long as open loan is smaller)
        #[ink(message)]
        pub fn withdraw_nft(&mut self, evm_address: EvmContractAddress, id: u32) -> Result<(),CollateralError> {
            let caller = self.env().caller();
            //PSP34Ref::transfer(&mut self.psp34_controller, caller, id, Vec::new())

            //TODO: check user holds this NFT as collateral
            //TODO: check user balance allows this

            SignTransferRef::transfer(&mut self.sign_transfer, evm_address, caller, id)

            //TODO: modify user load limit
        }

        /// Allows admin to add NFT collection to list of allowed collections
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn register_nft_collection(&mut self, evm_address: EvmContractAddress, risk_factor: RiskFactor, collateral_factor: CollateralFactor) -> Result<(),CollateralError> {
            self.collections.insert(&evm_address, &(risk_factor, collateral_factor));
            Ok(())
        }

        /// Allows anyone to query registered NFT collections
        #[ink(message)]
        pub fn registered_nft_collection(&self, evm_address: EvmContractAddress) -> Result<(RiskFactor, CollateralFactor), CollateralError> {
            match self.collections.get(&evm_address) {
                Some((r,c)) => return Ok((r,c)),
                None => Err(CollateralError::Custom(String::from("Unknown Collection")))
            }
        }

        /// Allows user to request transfer of SCoin as long as loan limit allows it
        #[ink(message)]
        pub fn take_loan(&mut self, _amount: Balance ) -> Result<(), CollateralError> {
            // TODO: check if user has enough loan limit - open loan

            // TODO: transfer CCoin using SignTransferRef

            // TODO: register open-loan

            Ok(())

        }

        /// Allows user to repay previously claimed loan
        #[ink(message)] 
        pub fn repay_loan(&mut self, _amount: Balance) -> Result<(), CollateralError> {
            // TODO: check if the user has this much open loan

            // TODO: transfer CCoin from user to contract (SignTransferRef)

            Ok(())
        }

        /// Allows anyone to check loan status of given user
        #[ink(message)]
        pub fn loan_status(&self, user: AccountId) -> Result<(LoanLimit, LoanOpen, LoanLastChange),CollateralError> {
            match self.loans.get(&user) {
                Some((l,o,c)) => return Ok((l,o,c)),
                None => Err(CollateralError::Custom(String::from("Unknown User")))
            }
        }

        /// Allows caller to check its loan balance
        #[ink(message)]
        pub fn my_loan_status(&self) -> Result<(LoanLimit, LoanOpen, LoanLastChange),CollateralError> {
            let caller = self.env().caller();
            self.loan_status(caller)
        }



        /// Caculates and updates open loan for given user
        /// new open loan = old open loan + (last loan change - now) * interest rate
        #[ink(message)] 
        pub fn updates_loan_balance(&mut self, _user: AccountId) -> Result<(), CollateralError> {
            //TODO: calculate interest since last update

            //TOD: update open loan and loan last change for this user

            Ok(())
        }

    }

}