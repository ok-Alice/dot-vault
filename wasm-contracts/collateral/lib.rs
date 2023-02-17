#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod collateral {
    use ink_storage::{
        Mapping,
        traits::SpreadAllocate
    };
    
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
    
    pub use pallet_assets_chain_extension::{
         ink::*,
         traits::*,
     };
    //use assets_extension::*;
    
    use rand_chacha::ChaChaRng; // for mock data only
    use rand_chacha::rand_core::RngCore;
    use rand_chacha::rand_core::SeedableRng;

    type EvmContractAddress = [u8; 20];
    type RiskFactor = u32;
    type CollateralFactor = u32;

    type LoanLimit = Balance;
    type LoanOpen = Balance;
    type LoanLastChange = BlockNumber;
    type InterestRate = u32; // Interest rate / block (/1_000_000)

    type AssetId = u128;

    type NftId = u32;
    type FloorPrice = Balance;

    #[ink(event)]
    pub struct BalanceUpdate {
        account: AccountId,
        loan_limit: LoanLimit,
        loan_open: LoanOpen,
    }

    #[derive(SpreadAllocate, Storage)]
    #[ink(storage)]
    pub struct Collateral {
        collections: Mapping<EvmContractAddress, (RiskFactor, CollateralFactor)>,
        loans: Mapping<AccountId, (LoanLimit, LoanOpen, LoanLastChange)>,
        collaterals: Mapping<(AccountId,EvmContractAddress, NftId), FloorPrice>,
        sign_transfer: SignTransferRef,
        oracle: OracleRef,
        interest_rate: InterestRate,
        scoin_asset_id: AssetId,
        pallet_assets: AssetsExtension,
        using_mock: bool,
        #[storage_field]
        ownable: ownable::Data,
    }

    impl Ownable for Collateral {}

    impl Collateral {

        /// Constructor:
        ///  - Needs the hash of previously deployed sign-transfer and oracle contract
        #[ink(constructor)]
        pub fn new(version: u32, 
            sign_transfer_hash: Hash, 
            oracle_hash: Hash, 
            using_mock: bool,
            scoin_asset_id: Option<AssetId>,
            interest_rate: Option<InterestRate>
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_owner(caller);

                instance.interest_rate = interest_rate.unwrap_or(15);
                instance.using_mock = using_mock;
                instance.scoin_asset_id = scoin_asset_id.unwrap_or(4242);

                let salt = version.to_le_bytes();
                instance.sign_transfer = SignTransferRef::new()
                    .endowment(0)
                    .code_hash(sign_transfer_hash)
                    .salt_bytes(salt)
                    .instantiate()
                    .unwrap_or_else(|error| {
                        panic!("failed at instantiating the Sign Transfer contract: {:?}", error)
                    });

                
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
        pub fn deposit_nft(&mut self, evm_address: EvmContractAddress, id: NftId) -> Result<(), CollateralError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();

            let (_risk_factor, collateral_factor) = self.registered_nft_collection(evm_address)?;
            
            XvmErc721::transfer_from(evm_address, caller, contract, U256::from(id))
                .map_err(|_| CollateralError::Custom(String::from("transfer failed")))?;

            // query oracle
            let query = ink_env::format!("nft/0x{}", hex::encode(&evm_address));
            let floor_price = self.oracle.get_floor_price(query)
                .map_err(|_| CollateralError::Custom(String::from("floor price retrieval failed")))?;

            // modify user loan balance
            let (loan_limit, loan_open, current_block) = self.update_loan_status(caller)?;
            let new_loan_limit = loan_limit.saturating_add(floor_price.saturating_mul(collateral_factor.into()));

            self.loans_insert(&caller, new_loan_limit, loan_open, current_block);

            self.collaterals.insert(&(caller, evm_address, id), &floor_price); 

            Ok(())
        }


        /// Allows a user to reclaim NFT to decrease it's loan balance (as long as open loan is smaller)
        #[ink(message)]
        pub fn withdraw_nft(&mut self, evm_address: EvmContractAddress, id: NftId) -> Result<(), CollateralError> {
            let caller = self.env().caller();
            let (_risk_factor, collateral_factor) = self.registered_nft_collection(evm_address)?;

            // check user holds this NFT as collateral
            let floor_price = match self.collaterals.get((caller, evm_address, id)) {
                Some(p) =>  p,
                None => return  Err(CollateralError::Custom(String::from("Not owner of NFT.")))
            };

            // check user balance allows this
            let (loan_limit, loan_open, current_block) = self.update_loan_status(caller)?;
            let new_loan_limit = loan_limit.saturating_sub(floor_price.saturating_mul(collateral_factor.into()));

            if new_loan_limit < loan_open {
                return Err(CollateralError::Custom(String::from("open loan is too large to allow withdrawal of nft")));
            }

            self.sign_transfer.transfer_nft(evm_address, caller, id)?;
            self.collaterals.remove(&(caller, evm_address, id));

            // modify user loan limit
            self.loans_insert(&caller, new_loan_limit, loan_open, current_block);


            Ok(())
        }

        /// Allows admin to add NFT collection to list of allowed collections
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn register_nft_collection(&mut self, evm_address: EvmContractAddress, risk_factor: RiskFactor, collateral_factor: CollateralFactor) -> Result<(),CollateralError> {
            if self.using_mock {
                self.mock_populate_oracle(evm_address)?;
            }

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
            let contract = self.env().account_id();
            // let (loan_limit, loan_open, _) = self.update_loan_status(caller)?;

            // if loan_open.saturating_add(amount) > loan_limit {
            //     return Err(CollateralError::Custom(String::from("Insufficient loan balance")));
            // }

            self.pallet_assets.approve_transfer(Origin::Address, self.scoin_asset_id, caller, amount)
                 .map_err(|_| CollateralError::Custom("approve failed".into()))?;
            self.pallet_assets.transfer(Origin::Address, self.scoin_asset_id, caller, amount)
                .map_err(|_| CollateralError::Custom("transfer failed".into()))?;

            // self.loans.insert(&caller, &(loan_limit, loan_open.saturating_add(amount), self.env().block_number()));

            Ok(())

        }

        /// Allows user to repay previously claimed loan
        #[ink(message)] 
        pub fn repay_loan(&mut self, amount: Option<Balance>) -> Result<(), CollateralError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();
            let (loan_limit, loan_open, loan_last_change) = self.update_loan_status(caller)?;

            let mut amount_to_transfer = amount.unwrap_or(loan_open);
            
            if amount_to_transfer > loan_open {
                amount_to_transfer = loan_open;
            }

            if amount_to_transfer == 0 {
                return Err(CollateralError::Custom(String::from("Amount cannot be 0")))
            }

            // self.pallet_assets.approve_transfer(Origin::Caller, self.scoin_asset_id, contract, amount)
            //     .map_err(|_| CollateralError::Custom("transfer failed".into()))?;
            self.pallet_assets.transfer(Origin::Caller, self.scoin_asset_id, contract, amount_to_transfer)
                .map_err(|_| CollateralError::Custom("transfer failed".into()))?;

            self.loans.insert(&caller, &(loan_limit, loan_open - amount_to_transfer, loan_last_change));

            Ok(())
        }

        /// Allows anyone to check loan status of given user
        #[ink(message)]
        pub fn loan_status(&self, user: AccountId) -> Result<(LoanLimit, LoanOpen, LoanLastChange),CollateralError> {
            match self.loans.get(&user) {
                Some((l,o,c)) => Ok((l,o,c)),
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
            let (loan_limit, loan_open, loan_last_change) = match self.loans.get(&user) {
                Some((l,o,c)) => (l,o,c),   // existing user
                None => return Ok((0,0,self.env().block_number())), // new user
            };
            
            self.loan_status(user)?;

            let current_block = self.env().block_number();

            //TODO: using u64 cast as u128 gives 'Validation of the Wasm failed' error, figure out why 
            // formula: new_loan_open = loan_open + ((loan_last_change - current_block) * self.interest_rate )* loan_open / 1_000_000
            let interest = u64::from(loan_last_change).saturating_sub(current_block.into()).saturating_mul(self.interest_rate.into()).saturating_mul(loan_open as u64).saturating_div(1_000_000);
            let new_loan_open = loan_open.saturating_add(interest.into());

            self.loans_insert(&user, loan_limit, new_loan_open, current_block);

            Ok((loan_limit, new_loan_open, current_block))
        }

        pub fn loans_insert(&mut self, account: &AccountId, loan_limit: LoanLimit, loan_open: LoanOpen, block: BlockNumber) {
            self.loans.insert(&account, &(loan_limit, loan_open, block));

            Self::env().emit_event(BalanceUpdate {
                account: *account,
                loan_limit: loan_limit,
                loan_open: loan_open,
            });
        }

        #[ink(message)]
        pub fn mock_populate_oracle(&mut self, evm_address: EvmContractAddress) -> Result<(),CollateralError> {
            let key = ink_env::format!("nft/0x{}", hex::encode(&evm_address));

            let value = ink_env::format!("{}", self.get_random_number(1_000_000, 2_000_000));

            OracleRef::set(&mut self.oracle, key, value);
            
            Ok(())
        }

        fn get_random_number(&self, min: u64, max: u64) -> u64 {
            let random_seed = self.env().random(self.env().caller().as_ref());
            let mut seed_converted: [u8; 32] = Default::default();
            seed_converted.copy_from_slice(random_seed.0.as_ref());
            let mut rng = ChaChaRng::from_seed(seed_converted);
            ((rng.next_u64() / u64::MAX) * (max - min) + min) as u64
        }


        #[ink(message)]
        pub fn test_query_oracle(&self, evm_address: EvmContractAddress) -> Result<u128, CollateralError> {
            let key = ink_env::format!("nft/0x{}", hex::encode(&evm_address));
            
            let value = self.oracle.get(key.clone());
            let msg = ink_env::format!("{} : {}", key, value);
            Err(CollateralError::Custom(String::from(msg)))


        }

        #[ink(message)]
        pub fn test_accountid(&self) -> String {
            let account_id_self = self.env().account_id();
            let account_id_oracle = self.oracle.account_id();
            let account_id_signtransfer = self.sign_transfer.account_id();
            let msg = ink_env::format!("Collateral: {:?} Oracle: {:?} SignTransfer: {:?}", 
                    account_id_self, account_id_oracle, account_id_signtransfer);

            return String::from(msg);
        }

    }
}