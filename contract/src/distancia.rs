
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LazyOption};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Gas, PromiseOrValue, Promise, PromiseResult, ext_contract, require};

pub const TOKEN_CONTRACT: &str = "token.distancia.testnet";
pub const XCC_GAS: Gas = Gas(20000000000000);

#[ext_contract(ext_token_contract)]
pub trait DistanciaToken {
    fn get_token_owner(&self) -> AccountId;

    fn transfer_tokens(&self, from: &AccountId, to: &AccountId, amount: &Balance, memo: Option<String>);

    fn ft_balance_of(&self, account: &AccountId) -> U128;
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_get_token_owner_callback(&mut self) -> AccountId;

    fn on_get_token_balance_callback(&mut self) -> U128;
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Distancia {
    distancia_price: u128,

    token_contract_owner: AccountId,

    ads_watched: LookupMap<AccountId, Vec<String>>,
}



impl Default for Distancia {
    fn default() -> Self {
        env::panic_str("Contract should be initialized before usage")
    }
}


#[near_bindgen]
impl Distancia {

    #[init]
    pub fn new(
        distancia_price: u128
    ) -> Self {
        let this = Self {
            distancia_price: distancia_price,
            token_contract_owner: env::current_account_id(),
            ads_watched: LookupMap::new(b"s".to_vec()),
        };

        this
    }

    #[private]
    pub fn on_get_token_owner_callback(&mut self, #[callback_unwrap] owner: AccountId) {
        if self.token_contract_owner != env::current_account_id() {
            self.token_contract_owner = owner;
        }
    }

    #[private]
    pub fn on_transfer_tokens_callback(&mut self) {
        
    }

    fn get_token_contract_owner(&self) -> Promise {
        
        ext_token_contract::ext(AccountId::new_unchecked(TOKEN_CONTRACT.to_string()))
            .get_token_owner()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .on_get_token_owner_callback()
            )
        
    }

    // fn get_address_token_balance(&self, account: &AccountId) -> Promise {
    //     let balance: U128;
    //     ext_token_contract::ext(AccountId::new_unchecked(TOKEN_CONTRACT.to_string()))
    //         .ft_balance_of(account)
    //         .then(
    //             Self::ext(env::current_account_id())
    //                 .with_static_gas(XCC_GAS)
    //                 .on_get_token_balance_callback()
    //         )
       
    // }

    pub fn ad_watched(&mut self, amount: U128, ad_key: String) {
        
        let account_id = env::signer_account_id();

        ext_token_contract::ext(AccountId::new_unchecked(TOKEN_CONTRACT.to_string()))
            .transfer_tokens(&self.token_contract_owner, &account_id, &amount.0, Option::from("Transferring".to_string()))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .on_transfer_tokens_callback()
            );

        let mut ads: Vec<String> = self.ads_watched.get(&account_id).unwrap();
        ads.push(ad_key);
        self.ads_watched.insert(&account_id, &ads);
    }

    
    pub fn convert_distancia(&mut self, distancia_amount: U128, milestone_cleared: bool) {
        
        let account_id = env::signer_account_id();
        let near_amount: u128;
        // let balance: U128 = self.get_address_token_balance(&account_id);
        // require!(balance.0 >= distancia_amount.0, "Not enough tokens");
        ext_token_contract::ext(AccountId::new_unchecked(TOKEN_CONTRACT.to_string()))
            .transfer_tokens(&account_id, &self.token_contract_owner, &distancia_amount.0, Option::from("Transferring".to_string()))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .on_transfer_tokens_callback()
            );

        
        if milestone_cleared {
            near_amount = (distancia_amount.0) * 12/(self.distancia_price * 10);
        } else {
            near_amount = (distancia_amount.0)/self.distancia_price;
        }
        
        Promise::new(account_id).transfer(near_amount);
    }


    pub fn clear_milestone(&mut self, distancia_amount: U128) {
        self.convert_distancia(distancia_amount, true);
    }

    pub fn get_distancia_price(&self) -> u128 {
        self.distancia_price
    }

    pub fn get_ads_watched(&self, account_id: AccountId) -> Vec<String> {
        self.ads_watched.get(&account_id).unwrap()
    }

}