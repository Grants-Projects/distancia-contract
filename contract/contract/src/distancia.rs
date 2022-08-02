
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{HashMap, LazyOption};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, PromiseOrValue};

pub const TOKEN_CONTRACT: &str = "token.distancia.testnet";
pub const XCC_GAS: Gas = Gas(20000000000000);

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Distancia {
    distancia_price: u64,

    ads_watched: LookUpMap<AccountId, Vec<String>>,
}

#ext_contract[(ext_token_contract)]
trait TokenTransfer {
    fn get_token_owner(&self);

    fn transfer_tokens(&self, from: AccountId, to: AccountId, amount: Balance, memo: Option<String>);

    fn ft_balance_of(&self, account: AccountId);
}

impl Default for Distancia {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Distancia {
    pub const token_contract: AccountId = AccountId::from_str(TOKEN_CONTRACT).unwrap();

    pub fn new(
        distancia_price: u64
    ) -> Self {
        let mut this = Self {
            distancia_price: &distancia_price,
            ads_watched: LookUpMap::new(b"s".to_vec()),
        };

        this
    }

    fn get_token_contract_owner() -> Promise {
        ext_token_contract::ext(&token_contract)
            .get_token_owner()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .callback_promise_result()
            )
    }

    fn get_address_token_balance() -> Promise {
        ext_token_contract::ext(&token_contract)
            .ft_balance_of()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .callback_promise_result()
            )
    }

    pub fn ad_watched(&self, amount: U128, ad_key: String) -> Promise {
        
        let account_id = env::signer_account_id();

        let token_contract_owner: AccountId = Self::get_token_contract_owner();

        ext_token_contract::ext(&token_contract)
            .transfer_tokens(token_contract_owner, &account_id, amount)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .callback_promise_result()
            );

        let ads: Vec<String> = ads_watched.get(&account_id);
        ads.push(ad_key.to_string());
        ads_watched.insert(account_id, ads);
    }

    #[payable]
    pub fn convert_distancia(&self, distancia_amount: U128) {
        token.ft_balance_of(accounts(2)).0
        let account_id = env::signer_account_id();
        let token_contract_owner: AccountId = Self::get_token_contract_owner();
        ext_token_contract::ext(&token_contract)
            .transfer_tokens(token_contract_owner, &account_id, amount)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .callback_promise_result()
            );

        
        let near_amount = (distancia_amount.0)/self.distancia_price;
        Promise::new(account_id).transfer(near_amount);
    }



}