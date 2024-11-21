pub mod execute {
    use cosmwasm_std::{DepsMut, MessageInfo, Response};

    use crate::{
        error::ContractError,
        state::{self, admin::check_admin},
        types::msg::{AddTkxIbcDenomMsg, RemoveTkxIbcDenomMsg},
    };

    pub fn add_tkx_ibc_token_denom(
        deps: DepsMut,
        info: MessageInfo,
        data: AddTkxIbcDenomMsg,
    ) -> Result<Response, ContractError> {
        // check admin
        check_admin(deps.storage, &info.sender)?;

        state::config::add_tkx_ibc_token_denom(deps.storage, &data.denom)?;

        let resp = Response::new()
            .add_attribute("method", "add_tkx_ibc_token_denom")
            .add_attribute("denom", data.denom.to_string());

        Ok(resp)
    }

    pub fn remove_tkx_ibc_token_denom(
        deps: DepsMut,
        info: MessageInfo,
        data: RemoveTkxIbcDenomMsg,
    ) -> Result<Response, ContractError> {
        // check admin
        check_admin(deps.storage, &info.sender)?;

        state::config::remove_tkx_ibc_token_denom(deps.storage, &data.denom);

        let resp = Response::new()
            .add_attribute("method", "remove_tkx_ibc_token_denom")
            .add_attribute("denom", data.denom.to_string());

        Ok(resp)
    }
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{state, types::query::ListTxkIbcDenomResp};

    pub fn list_tkx_ibc_token_denoms(deps: Deps) -> StdResult<ListTxkIbcDenomResp> {
        let denoms = state::config::list_tkx_ibc_token_denoms(deps.storage);

        Ok(ListTxkIbcDenomResp { denoms })
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
    fn admin_can_set_tkx_ibc_token_denom() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTkxIbcDenomMsg {
            denom: "uusd".to_string(),
        };

        let info = message_info(&ADMIN, &[]);
        let res = execute::add_tkx_ibc_token_denom(deps.as_mut(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![("method", "add_tkx_ibc_token_denom"), ("denom", "uusd"),]
        );
    }

    #[test]
    fn non_admin_cannot_set_tkx_ibc_token_denom() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTkxIbcDenomMsg {
            denom: "uusd".to_string(),
        };
        let info = message_info(&USER, &[]);
        let res = execute::add_tkx_ibc_token_denom(deps.as_mut(), info, msg);

        assert_eq!(
            res.err().unwrap(),
            crate::error::ContractError::Unauthorized {}
        );
    }

    #[test]
    fn admin_can_remove_tkx_ibc_token_denom() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTkxIbcDenomMsg {
            denom: "uusd".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        execute::add_tkx_ibc_token_denom(deps.as_mut(), info, msg).unwrap();

        let msg = crate::types::msg::RemoveTkxIbcDenomMsg {
            denom: "uusd".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        let res = execute::remove_tkx_ibc_token_denom(deps.as_mut(), info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![("method", "remove_tkx_ibc_token_denom"), ("denom", "uusd"),]
        );
    }

    #[test]
    fn non_admin_cannot_remove_tkx_ibc_token_denom() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTkxIbcDenomMsg {
            denom: "uusd".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        execute::add_tkx_ibc_token_denom(deps.as_mut(), info, msg).unwrap();

        let msg = crate::types::msg::RemoveTkxIbcDenomMsg {
            denom: "uusd".to_string(),
        };
        let info = message_info(&USER, &[]);
        let res = execute::remove_tkx_ibc_token_denom(deps.as_mut(), info, msg);

        assert_eq!(
            res.err().unwrap(),
            crate::error::ContractError::Unauthorized {}
        );
    }

    #[test]
    fn can_list_tkx_ibc_token_denoms() {
        let (mut deps, _env) = init_test();

        let msg = crate::types::msg::AddTkxIbcDenomMsg {
            denom: "uusd".to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        execute::add_tkx_ibc_token_denom(deps.as_mut(), info, msg).unwrap();

        let res = query::list_tkx_ibc_token_denoms(deps.as_ref()).unwrap();

        assert_eq!(res.denoms, vec!["uusd".to_string()]);
    }
}
