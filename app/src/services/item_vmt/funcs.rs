use super::utils::{Error, Result, TokenId, TokenMetadata};
use crate::services::item_vmt::{Event, ItemStorage};
use sails_rs::{
    collections::{HashMap, HashSet},
    prelude::*,
};
pub fn mint(
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    total_supply: &mut HashMap<TokenId, U256>,
    storage: &mut ItemStorage,
    to: ActorId,
    ids: Vec<TokenId>,
    amounts: Vec<U256>,
    meta: Vec<Option<TokenMetadata>>,
) -> Result<Event> {
    if to == ActorId::zero() {
        return Err(Error::ZeroAddress);
    }

    if ids.len() != amounts.len() || ids.len() != meta.len() {
        return Err(Error::LengthMismatch);
    }

    let unique_ids: HashSet<_> = ids.clone().into_iter().collect();

    if ids.len() != unique_ids.len() {
        return Err(Error::IdIsNotUnique);
    }

    for (i, meta_item) in meta.into_iter().enumerate() {
        mint_impl(storage, balances, &to, &ids[i], amounts[i], meta_item)?;
    }

    for (id, amount) in ids.iter().zip(amounts.iter()) {
        total_supply
            .entry(*id)
            .and_modify(|quantity| {
                *quantity = quantity.saturating_add(*amount);
            })
            .or_insert(*amount);
    }

    Ok(Event::Minted { to, ids, amounts })
}

fn mint_impl(
    storage: &mut ItemStorage,
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    account: &ActorId,
    id: &TokenId,
    amount: U256,
    meta: Option<TokenMetadata>,
) -> Result<()> {
    if let Some(metadata) = meta {
        storage.token_metadata.insert(*id, metadata);
        // since we have metadata = means we have an nft, so add it to the owners
        storage.owners.insert(*id, *account);
    }

    balances
        .entry(*id)
        .or_default()
        .entry(*account)
        .and_modify(|balance| *balance = balance.saturating_add(amount))
        .or_insert(amount);

    Ok(())
}

pub fn burn(
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    total_supply: &mut HashMap<TokenId, U256>,
    storage: &mut ItemStorage,
    from: ActorId,
    ids: Vec<TokenId>,
    amounts: Vec<U256>,
) -> Result<Event> {
    if ids.len() != amounts.len() {
        return Err(Error::LengthMismatch);
    }

    ids.iter()
        .zip(amounts.clone())
        .try_for_each(|(id, amount)| {
            if storage.token_metadata.contains_key(id) && amount > U256::one() {
                // return Err(Error::AmountGreaterThanOneForNft);
            }
            check_opportunity_burn(balances, &from, id, amount)
        })?;

    ids.iter()
        .enumerate()
        .for_each(|(i, id)| burn_impl(storage, balances, &from, id, amounts[i]));

    for (id, amount) in ids.iter().zip(amounts.iter()) {
        let quantity = total_supply.get_mut(id).ok_or(Error::WrongId)?;
        *quantity = quantity.saturating_sub(*amount);
    }

    Ok(Event::Burned { from, ids, amounts })
}

fn check_opportunity_burn(
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    owner: &ActorId,
    id: &TokenId,
    amount: U256,
) -> Result<(), Error> {
    let zero = U256::zero();
    let balance = *balances.get(id).and_then(|m| m.get(owner)).unwrap_or(&zero);
    if balance < amount {
        return Err(Error::NotEnoughBalance);
    }
    Ok(())
}

fn burn_impl(
    storage: &mut ItemStorage,
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    from: &ActorId,
    id: &TokenId,
    amount: U256,
) {
    storage.owners.remove(id);
    balances
        .entry(*id)
        .or_default()
        .entry(*from)
        .and_modify(|balance| *balance = balance.saturating_sub(amount));
}
