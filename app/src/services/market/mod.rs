use gstd::msg;
use sails_rs::{
    collections::HashMap,
    gstd::service,
    prelude::*,
};
use crate::services::{gold_vft::GoldService, item_vmt::ItemService};
use vmt_service::utils::TokenId;
use crate::admin::Admins;

#[derive(Encode, Decode, TypeInfo)]
pub enum Event {
    PriceSet { token_id: TokenId, price: U256 },
    Purchased { buyer: ActorId, token_id: TokenId, amount: U256, price: U256 },
}

#[derive(Default)]
pub struct MarketStorage {
    prices: HashMap<TokenId, U256>,
}

static mut MARKET_STORAGE: Option<MarketStorage> = None;

#[derive(Clone)]
pub struct MarketService {}

impl MarketService {
    pub fn seed() -> Self {
        unsafe {
            let mut prices_map: HashMap<TokenId, U256> = HashMap::new();
            prices_map.insert(110.into(), 100.into());
            prices_map.insert(220.into(), 200.into());
            MARKET_STORAGE = Some(MarketStorage {
                prices: prices_map,
            });
        };
        MarketService {}
    }

    pub fn get_mut(&mut self) -> &'static mut MarketStorage {
        unsafe {
            MARKET_STORAGE
                .as_mut()
                .expect("Market storage is not initialized")
        }
    }

    pub fn get(&self) -> &'static MarketStorage {
        unsafe {
            MARKET_STORAGE
                .as_ref()
                .expect("Market storage is not initialized")
        }
    }


}

#[service(events = Event)]
impl MarketService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn set_price(&mut self, token_id: TokenId, price: U256) {
        self.ensure_is_admin();
        let storage = self.get_mut();
        storage.prices.insert(token_id, price);
        self.notify_on(Event::PriceSet { token_id, price })
            .expect("Notification Error");
    }

    pub fn buy(&mut self, token_id: TokenId, amount: U256) {
        let buyer = msg::source();
       
        let storage = self.get();
        let price = storage.prices.get(&token_id).expect("Price not set for token_id");

        let total_cost = *price * amount;
        let buyer_balance = GoldService::balance_of_mine(buyer);

        if buyer_balance < total_cost {
            panic!("Insufficient balance");
        }

        GoldService::burn_internal_notify_off(buyer, total_cost);
        ItemService::mint_internal_notify_off(ItemService::get_item(), buyer, token_id, amount);

        self.notify_on(Event::Purchased {
            buyer,
            token_id,
            amount,
            price: *price,
        })
        .expect("Notification Error");
    }

    pub fn get_price(&self, token_id: TokenId) -> Option<U256> {
        let storage = self.get();
        storage.prices.get(&token_id).cloned()
    }
}

impl MarketService {
    fn ensure_is_admin(&self) {
        if !Admins::is_admin(&msg::source()) {
            panic!("Not admin")
        };
    }
}