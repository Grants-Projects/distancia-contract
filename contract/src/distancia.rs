
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LazyOption};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Gas, PromiseOrValue, Promise, PromiseResult, ext_contract, require};

pub const TOKEN_CONTRACT: &str = "token.distancia.testnet";
pub const XCC_GAS: Gas = Gas(20000000000000);

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Milestone {
    id: U128,
    milestone_centralized_identifier: String,
    milestone_key: String,
    value: Balance
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Ad {
    id: U128,
    ad_centralized_identifier: String,
    media_url: String,
    ad_key: String,
    value: Balance
}

#[ext_contract(ext_token_contract)]
pub trait DistanciaToken {
    fn get_token_owner(&self) -> AccountId;

    fn mint_tokens_on_ad_watched(&self, account_id: AccountId, amount: Balance);

    fn burn_tokens_on_convert(&self, account_id: AccountId, amount: Balance);

    fn ft_balance_of(&self, account_id: &AccountId) -> U128;


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

    ads_watched: LookupMap<AccountId, Vec<U128>>,

    milestones: Vec<Milestone>,

    ads: Vec<Ad>,

    ads_by_key: LookupMap<String, Ad>,

    milestones_by_key: LookupMap<String, Milestone>,

    owner: AccountId
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
            milestones: Vec::new(),
            ads: Vec::new(),
            ads_by_key: LookupMap::new(b"a".to_vec()),
            milestones_by_key: LookupMap::new(b"m".to_vec()),
            owner: env::signer_account_id()
        };

        this
    }

    pub fn upload_ad(&mut self, ad_key: String, media_url: String, centralized_identifier: String) {

        require!(self.get_ad_by_key(ad_key.clone()).is_none(), "Ad with supplied key already exists");

        let val = env::attached_deposit();

        let ad_id = U128::from(u128::from(self.ads.len() as u64) + 1);

        let ad = Ad {
            id: ad_id,
            ad_centralized_identifier: centralized_identifier,
            media_url: media_url,
            value: val,
            ad_key: ad_key.clone()
        };

        self.ads_by_key.insert(&ad_key, &ad);

        self.ads.push(ad);

    }

    

    
    pub fn create_milestone(&mut self, milestone_key: String, valuation: Balance, centralized_identifier: String) {
        require!(env::signer_account_id() == self.owner, "Not authorized");
        let milestone_id = U128::from(u128::from(self.milestones.len() as u64) + 1);

        let milestone = Milestone {
            id: milestone_id,
            milestone_key: milestone_key.clone(),
            milestone_centralized_identifier: centralized_identifier,
            value: valuation
        };

        self.milestones_by_key.insert(&milestone_key, &milestone);

        self.milestones.push(milestone);
    }

    pub fn ad_watched(&mut self, amount: Balance, ad_key: String) {

        let account_id = env::signer_account_id();

        if let Some(ad) = self.get_ad_by_key(ad_key) {

            ext_token_contract::ext(AccountId::new_unchecked(TOKEN_CONTRACT.to_string()))
            .mint_tokens_on_ad_watched(account_id.clone(), amount.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .on_mint_tokens_callback(&account_id, ad.id)
            );
        }
    }

    
    pub fn convert_distancia(&mut self, distancia_amount: Balance, milestone_cleared: bool) {
        
        let account_id = env::signer_account_id();
        let near_amount: u128;
        
        ext_token_contract::ext(AccountId::new_unchecked(TOKEN_CONTRACT.to_string()))
            .burn_tokens_on_convert(account_id.clone(), distancia_amount.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .on_burn_tokens_callback()
            );

        
        if milestone_cleared {
            near_amount = (distancia_amount) * 12/(self.distancia_price * 10);
        } else {
            near_amount = (distancia_amount)/self.distancia_price;
        }
        
        Promise::new(account_id).transfer(near_amount);
    }


    pub fn clear_milestone(&mut self, milestone_key: String) {

        if let Some(milestone) = self.get_milestone_by_key(milestone_key) {
            let distancia_amount = milestone.value;
            self.convert_distancia(distancia_amount, true);
        } else {
            env::panic_str("Milestone doesnt exist");
        }
        
    }

    

    pub fn get_ads(&self) -> &Vec<Ad> {
        &self.ads
    }

    pub fn get_milestones(&self) -> &Vec<Milestone> {
        &self.milestones
    }

    pub fn get_token_contract_owner(&self) -> Promise {
        
        ext_token_contract::ext(AccountId::new_unchecked(TOKEN_CONTRACT.to_string()))
            .get_token_owner()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .on_get_token_owner_callback()
            )
        
    }



    pub fn get_distancia_price(&self) -> u128 {
        self.distancia_price
    }

    pub fn get_ads_watched(&self, account_id: AccountId) -> Vec<&Ad> {
        let ad_ids = self.ads_watched.get(&account_id).unwrap_or(Vec::new());

        ad_ids.into_iter().map(|ad_id| &self.ads[(ad_id.0 - 1) as usize]).collect::<Vec<&Ad>>()
    }

    #[private]
    pub fn get_ad_by_key(&self, ad_key: String) -> Option<Ad> {

        self.ads_by_key.get(&ad_key)
    }

    #[private]
    pub fn get_milestone_by_key(&self, milestone_key: String) -> Option<Milestone> {

        self.milestones_by_key.get(&milestone_key)
    }

    #[private]
    pub fn on_get_token_owner_callback(&mut self, #[callback_unwrap] owner: AccountId) {
        if self.token_contract_owner != env::current_account_id() {
            self.token_contract_owner = owner;
        }
    }

    #[private]
    pub fn on_mint_tokens_callback(&mut self, account_id: &AccountId, ad_id: U128) {
        let mut ads: Vec<U128> = self.ads_watched.get(&account_id).unwrap_or(Vec::new());
        ads.push(ad_id);
        self.ads_watched.insert(&account_id, &ads);
    }

    #[private]
    pub fn on_burn_tokens_callback(&mut self) {}

}