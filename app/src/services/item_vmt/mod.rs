use gstd::msg;
use sails_rs::{
    collections::HashMap,
    gstd::service,
    prelude::*,
};
mod funcs;
use crate::services;
pub mod utils;
use utils::*;
use vmt_service::{Service as VmtService, Storage};
use crate::admin::Admins;

#[derive(Default)]
pub struct ItemStorage {
    token_metadata: HashMap<TokenId, TokenMetadata>,
    owners: HashMap<TokenId, ActorId>,
}

static mut EXTENDED_STORAGE: Option<ItemStorage> = None;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    Minted {
        to: ActorId,
        ids: Vec<TokenId>,
        amounts: Vec<U256>,
    },
    Burned {
        from: ActorId,
        ids: Vec<TokenId>,
        amounts: Vec<U256>,
    },
}

#[derive(Clone)]
pub struct ItemService {
    vmt: VmtService,
}

impl ItemService {
    pub fn seed() -> Self {
        unsafe {

            let mut token_metadata: HashMap<TokenId, TokenMetadata> = HashMap::new();
            let candy_metadata = TokenMetadata {
                title: Some("Candy".to_string()),
                description: Some("Candy".to_string()),
                media: None,
                reference: None,
                // Replace with actual field name and value
            };
            let hummer_metadata = TokenMetadata {
                title: Some("Hummer".to_string()),
                description: Some("Hummer".to_string()),
                media: None,
                reference: None,
                // Replace with actual field name and value
            };
            token_metadata.insert(110.into(), candy_metadata);
            token_metadata.insert(220.into(), hummer_metadata);
            


            EXTENDED_STORAGE = Some(ItemStorage {
                token_metadata: token_metadata,
                owners: HashMap::new(),
            });



        };
        ItemService {
            vmt: <VmtService>::seed("GameItem".to_owned(), "Item".to_owned(), 0),
        }
    }

    pub fn get_item() -> &'static mut ItemStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_mut()
                .expect("Extended vmt is not initialized")
        }
    }

    pub fn get_mut(&mut self) -> &'static mut ItemStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_mut()
                .expect("Extended vmt is not initialized")
        }
    }
    pub fn get(&self) -> &'static ItemStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_ref()
                .expect("Extended vmt is not initialized")
        }
    }
}

#[service(extends = VmtService, events = Event)]
impl ItemService {
    pub fn new() -> Self {
        Self {
            vmt: VmtService::new(),
        }
    }

    pub fn create_token_metadata(&mut self, id: TokenId, metadata: TokenMetadata) {
        self.ensure_is_admin();
        let storage = self.get_mut();
        storage.token_metadata.insert(id, metadata);
    }

    pub fn mint(&mut self, to: ActorId, id: TokenId, amount: U256) {
        self.ensure_is_admin();
        self.mint_internal(to, id, amount);
    }

    pub fn mint_batch(&mut self, to: ActorId, ids: Vec<TokenId>, amounts: Vec<U256>) {
        self.ensure_is_admin();
        self.mint_batch_internal(to, ids, amounts);
    }

    pub fn burn(&mut self, from: ActorId, id: TokenId, amount: U256) {
        self.ensure_is_admin();
        self.burn_internal(from, id, amount);
    }

    pub fn burn_batch(&mut self, from: ActorId, ids: Vec<TokenId>, amounts: Vec<U256>) {
        self.ensure_is_admin();
        self.burn_batch_internal(from, ids, amounts);
    }
}

impl ItemService {
    fn ensure_is_admin(&self) {
        if !Admins::is_admin(&msg::source()) {
            panic!("Not admin")
        };
    }

    // Internal mint method without admin check
    pub fn mint_internal(&mut self, to: ActorId, id: TokenId, amount: U256) {
        let storage = self.get_mut();
        let metadata = storage.token_metadata.get(&id).cloned();
        let event = services::utils::panicking(|| {
            funcs::mint(
                Storage::balances(),
                Storage::total_supply(),
                storage,
                to,
                vec![id],
                vec![amount],
                vec![metadata],
            )
        });
        self.notify_on(event).expect("Notification Error");
    }

    pub fn mint_internal_notify_off(storage:&mut ItemStorage, to: ActorId, id: TokenId, amount: U256) {
        let metadata = storage.token_metadata.get(&id).cloned();
        services::utils::panicking(|| {
            funcs::mint(
                Storage::balances(),
                Storage::total_supply(),
                storage,
                to,
                vec![id],
                vec![amount],
                vec![metadata],
            )
        });
    }



    // Internal mint batch method without admin check
    pub fn mint_batch_internal(&mut self, to: ActorId, ids: Vec<TokenId>, amounts: Vec<U256>) {
        let storage = self.get_mut();
        let metadata: Vec<Option<TokenMetadata>> = ids.iter().map(|id| storage.token_metadata.get(id).cloned()).collect();
        let event = services::utils::panicking(|| {
            funcs::mint(
                Storage::balances(),
                Storage::total_supply(),
                storage,
                to,
                ids,
                amounts,
                metadata,
            )
        });
        self.notify_on(event).expect("Notification Error");
    }

    // Internal burn method without admin check
    pub fn burn_internal(&mut self, from: ActorId, id: TokenId, amount: U256) {
        let event = services::utils::panicking(|| {
            funcs::burn(
                Storage::balances(),
                Storage::total_supply(),
                self.get_mut(),
                from,
                vec![id],
                vec![amount],
            )
        });
        self.notify_on(event).expect("Notification Error");
    }

    pub fn burn_internal_notify_off(storage:&mut ItemStorage, from: ActorId, id: TokenId, amount: U256) {
        services::utils::panicking(|| {
            funcs::burn(
                Storage::balances(),
                Storage::total_supply(),
                storage,
                from,
                vec![id],
                vec![amount],
            )
        });
    }

    // Internal burn batch method without admin check
    pub fn burn_batch_internal(&mut self, from: ActorId, ids: Vec<TokenId>, amounts: Vec<U256>) {
        let event = services::utils::panicking(|| {
            funcs::burn(
                Storage::balances(),
                Storage::total_supply(),
                self.get_mut(),
                from,
                ids,
                amounts,
            )
        });
        self.notify_on(event).expect("Notification Error");
    }
}

impl AsRef<VmtService> for ItemService {
    fn as_ref(&self) -> &VmtService {
        &self.vmt
    }
}