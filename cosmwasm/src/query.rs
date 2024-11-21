use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

use crate::types::query::QueryMsg;

pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Greet {} => to_json_binary(&query::greet()?),
    }
}

mod query {
    use super::*;
    use crate::types::query::*;

    pub fn greet() -> StdResult<GreetResp> {
        let resp = GreetResp {
            message: "Hello World".to_owned(),
        };

        Ok(resp)
    }
}
