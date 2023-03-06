use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    config, config_read, read_token_address, read_user_info, read_users, store_token_address,
    store_users, State, UserInfo,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        total_money: Uint128::new(0),
    };
    config(deps.storage).save(&state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let state = config_read(deps.storage).load()?;
    match msg {
        ExecuteMsg::SetTokenAddress { address } => execute_set_address(deps, address),
        ExecuteMsg::BuyToken { amount } => execute_buy_token(deps, info, amount),
        ExecuteMsg::Deposit { amount } => execute_deposit(deps, env, info, amount),
        ExecuteMsg::Withdraw { amount } => execute_withdraw(deps, env, info, amount),
    }
}

pub fn execute_set_address(deps: DepsMut, address: String) -> Result<Response, ContractError> {
    let token_address = deps.api.addr_validate(&address)?;
    store_token_address(deps.storage, &token_address)?;
    Ok(Response::default())
}

pub fn execute_buy_token(
    deps: DepsMut,
    // env:Env,
    info: MessageInfo,
    amount: i32,
) -> Result<Response, ContractError> {
    let token_address = read_token_address(deps.storage)?;
    let res = Response::new()
        .add_attribute("action", "buy")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: String::from(info.sender.as_str()),
                amount: Uint128::new(amount as u128),
            })?,
        }));
    Ok(res)
}

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: i32,
) -> Result<Response, ContractError> {
    let mut current_state = config_read(deps.storage).load()?;
    let token_address = read_token_address(deps.storage)?;

    let user_address = info.sender.clone();
    let user_info = read_user_info(deps.storage, &user_address);
    let amount = Uint128::new(amount as u128);

    let res = Response::new()
        .add_attribute("action", "deposit")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: String::from(info.sender.as_str()),
                recipient: String::from(env.contract.address.as_str()),
                amount: amount,
            })?,
        }));

    if user_info == None {
        let new_user_info = UserInfo {
            address: user_address.to_string(),
            amount: amount,
        };
        store_users(deps.storage, &user_address, new_user_info)?;
    } else {
        let mut current_user_info = user_info.unwrap();
        current_user_info.amount += amount;
        store_users(deps.storage, &user_address, current_user_info)?;
    }
    current_state.total_money += amount;
    config(deps.storage).save(&current_state)?;
    Ok(res)
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: i32,
) -> Result<Response, ContractError> {
    //save_current state
    let mut current_state = config_read(deps.storage).load()?;
    let token_address = read_token_address(deps.storage)?;
    // //save  Users
    let user_address = info.sender;
    let user_info = read_user_info(deps.storage, &user_address);
    if user_info == None {
        return Err(ContractError::Unauthorized {});
    } else {
        let mut current_user_info = user_info.unwrap();
        if current_user_info.amount < Uint128::new(amount as u128) {
            return Err(ContractError::Notenough {});
        }
        current_user_info.amount -= Uint128::new(amount as u128);
        store_users(deps.storage, &user_address, current_user_info)?;
    }

    current_state.total_money -= Uint128::new(amount as u128);
    config(deps.storage).save(&current_state)?;

    Ok(Response::new()
        .add_attribute("action", "buy")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: String::from(user_address.as_str()),
                amount: Uint128::new(amount as u128),
            })?,
        })))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { address } => to_binary(&query_get_balance(deps, address)?),
        QueryMsg::GetTokenAddress {} => to_binary(&query_get_address(deps)?),
        QueryMsg::GetAllUsers {} => to_binary(&query_get_users(deps)?),
        QueryMsg::GetUserInfo { address } => to_binary(&query_user_info(deps, address)?),
        QueryMsg::GetTopUsers {} => to_binary(&query_top_users(deps)?),
    }
}

pub fn query_get_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    deps.api.addr_validate(&address)?;
    let token_address = read_token_address(deps.storage)?;
    let balance = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_address.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance { address })?,
    }))?;
    Ok(balance)
}

pub fn query_get_address(deps: Deps) -> StdResult<String> {
    let token_address = read_token_address(deps.storage)?;
    let result = token_address.to_string();
    Ok(result)
}

