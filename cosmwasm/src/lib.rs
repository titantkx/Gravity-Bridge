use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use error::ContractError;
use types::{
    msg::{ExecuteMsg, InstantiateMsg},
    query::QueryMsg,
    sudo::SudoMsg,
};

mod constant;
mod contract;
mod error;
mod exec;
mod msgs;
mod query;
mod state;
mod types;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    query::query(deps, env, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    exec::execute(deps, env, info, msg)
}

#[entry_point]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> StdResult<Response> {
    match msg {
        SudoMsg::IBCLifecycleComplete(data) => msgs::withdraw::sudo::sudo(deps, env, data),
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        constant::SUB_MSG_ID_WITHDRAW_IBC_1 => msgs::withdraw::reply::reply(deps, env, msg),
        _ => Err(ContractError::Logic {}),
    }
}
