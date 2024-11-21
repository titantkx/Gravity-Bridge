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
