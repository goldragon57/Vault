use cosmwasm_std::{Addr, Coin, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetTokenAddress { address: String },
    BuyToken { amount: i32 },
    Deposit { amount: i32 },
    Withdraw { amount: i32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetTokenAddress {},
    GetBalance { address: String },
    GetAllUsers {},
    GetUserInfo { address: String },
    GetTopUsers {},
}
