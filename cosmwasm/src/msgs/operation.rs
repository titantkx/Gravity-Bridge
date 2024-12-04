pub mod execute {
    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

    use crate::{constant::TKX_NATIVE_DENOM, error::ContractError};

    pub fn supply_tkx_token(
        _deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        // info.funds should only contain tkx native coin
        if info.funds.len() != 1 {
            return Err(ContractError::InvalidToken {});
        }
        let tkx_token = info.funds[0].clone();
        if tkx_token.denom != TKX_NATIVE_DENOM {
            return Err(ContractError::InvalidToken {});
        }

        // tkx_token must be greater than 0
        if tkx_token.amount.is_zero() {
            return Err(ContractError::InvalidAmount {});
        }

        let resp = Response::new()
            .add_attribute("method", "supply_tkx_token")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", tkx_token.amount.to_string())
            .add_attribute("denom", tkx_token.denom.to_string());

        Ok(resp)
    }
}
