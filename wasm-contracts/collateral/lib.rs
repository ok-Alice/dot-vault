#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod collateral {
    use ink_storage::{
        Mapping,
        traits::SpreadAllocate
    };
    use ink_prelude::vec::Vec;
    use ink_prelude::string::ToString;
    use openbrush::{
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
        CollateralError
    };
    use oracle::oracle::OracleRef;

    use ethabi::ethereum_types::U256;
    use xvm_helper::XvmErc721;

    type EvmContractAddress = [u8; 20];
    type RiskFactor = u32;
    type CollateralFactor = u32;

    type LoanLimit = Balance;
    type LoanOpen = Balance;
    type LoanLastChange = BlockNumber;
    type InterestRate = u32; // Interest rate / block (/1_000_000)

    type NftId = u32;

    #[derive(SpreadAllocate, Storage)]
    #[ink(storage)]
    pub struct Collateral {
        collections: Mapping<EvmContractAddress, (RiskFactor, CollateralFactor)>,
        loans: Mapping<AccountId, (LoanLimit, LoanOpen, LoanLastChange)>,
        collaterals: Mapping<AccountId,Vec<(EvmContractAddress, NftId)>>, 
        sign_transfer: SignTransferRef,
        oracle: OracleRef,
        interest_rate: InterestRate,
        #[storage_field]
        ownable: ownable::Data,
    }

    impl Ownable for Collateral {}

    impl Collateral {

        /// Constructor:
        ///  - Needs the hash of previously deployed sign-transfer contract 
        #[ink(constructor)]
        pub fn new(version: u32, sign_transfer_hash: Hash, oracle_hash: Hash, interest_rate: Option<InterestRate>) -> Self {
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
                instance.oracle = OracleRef::new()
                    .endowment(0)
                    .code_hash(oracle_hash)
                    .salt_bytes(salt)
                    .instantiate()
                    .unwrap_or_else(|error| {
                        panic!("failed at instantiating the Oracle contract: {:?}", error)
                    });
            })
        }

        /// Allows a user to deposit an NFT as collateral to increase it's loan limit
        #[ink(message)]
        pub fn deposit_nft(&mut self, evm_address: EvmContractAddress, id: NftId) -> Result<(),CollateralError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();

            let (_risk_factor, collateral_factor) = self.registered_nft_collection(evm_address)?;
            
            XvmErc721::transfer_from(evm_address, caller, contract, U256::from(id))
                .map_err(|_| CollateralError::Custom(String::from("transfer failed")))?;

            // query oracle
            let floor_price = self.oracle.get_floor_price(id.to_string())
                .map_err(|_| CollateralError::Custom(String::from("floor price retrieval failed")))?;
            // modify user loan balance
            let (loan_limit, loan_open, current_block) = self.update_loan_status(caller)?;
            let new_loan_limit = loan_limit.saturating_add(floor_price.saturating_mul(collateral_factor.into()));

            self.loans.insert(&caller, &(new_loan_limit, loan_open, current_block));

            // TODO: Verify if this is needed
            let mut caller_collaterals = self.collaterals.get(caller).unwrap_or(Vec::new());

            caller_collaterals.push((evm_address, id));
            self.collaterals.insert(&caller, &caller_collaterals);

            Ok(())
        }


        /// Allows a user to reclaim NFT to decrease it's loan balance (as long as open loan is smaller)
        #[ink(message)]
        pub fn withdraw_nft(&mut self, evm_address: EvmContractAddress, id: NftId) -> Result<(),CollateralError> {
            let caller = self.env().caller();

            //TODO: check user holds this NFT as collateral

            //TODO: check user balance allows this

            self.sign_transfer.transfer(evm_address, caller, id)

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
        pub fn take_loan(&mut self, amount: Balance ) -> Result<(), CollateralError> {
            let caller = self.env().caller();
            let (loan_limit, loan_open, _) = self.update_loan_status(caller)?;

            if loan_open + amount > loan_limit {
                return Err(CollateralError::Custom(String::from("Insufficient loan balance")));
            }

            // TODO: transfer CCoin using SignTransferRef

            self.loans.insert(&caller, &(loan_limit,loan_open+amount,self.env().block_number()));
           
            Ok(())

        }

        /// Allows user to repay previously claimed loan
        #[ink(message)] 
        pub fn repay_loan(&mut self, amount: Option<Balance>) -> Result<(), CollateralError> {
            let caller = self.env().caller();
            let (_, loan_open, _) = self.update_loan_status(caller)?;

            let _amount_to_transfer = amount.unwrap_or(loan_open);
            
            
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
        pub fn update_loan_status(&mut self, user: AccountId) -> Result<(LoanLimit, LoanOpen, LoanLastChange), CollateralError> {
            let (loan_limit, loan_open, _loan_last_change) = self.loan_status(user)?;

            let current_block = self.env().block_number();

            //TODO: commented line below is relatively correct, but causes nasty rust error, figure out work-around
            //let new_loan_open = loan_open + u128::from((loan_last_change - current_block) * self.interest_rate )* loan_open / 1_000_000;
            let new_loan_open = loan_open;

            self.loans.insert(&user, &(loan_limit, new_loan_open, current_block));

            Ok((loan_limit, new_loan_open, current_block))
        }

    }

}