pub mod execute {
    use cosmwasm_std::{DepsMut, MessageInfo, Response};
    use regex_lite::Regex;

    use crate::{
        error::ContractError,
        state::{self, admin::check_admin},
        types::msg::{AddTKXIbcDenomMsg, RemoveTKXChainMsg},
    };

    pub fn add_tkx_ibc_token_info(
        deps: DepsMut,
        info: MessageInfo,
        data: AddTKXIbcDenomMsg,
    ) -> Result<Response, ContractError> {
        // check admin
        check_admin(deps.storage, &info.sender)?;

        if Regex::new(data.address_regex.as_str()).is_err() {
            return Err(ContractError::InvalidRegex {
                re: data.address_regex.to_string(),
            });
        }

        state::config::add_tkx_ibc_token_denom(
            deps.storage,
            &data.chain_prefix,
            &data.address_regex,
            &data.denom,
            data.decimals,
            &data.channel_id,
            &data.forwarder_prefix,
        )?;

        let resp = Response::new()
            .add_attribute("method", "add_tkx_ibc_token_info")
            .add_attribute("chain_prefix", data.chain_prefix.to_string())
            .add_attribute("address_regex", data.address_regex.to_string())
            .add_attribute("denom", data.denom.to_string())
            .add_attribute("decimals", data.decimals.to_string())
            .add_attribute("channel_id", data.channel_id.to_string())
            .add_attribute("forwarder_prefix", data.forwarder_prefix.to_string());

        Ok(resp)
    }

    pub fn remove_tkx_ibc_token_info(
        deps: DepsMut,
        info: MessageInfo,
        data: RemoveTKXChainMsg,
    ) -> Result<Response, ContractError> {
        // check admin
        check_admin(deps.storage, &info.sender)?;

        state::config::remove_tkx_chain(deps.storage, &data.chain_prefix);

        let resp = Response::new()
            .add_attribute("method", "remove_tkx_ibc_token_info")
            .add_attribute("chain_prefix", data.chain_prefix.to_string());

        Ok(resp)
    }
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{
        state,
        types::query::{ListTKXChainInfoResp, ListTxkIbcDenomResp, TKXChainInfo},
    };

    pub fn list_tkx_ibc_token_denoms(deps: Deps) -> StdResult<ListTxkIbcDenomResp> {
        let denoms = state::config::list_tkx_ibc_token_denoms(deps.storage);

        Ok(ListTxkIbcDenomResp { denoms })
    }

