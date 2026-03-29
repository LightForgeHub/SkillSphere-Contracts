#![allow(deprecated)]
use soroban_sdk::{symbol_short, Address, Env};

pub fn contract_paused(env: &Env, paused: bool) {
    let topics = (symbol_short!("paused"),);
    env.events().publish(topics, paused);
}

pub fn admin_transferred(env: &Env, old_admin: &Address, new_admin: &Address) {
    let topics = (symbol_short!("adm_xfer"),);
    env.events()
        .publish(topics, (old_admin.clone(), new_admin.clone()));
}
