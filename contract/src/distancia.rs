
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LazyOption, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Gas, PromiseOrValue, Promise, PromiseResult, ext_contract, require};

pub const TOKEN_CONTRACT: &str = "token.distancia.testnet";
pub const XCC_GAS: Gas = Gas(20000000000000);

#[derive(Clone, BorshDeserialize, BorshSerialize)]
pub struct Milestone {
    id: U128,
    milestone_key: String,
    value: Balance
}

#[derive(Clone, BorshDeserialize, BorshSerialize)]
pub struct Ad {
    id: U128,
    metadata: String,
    owner: AccountId,
    ad_key: String,
    value: Balance,
    watch_value: Balance,
    watchers_allowed: u128,
    watched_count: u128
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

    //minimum ad value that can be set for any ad
    minimum_ad_value: u128,

    //percentage of ad value that should be distributed to ad watchers: approxed to 4 decimal precision (100 * 10000)
    percentage_ad_watch_value: u128,

    //percentage interest received when milestone is completed: approxed to 4 decimal precision (100 * 10000)
    percentage_milestone_completion_value: u128,

    //percentage commission consumed from ad value payment: approxed to 4 decimal precision (100 * 10000)
    percentage_commission_value: u128,

    token_contract_owner: AccountId,

    ads_watched: LookupMap<AccountId, Vector<U128>>,

    milestones: Vector<Milestone>,

