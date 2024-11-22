pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{state, types::query::GetAdminResp};

    pub fn get_admin(deps: Deps) -> StdResult<GetAdminResp> {
        let addr = state::admin::get_admin(deps.storage)?;

        Ok(GetAdminResp {
            address: addr.into_string(),
        })
    }
}

pub mod execute {
    use cosmwasm_std::{DepsMut, MessageInfo, Response};

    use crate::{error::ContractError, state, types::msg::SetAdminMsg};

    pub fn set_admin(
        deps: DepsMut,
        info: MessageInfo,
        data: SetAdminMsg,
    ) -> Result<Response, ContractError> {
        state::admin::check_admin(deps.storage, &info.sender)?;

        let admin_addr = deps.api.addr_validate(&data.admin)?;
        state::admin::set_admin(deps.storage, &admin_addr)?;

        let resp = Response::new()
            .add_attribute("method", "set_admin")
            .add_attribute("admin", data.admin.to_string());

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {

    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier},
        Addr, Env, MemoryStorage, OwnedDeps,
    };
    use lazy_static::lazy_static;

    use crate::{
        contract::instantiate,
        types::msg::{InstantiateMsg, SetAdminMsg},
    };

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
    fn admin_can_set_admin() {
        let (mut deps, _) = init_test();

        let msg = SetAdminMsg {
            admin: USER.to_string(),
        };
        let info = message_info(&ADMIN, &[]);
        let res = execute::set_admin(deps.as_mut(), info.clone(), msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                ("method", "set_admin"),
                ("admin", USER.to_string().as_str()),
            ]
        );

        let res = query::get_admin(deps.as_ref()).unwrap();
        assert_eq!(res.address, USER.to_string());
    }

    #[test]
    fn non_admin_cannot_set_admin() {
        let (mut deps, _) = init_test();

        let msg = SetAdminMsg {
            admin: USER.to_string(),
        };
        let info = message_info(&USER, &[]);
        let res = execute::set_admin(deps.as_mut(), info, msg);
        assert_eq!(
            res.err().unwrap(),
            crate::error::ContractError::Unauthorized {}
        );
    }
}
