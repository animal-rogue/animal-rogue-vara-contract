#![no_std]

use sails_rs::prelude::*;
use services::*;

mod services;

pub struct AnimalRogueProgram(());

#[sails_rs::program]
impl AnimalRogueProgram {
    // Program's constructor
    pub fn new() -> Self {
        <admin::Service>::seed();
        <gold_vft::GoldService>::seed();
        <item_vmt::ItemService>::seed();
        <market::MarketService>::seed();
        <game::GameService>::seed();
        Self(())
    }

    // admin service
    pub fn admin(&self) -> admin::Service {
        admin::Service::new()
    }

    // gold service
    pub fn vft(&self) -> gold_vft::GoldService {
        gold_vft::GoldService::new()
    }

    // item service
    pub fn vmt(&self) -> item_vmt::ItemService {
        item_vmt::ItemService::new()
    }

    // market service
    pub fn market(&self) -> market::MarketService {
        market::MarketService::new()
    }

    // game service
    pub fn game(&self) -> game::GameService {
        game::GameService::new()
    }
}
