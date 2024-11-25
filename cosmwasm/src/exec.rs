use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::msgs::*;
use crate::{error::ContractError, types::msg::ExecuteMsg};

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        SetAdmin(data) => admin::execute::set_admin(deps, info, data),
        AddTkxIbcDenom(data) => config::execute::add_tkx_ibc_token_denom(deps, info, data),
        RemoveTkxIbcDenom(data) => config::execute::remove_tkx_ibc_token_denom(deps, info, data),
        Deposit(data) => deposit::execute::deposit(deps, env, info, data),
    }
}
