use core::fmt::Debug;
use sails_rs::{
    Vec,
    gstd::{msg, service},
    prelude::*,
};

static mut ADMINS: Option<Admins> = None;

#[derive(Debug, Default)]
pub struct Admins {
    list: Vec<ActorId>,
}

impl Admins {
    pub fn get_mut() -> &'static mut Self {
        unsafe { ADMINS.as_mut().expect("Admins is not initialized") }
    }
    // pub fn get() -> &'static Self {
    //     unsafe { ADMINS.as_ref().expect("Admins is not initialized") }
    // }
    pub fn is_admin(account: &ActorId) -> bool {
        let admins = unsafe { ADMINS.as_ref().expect("Admins is not initialized") };
        admins.list.contains(account)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    AdminAdded {
        admin: ActorId,
    },
    AdminRemoved {
        admin: ActorId,
    },
}

#[derive(Clone)]
pub struct Service();

impl Service {
    pub fn seed() -> Self {
        unsafe {
            let mut admins = Admins {
                list: Vec::new(),
            };
            let deployer = msg::source();
            admins.list.push(deployer); 
            ADMINS = Some(admins);
        }
        Self()
    }
}

#[service(events = Event)]
impl Service {
    pub fn new() -> Self {
        Self()
    }

    pub fn add_admin(&mut self, admin: ActorId) -> bool {
        let caller = msg::source();
        if !Admins::is_admin(&caller) {
            return false;
        }
        let admins = Admins::get_mut();
        if !admins.list.contains(&admin) {
            admins.list.push(admin);
            self.notify_on(Event::AdminAdded { admin }).expect("Notification Error");
            true
        } else {
            false
        }
    }

    pub fn remove_admin(&mut self, admin: ActorId) -> bool {
        let caller = msg::source();
        if !Admins::is_admin(&caller) {
            return false;
        }
        let admins = Admins::get_mut();
        if let Some(pos) = admins.list.iter().position(|x| *x == admin) {
            admins.list.remove(pos);
            self.notify_on(Event::AdminRemoved { admin }).expect("Notification Error");
            true
        } else {
            false
        }
    }

    pub fn is_admin(&self, account: ActorId) -> bool {
        Admins::is_admin(&account)
    }
}