    pub fn list_tkx_chain_with_denoms(deps: Deps) -> StdResult<ListTKXChainInfoResp> {
        let data = state::config::list_tkx_chain_with_ibc_token_denoms(deps.storage)
            .into_iter()
            .map(|(chain_prefix, info)| TKXChainInfo {
                chain_prefix,
                address_regex: info.address_regex,
                denom: info.denom,
                decimals: info.decimals,
                channel_id: info.channel_id,
                forwarder_prefix: info.forwarder_prefix,
            })
            .collect();

        Ok(ListTKXChainInfoResp { data })
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier},
        Addr, Env, MemoryStorage, OwnedDeps,
    };
    use lazy_static::lazy_static;

    use crate::{contract::instantiate, types::msg::InstantiateMsg};

    use super::*;

    lazy_static! {
        static ref ADMIN: Addr = mock_dependencies()
            .api
            .with_prefix("titan")
            .addr_make("admin");
        static ref USER: Addr = mock_dependencies()
            .api
            .with_prefix("titan")
            .addr_make("user");
    }

    fn init_test() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier>, Env) {
        let mut deps = mock_dependencies();
        deps.api = deps.api.with_prefix("titan");
        let env = mock_env();
        // init contract with admin
        let msg = InstantiateMsg {
            admin: Some(ADMIN.to_string()),
        };
        let info = message_info(&ADMIN, &[]);
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        (deps, env)
    }

    #[test]
    fn admin_can_set_tkx_ibc_token_info() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTKXIbcDenomMsg {
            chain_prefix: "eth".to_string(),
            address_regex: "0x[a-fA-F0-9]{40}".to_string(),
            denom: "uusd".to_string(),
            decimals: 6,
            channel_id: "channel-0".to_string(),
            forwarder_prefix: "gravity".to_string(),
        };

        let info = message_info(&ADMIN, &[]);
        let res = execute::add_tkx_ibc_token_info(deps.as_mut(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![
                ("method", "add_tkx_ibc_token_info"),
                ("chain_prefix", "eth"),
                ("address_regex", "0x[a-fA-F0-9]{40}"),
                ("denom", "uusd"),
                ("decimals", "6"),
                ("channel_id", "channel-0"),
                ("forwarder_prefix", "gravity"),
            ]
        );
    }

    #[test]
    fn non_admin_cannot_set_tkx_ibc_token_denom() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTKXIbcDenomMsg {
            chain_prefix: "eth".to_string(),
            address_regex: "0x[a-fA-F0-9]{40}".to_string(),
            denom: "uusd".to_string(),
            decimals: 6,
            channel_id: "channel-0".to_string(),
            forwarder_prefix: "gravity".to_string(),
        };
        let info = message_info(&USER, &[]);
        let res = execute::add_tkx_ibc_token_info(deps.as_mut(), info, msg);

        assert_eq!(
            res.err().unwrap(),
            crate::error::ContractError::Unauthorized {}
        );
    }

    #[test]
    fn admin_can_remove_tkx_ibc_token_info() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTKXIbcDenomMsg {
            chain_prefix: "eth".to_string(),
            address_regex: "0x[a-fA-F0-9]{40}".to_string(),
            denom: "uusd".to_string(),
            decimals: 6,
            channel_id: "channel-0".to_string(),
            forwarder_prefix: "gravity".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        execute::add_tkx_ibc_token_info(deps.as_mut(), info, msg).unwrap();

        let msg = crate::types::msg::RemoveTKXChainMsg {
            chain_prefix: "eth".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        let res = execute::remove_tkx_ibc_token_info(deps.as_mut(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![
                ("method", "remove_tkx_ibc_token_info"),
                ("chain_prefix", "eth"),
            ]
        );
    }

    #[test]
    fn non_admin_cannot_remove_tkx_ibc_token_denom() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTKXIbcDenomMsg {
            chain_prefix: "eth".to_string(),
            address_regex: "0x[a-fA-F0-9]{40}".to_string(),
            denom: "uusd".to_string(),
            decimals: 6,
            channel_id: "channel-0".to_string(),
            forwarder_prefix: "gravity".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        execute::add_tkx_ibc_token_info(deps.as_mut(), info, msg).unwrap();

        let msg = crate::types::msg::RemoveTKXChainMsg {
            chain_prefix: "eth".to_string(),
        };
        let info = message_info(&USER, &[]);
        let res = execute::remove_tkx_ibc_token_info(deps.as_mut(), info, msg);

        assert_eq!(
            res.err().unwrap(),
            crate::error::ContractError::Unauthorized {}
        );
    }

    #[test]
    fn can_list_tkx_ibc_token_denoms() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTKXIbcDenomMsg {
            chain_prefix: "eth".to_string(),
            address_regex: "0x[a-fA-F0-9]{40}".to_string(),
            denom: "uusd".to_string(),
            decimals: 6,
            channel_id: "channel-0".to_string(),
            forwarder_prefix: "gravity".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        execute::add_tkx_ibc_token_info(deps.as_mut(), info, msg).unwrap();

        let res = query::list_tkx_ibc_token_denoms(deps.as_ref()).unwrap();

        assert_eq!(res.denoms, vec!["uusd".to_string()]);
    }

    #[test]
    fn can_list_tkx_chain_with_denoms() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTKXIbcDenomMsg {
            chain_prefix: "eth".to_string(),
            address_regex: "0x[a-fA-F0-9]{40}".to_string(),
            denom: "uusd".to_string(),
            decimals: 6,
            channel_id: "channel-0".to_string(),
            forwarder_prefix: "gravity".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        execute::add_tkx_ibc_token_info(deps.as_mut(), info, msg).unwrap();

        let res = query::list_tkx_chain_with_denoms(deps.as_ref()).unwrap();

        assert_eq!(
            res.data,
            vec![crate::types::query::TKXChainInfo {
                chain_prefix: "eth".to_string(),
                address_regex: "0x[a-fA-F0-9]{40}".to_string(),
                denom: "uusd".to_string(),
                decimals: 6,
                channel_id: "channel-0".to_string(),
                forwarder_prefix: "gravity".to_string(),
            }]
        );
    }
}
