#[cfg(feature = "library")]
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
#[cfg(feature = "library")]
use error::ContractError;
#[cfg(feature = "library")]
use types::{
    msg::{ExecuteMsg, InstantiateMsg},
    query::QueryMsg,
    sudo::SudoMsg,
};

mod constant;
#[cfg(feature = "library")]
mod contract;
#[cfg(feature = "types")]
mod error;
#[cfg(feature = "library")]
mod exec;
#[cfg(feature = "library")]
mod msgs;
#[cfg(feature = "library")]
mod query;
#[cfg(feature = "library")]
mod state;
#[cfg(feature = "types")]
mod types;

#[cfg(feature = "library")]
#[cfg_attr(feature = "contract", entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, env, info, msg)
}

#[cfg(feature = "library")]
#[cfg_attr(feature = "contract", entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    query::query(deps, env, msg)
}

#[cfg(feature = "library")]
#[cfg_attr(feature = "contract", entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    exec::execute(deps, env, info, msg)
}

#[cfg(feature = "library")]
#[cfg_attr(feature = "contract", entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> StdResult<Response> {
    match msg {
        SudoMsg::IBCLifecycleComplete(data) => msgs::withdraw::sudo::sudo(deps, env, data),
    }
}

#[cfg(feature = "library")]
#[cfg_attr(feature = "contract", entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        constant::SUB_MSG_ID_WITHDRAW_IBC_1 => msgs::withdraw::reply::reply(deps, env, msg),
        _ => Err(ContractError::Logic {
            err: format!("Invalid sub msg id {:?}", msg.id),
        }),
    }
}
