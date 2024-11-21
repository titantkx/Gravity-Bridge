use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::{
    error::ContractError,
    state::{self, admin::check_admin},
    types::msg::{AddTkxIbcDenomMsg, RemoveTkxIbcDenomMsg},
};

pub mod execute {
    use super::*;

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
