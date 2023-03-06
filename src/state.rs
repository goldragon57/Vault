use cosmwasm_std::{Addr, Env, Order, StdResult, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, ReadonlySingleton, Singleton,
};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static CONFIG_KEY: &[u8] = b"config";
pub const CONFIG_ADDRESS: &[u8] = b"token_address";
pub const CONFIG_USERS: &[u8] = b"User";
pub const CONFIG_USER_INFO: &[u8] = b"UserInfo";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_money: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
    pub address: String,
    pub amount: Uint128,
}

pub const USERS: Map<&str, UserInfo> = Map::new("Users");

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}
pub fn store_token_address(storage: &mut dyn Storage, token_address: &Addr) -> StdResult<()> {
    Singleton::new(storage, CONFIG_ADDRESS).save(token_address)
}

pub fn read_token_address(storage: &dyn Storage) -> StdResult<Addr> {
    ReadonlySingleton::new(storage, CONFIG_ADDRESS).load()
}

pub fn store_users(storage: &mut dyn Storage, user: &Addr, user_info: UserInfo) -> StdResult<()> {
    bucket(storage, CONFIG_USERS).save(user.as_bytes(), &user_info)
}

pub fn read_user_info(storage: &dyn Storage, user: &Addr) -> Option<UserInfo> {
    match bucket_read(storage, CONFIG_USERS).load(user.as_bytes()) {
        Ok(v) => Some(v),
        _ => None,
    }
}

pub fn read_users(storage: &dyn Storage) -> StdResult<Vec<String>> {
    USERS.keys(storage, None, None, Order::Ascending).collect()
}
