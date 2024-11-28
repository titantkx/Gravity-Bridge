use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

use crate::msgs::*;
use crate::types::query::QueryMsg;

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        GetAdmin {} => to_json_binary(&admin::query::get_admin(deps)?),
        ListTxkIbcDenom {} => to_json_binary(&config::query::list_tkx_ibc_token_denoms(deps)?),
        ListTKXChainWithDenom {} => {
            to_json_binary(&config::query::list_tkx_chain_with_denoms(deps)?)
        }
        GetWithdrawInfo {
            channel_id,
            sequence,
        } => to_json_binary(&withdraw::query::get_withdraw_info(
            deps, channel_id, sequence,
        )?),
    }
}
