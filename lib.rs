#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod alephbunny_token {
    use openbrush::traits::String;
    use openbrush::{
        contracts::psp22::extensions::metadata::*,
        traits::{
            AccountIdExt,
            Storage
        }
    };

    use ink::codegen::Env;

    use ink::storage::Mapping;

    use ink::prelude::vec::Vec;

    use ink::prelude::vec;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct AlephbunnyToken {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
        creator: Vec<AccountId>,
        marketing_wallet: Vec<AccountId>,
        fee_privileges: Mapping<AccountId, bool>,
        airdrop: Mapping<AccountId, Balance>,
        privileged_accounts: Vec<AccountId>,
        disbursement_pool:Balance,
        airdrop_start_time:u64,
        is_allowed_to_control:bool
    }

    impl PSP22 for AlephbunnyToken {}

    impl psp22::Internal for AlephbunnyToken {
        fn _transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            if from.is_zero() {
                return Err(PSP22Error::ZeroSenderAddress)
            }
            if to.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress)
            }
    
            let mut from_balance = self._balance_of(&from);

            let airdrop_balance = self.get_airdrop(from);

            if self.get_airdrop_start_time() <= self.env().block_timestamp() {
                from_balance = from_balance;
            }
            else {
                if airdrop_balance > 0 {
                    from_balance -= airdrop_balance;
                }
            }
    
            if from_balance < amount {
                return Err(PSP22Error::InsufficientBalance)
            }
    
            let exclusion_from = self.fee_privileges.get(&from).unwrap_or(false);
            let exclusion_to = self.fee_privileges.get(&to).unwrap_or(false);
    
            self.psp22._before_token_transfer(Some(&from), Some(&to), &amount)?;
    
            self.psp22.balances.insert(&from, &(from_balance - amount));
    
            let mut fee = 0;
    
            if exclusion_from == false && exclusion_to == false {
                fee = (7 * amount) / 100;
            }
    
            let addition = amount - fee;
    
            self._do_safe_transfer_check(&from, &to, &addition, &data)?;
    
            let to_balance = self._balance_of(&to);
    
            self.psp22.balances.insert(&to, &(to_balance + addition));
    
            self.disbursement_pool += fee;
    
            if self.disbursement_pool >= 10u128.pow(12) {
                self.disbursements();
            }
    
            self._after_token_transfer(Some(&from), Some(&to), &addition)?;
            self._emit_transfer_event(Some(from), Some(to), amount);
    
            Ok(())
        }
    }

    impl PSP22Metadata for AlephbunnyToken {}

    impl AlephbunnyToken {
        #[ink(constructor)]
        pub fn new(total_supply: Balance, creator:AccountId, marketing_wallet: AccountId) -> Self {
            let mut instance = Self::default();
            
            instance.metadata.name = Some(String::from("Aleph Bunnies"));
            instance.metadata.symbol = Some(String::from("BUNNY"));
            instance.metadata.decimals = 6;
            instance.creator = vec![creator];
            instance.marketing_wallet = vec![marketing_wallet];
            instance.airdrop_start_time = 1681516800000;
            instance._exclude_from_fees(creator);
            instance._exclude_from_fees(marketing_wallet);
            instance
                ._mint_to(creator, total_supply)
                .expect("Should mint total_supply");
            
            instance
        }
        #[ink(message)]
        pub fn exclude_from_fees(&mut self, account:AccountId) {
            if Self::env().caller() == self.creator[usize::try_from(0).unwrap()] {
                self._exclude_from_fees(account);
            }
        }
        #[ink(message)]
        pub fn add_to_airdrop(&mut self, account:AccountId, amount:Balance) {
            if Self::env().caller() == self.creator[usize::try_from(0).unwrap()] {
                self.airdrop.insert(&account, &amount);
                let balance = self._balance_of(&account);
                let sender_balance = self._balance_of(&Self::env().caller());
                self.psp22.balances.insert(&Self::env().caller(), &(sender_balance - amount));
                self.psp22.balances.insert(&account, &(balance + amount));
            }
        }
        #[ink(message)]
        pub fn get_airdrop(&self, account:AccountId) -> Balance {
            self.airdrop.get(&account).unwrap_or(0)
        }
        #[ink(message)]
        pub fn get_airdrop_start_time(&self) -> u64 {
            self.airdrop_start_time
        }
        #[inline]
        pub fn _exclude_from_fees(&mut self, account:AccountId) {
            let included = self.fee_privileges.get(&account).unwrap_or(false);
            if included == false {
                self.fee_privileges.insert(&account, &true);
                self.privileged_accounts.push(account);
            }
        }
        #[ink(message)]
        pub fn circulating_supply(&self) -> Balance {
            let privileged_accounts = &self.privileged_accounts;
            let mut balances = 0;
            for i in 0..= privileged_accounts.len() - 1 {
                balances += self.psp22._balance_of(&privileged_accounts[i]);
            }
            let circulating = self.psp22.total_supply() - balances;
            circulating
        }
        #[ink(message)]
        pub fn get_privileged_accounts(&self) -> Vec<AccountId> {
            let privileged_accounts = &self.privileged_accounts;
            privileged_accounts.to_vec()
        }
        #[inline]
        pub fn disbursements(&mut self) {
            
            let sharing = self.disbursement_pool;

            let marketing_balance = self._balance_of(&self.marketing_wallet[usize::try_from(0).unwrap()]);

            self.psp22.balances.insert(&self.marketing_wallet[usize::try_from(0).unwrap()], &(sharing + marketing_balance));
    
            self.disbursement_pool -= sharing;
    
        }
    }
}