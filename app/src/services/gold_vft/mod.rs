use gstd::msg;
use sails_rs::{gstd::service, prelude::*};
mod funcs;
use crate::services;
use vft_service::{Service as VftService, Storage};
use crate::admin::Admins;

#[derive(Encode, Decode, TypeInfo)]
pub enum Event {
    Minted { to: ActorId, value: U256 },
    Burned { from: ActorId, value: U256 },
}

#[derive(Clone)]
pub struct GoldService {
    vft: VftService,
}

impl GoldService {
    pub fn seed() -> Self {
        GoldService {
            vft: VftService::seed("Game Gold".to_owned(), "GOLD".to_owned(), 2),
        }
    }
}

#[service(extends = VftService, events = Event)]
impl GoldService {
    pub fn new() -> Self {
        Self {
            vft: VftService::new(),
        }
    }

    pub fn mint(&mut self, to: ActorId, value: U256) -> bool {
        self.ensure_is_admin();
        self.mint_internal(to, value)
    }

    pub fn burn(&mut self, from: ActorId, value: U256) -> bool {
        self.ensure_is_admin();
        self.burn_internal(from, value)
    }
}

impl GoldService {
    fn ensure_is_admin(&self) {
        if !Admins::is_admin(&msg::source()) {
            panic!("Not admin")
        };
    }

    // Internal mint method without admin check
    pub fn mint_internal(&mut self, to: ActorId, value: U256) -> bool {
        let mutated = services::utils::panicking(|| {
            funcs::mint(Storage::balances(), Storage::total_supply(), to, value)
        });
        if mutated {
            self.notify_on(Event::Minted { to, value })
                .expect("Notification Error");
        }
        mutated
    }

    // Internal burn method without admin check
    pub fn burn_internal(&mut self, from: ActorId, value: U256) -> bool {
        let mutated = services::utils::panicking(|| {
            funcs::burn(Storage::balances(), Storage::total_supply(), from, value)
        });
        if mutated {
            self.notify_on(Event::Burned { from, value })
                .expect("Notification Error");
        }
        mutated
    }

    pub fn burn_internal_notify_off(from: ActorId, value: U256) -> bool {
        let mutated = services::utils::panicking(|| {
            funcs::burn(Storage::balances(), Storage::total_supply(), from, value)
        });
        mutated
    }

    pub fn mint_internal_notify_off( to: ActorId, value: U256) -> bool {
        let mutated = services::utils::panicking(|| {
            funcs::mint(Storage::balances(), Storage::total_supply(), to, value)
        });
        mutated
    }

    pub fn balance_of_mine( owner: ActorId) -> U256 {
        Storage::balances().get(&owner).cloned().unwrap_or_default()
    }

}

impl AsRef<VftService> for GoldService {
    fn as_ref(&self) -> &VftService {
        &self.vft
    }
}