    ads: Vector<Ad>,

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
        distancia_price: U128,
        minimum_ad_value: U128,
        percentage_ad_watch_value: U128,
        percentage_milestone_completion_value: U128,
        percentage_commission_value: U128
    ) -> Self {
        let this = Self {
            distancia_price: distancia_price.0,
            minimum_ad_value: minimum_ad_value.0,
            percentage_ad_watch_value: percentage_ad_watch_value.0,
            percentage_milestone_completion_value: percentage_milestone_completion_value.0,
            percentage_commission_value: percentage_commission_value.0,
            token_contract_owner: env::current_account_id(),
            ads_watched: LookupMap::new(b"s".to_vec()),
            milestones: Vector::new(b"k".to_vec()),
            ads: Vector::new(b"t"),
            ads_by_key: LookupMap::new(b"a".to_vec()),
            milestones_by_key: LookupMap::new(b"m".to_vec()),
            owner: env::signer_account_id()
        };

        this
    }

    #[result_serializer(borsh)]
    pub fn upload_ad(&mut self, ad_key: String, metadata: String) -> Ad {

        require!(self.get_ad_by_key(ad_key.clone()).is_none(), "Ad with supplied key already exists");

        let val = env::attached_deposit();

        require!(val >= self.minimum_ad_value, "Ad value offered is very low");

        let amount_to_pay = (val * (1000000 - self.percentage_commission_value.clone()))/1000000;

        //1000000 to balance precision of percentage_ad_watch_value
        let watch_value = amount_to_pay * self.percentage_ad_watch_value.clone() * self.distancia_price.clone() / (1000000 + self.percentage_milestone_completion_value);

        let watchers_allowed = 1000000 / self.percentage_ad_watch_value.clone();

        let ad_id = U128::from(u128::from(self.ads.len()) + 1);

        let ad = Ad {
            id: ad_id,
            metadata: metadata,
            owner: env::signer_account_id(),
            value: val,
            ad_key: ad_key.clone(),
            watch_value: watch_value,
            watchers_allowed: watchers_allowed,
            watched_count: 0
        };

        self.ads_by_key.insert(&ad_key, &ad);

        self.ads.push(&ad);

        ad

    }

    

    #[result_serializer(borsh)]
    pub fn create_milestone(&mut self, milestone_key: String, valuation: Balance) -> Milestone {
        require!(env::signer_account_id() == self.owner, "Not authorized");
        let milestone_id = U128::from(u128::from(self.milestones.len()) + 1);

        let milestone = Milestone {
            id: milestone_id,
            milestone_key: milestone_key.clone(),
            value: valuation
        };

        self.milestones_by_key.insert(&milestone_key, &milestone);

        self.milestones.push(&milestone);

        milestone
    }

    pub fn ad_watched(&mut self, ad_key: String) {

        let account_id = env::signer_account_id();

        if let Some(ad) = self.get_ad_by_key(ad_key) {

            require!(ad.owner != env::signer_account_id(), "Can not get benefits from ads watched.");

            require!(ad.watchers_allowed > ad.watched_count, "Value on ad already fully redeemed");

            ext_token_contract::ext(AccountId::new_unchecked(TOKEN_CONTRACT.to_string()))
            .mint_tokens_on_ad_watched(account_id.clone(), ad.watch_value.clone())
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
            near_amount = (distancia_amount) * (1000000 + self.percentage_milestone_completion_value)/(self.distancia_price * 1000000);
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


    pub fn set_minimum_ad_value(&mut self, new_min_ad_value: u128) {
        require!(env::signer_account_id() == self.owner, "Not authorized");

        self.minimum_ad_value = new_min_ad_value;
    }

    pub fn set_percentage_ad_watch_value(&mut self, new_percentage_ad_watch_value: u128) {
        require!(env::signer_account_id() == self.owner, "Not authorized");

        self.percentage_ad_watch_value = new_percentage_ad_watch_value;
    }

    pub fn set_percentage_commission_value(&mut self, new_percentage_commission_value: u128) {
        require!(env::signer_account_id() == self.owner, "Not authorized");

        self.percentage_commission_value = new_percentage_commission_value;
    }

    pub fn set_percentage_milestone_completion_value(&mut self, new_percentage_milestone_completion_value: u128) {
        require!(env::signer_account_id() == self.owner, "Not authorized");

        self.percentage_milestone_completion_value = new_percentage_milestone_completion_value;
    }

    pub fn set_distancia_price(&mut self, new_distancia_price: u128) {
        require!(env::signer_account_id() == self.owner, "Not authorized");

        self.distancia_price = new_distancia_price;
    }

    
    #[result_serializer(borsh)]
    pub fn get_ads(&self) -> Vec<Ad> {

        self.ads.iter().map(|ad| ad).collect()
    }

    #[result_serializer(borsh)]
    pub fn get_milestones(&self) -> Vec<Milestone> {
        self.milestones.iter().map(|ad| ad).collect()
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

    #[result_serializer(borsh)]
    pub fn get_ads_watched(&self, account_id: AccountId) -> Vector<Ad> {
        let ad_ids = self.ads_watched.get(&account_id).unwrap_or(Vector::new(b"p"));

        let mut ads = Vector::new(b"z");

        if !ad_ids.is_empty() {
        
            for id in ad_ids.iter() {
                ads.push(&self.ads.get((id.0 - 1) as u64).unwrap());
            }
            
        }

        ads

        //ad_ids.iter().map(|ad_id| self.ads.get((ad_id.0 - 1) as u64).unwrap()).into()
    }

    #[private]
    #[result_serializer(borsh)]
    pub fn get_ad_by_key(&self, ad_key: String) -> Option<Ad> {

        self.ads_by_key.get(&ad_key)
    }

    #[private]
    #[result_serializer(borsh)]
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
        let mut ads = self.ads_watched.get(&account_id).unwrap_or(Vector::new(b"p"));
        ads.push(&ad_id);
        self.ads_watched.insert(&account_id, &ads);

        let mut ad = self.ads.get((ad_id.0 - 1) as u64).unwrap();

        ad.watched_count += 1;
        self.ads_by_key.insert(&ad.ad_key, &ad);
        self.ads.replace((ad_id.0 - 1) as u64, &ad);
    }

    #[private]
    pub fn on_burn_tokens_callback(&mut self) {}

}