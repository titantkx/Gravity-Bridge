use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::{state::admin::set_admin, types::msg::InstantiateMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admin = match msg.admin {
        None => info.sender,
        Some(addr) => deps.api.addr_validate(&addr)?,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    set_admin(deps.storage, &admin)?;

    let resp = Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin);

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use crate::state::admin::get_admin;

    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{attr, Addr, StdError};

    // fn instantiate_contract(deps: DepsMut, admin: &Addr) -> Result<String, ContractError> {}

    #[test]
    fn instantiate_without_admin() {
        let mut deps = mock_dependencies();
        deps.api = deps.api.with_prefix("titan");
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg { admin: None };
        let res = instantiate(deps.as_mut(), env, info.clone(), msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![
                attr("method", "instantiate"),
                attr("admin", Addr::unchecked("creator"))
            ]
        );
        assert_eq!(
            get_admin(&deps.storage).unwrap(),
            Addr::unchecked("creator")
        );
    }

    #[test]
    fn instantiate_with_admin() {
        let mut deps = mock_dependencies();
        deps.api = deps.api.with_prefix("titan");
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let admin = deps.api.addr_make("admin");
        let msg = InstantiateMsg {
            admin: Some(admin.to_string()),
        };
        let res = instantiate(deps.as_mut(), env, info.clone(), msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![attr("method", "instantiate"), attr("admin", admin.clone())]
        );

        assert_eq!(get_admin(&deps.storage).unwrap(), admin.clone());
    }

    #[test]
    fn instantiate_with_admin_wrong_prefix() {
        let mut deps = mock_dependencies();
        deps.api = deps.api.with_prefix("titan");
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let admin = deps.api.with_prefix("test").addr_make("admin");
        let msg = InstantiateMsg {
            admin: Some(admin.to_string()),
        };
        let res = instantiate(deps.as_mut(), env, info.clone(), msg);

        assert!(res.is_err());
        // validate err is correct
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("Wrong bech32 prefix",)
        );
    }
}