pub fn query_get_users(deps: Deps) -> StdResult<Vec<String>> {
    let users = read_users(deps.storage)?;
    Ok(users)
}

pub fn query_user_info(deps: Deps, address: String) -> StdResult<UserInfo> {
    let user = read_user_info(deps.storage, &deps.api.addr_validate(&address)?).unwrap();
    Ok(user)
}

pub fn query_top_users(deps: Deps) -> StdResult<Vec<UserInfo>> {
    let users = read_users(deps.storage)?;
    let mut user_info_group = vec![];
    for user in users.iter() {
        let user_info =
            read_user_info(deps.storage, &deps.api.addr_validate(&String::from(user))?).unwrap();
        user_info_group.push(user_info);
    }
    user_info_group.sort_by(|a, b| a.amount.cmp(&b.amount));
    let mut result = vec![];
    result.push(user_info_group.pop().unwrap());
    result.push(user_info_group.pop().unwrap());
    // result.push(user_info_group[1]);
    if user_info_group.len() < 2 {
        Ok(user_info_group)
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::CosmosMsg;

    #[test]
    fn buy_token() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &[]);
        let address = String::from("token_address");
        let message = ExecuteMsg::SetTokenAddress { address: address };
        execute(deps.as_mut(), mock_env(), info, message).unwrap();

        let res = query_get_address(deps.as_ref()).unwrap();
        assert_eq!(res, "token_address");

        let info = mock_info("sender", &[]);
        let amount = 100;
        let message = ExecuteMsg::BuyToken { amount };
        let res = execute(deps.as_mut(), mock_env(), info, message).unwrap();
        let message = res.messages[0].clone().msg;
        assert_eq!(
            message,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: String::from("token_address"),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: String::from("sender"),
                    amount: Uint128::new(100)
                })
                .unwrap(),
                funds: vec![]
            })
        );
    }

    #[test]
    fn deposit_and_withdraw() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &[]);
        let address = String::from("token_address");
        let message = ExecuteMsg::SetTokenAddress { address: address };

        execute(deps.as_mut(), mock_env(), info, message).unwrap();

        let info = mock_info("sender0", &[]);
        let msg = ExecuteMsg::Deposit { amount: 10 };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        let info = mock_info("sender1", &[]);
        let msg = ExecuteMsg::Deposit { amount: 20 };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let user = query_user_info(deps.as_ref(), "sender0".to_string()).unwrap();
        assert_eq!(
            user,
            UserInfo {
                address: "sender0".to_string(),
                amount: Uint128::new(10)
            }
        );

        let info = mock_info("sender2", &[]);
        let msg = ExecuteMsg::Deposit { amount: 5 };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let users = query_get_users(deps.as_ref()).unwrap();

        let info = mock_info("sender3", &[]);
        let msg = ExecuteMsg::Deposit { amount: 20 };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let users = query_get_users(deps.as_ref()).unwrap();

        assert_eq!(users, ["sender0", "sender1", "sender2", "sender3"]);

        let info = mock_info("creator", &[]);
        let address = String::from("token_address");
        let message = ExecuteMsg::SetTokenAddress { address: address };
        execute(deps.as_mut(), mock_env(), info, message).unwrap();

        let info = mock_info("sender1", &[]);
        let msg = ExecuteMsg::Withdraw { amount: 5 };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let user = query_user_info(deps.as_ref(), "sender1".to_string()).unwrap();
        assert_eq!(
            user,
            UserInfo {
                address: "sender1".to_string(),
                amount: Uint128::new(15)
            }
        );
        assert_eq!(res.messages.len(), 1);
        let message = res.messages[0].clone().msg;
        assert_eq!(
            message,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: String::from("token_address"),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: String::from("sender1"),
                    amount: Uint128::new(5)
                })
                .unwrap(),
                funds: vec![]
            })
        );
        let top_users = query_top_users(deps.as_ref()).unwrap();
        assert_eq!(
            top_users,
            [
                UserInfo {
                    address: "sender3".to_string(),
                    amount: Uint128::new(20)
                },
                UserInfo {
                    address: "sender1".to_string(),
                    amount: Uint128::new(15)
                }
            ]
        );
    }
}
