pub mod execute {
    use cosmwasm_std::{
        BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
        Response, StdResult, Uint128,
    };

    use crate::{constant::TKX_NATIVE_DENOM, error::ContractError, state, types::msg::DepositMsg};

    fn get_contract_tkx_balance(deps: Deps, env: Env) -> StdResult<Uint128> {
        let denom = TKX_NATIVE_DENOM;
        let address = env.contract.address;

        let balance_query = BankQuery::Balance {
            denom: denom.to_string(),
            address: address.to_string(),
        };
        let balance_response: BalanceResponse = deps.querier.query(&balance_query.into())?;
        Ok(balance_response.amount.amount)
    }

    fn check_contract_tkx_balance(
        deps: Deps,
        env: Env,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        let contract_balance = get_contract_tkx_balance(deps, env)?;
        if contract_balance < amount {
            return Err(ContractError::InsufficientContractBalance {});
        }
        Ok(())
    }

    pub fn deposit(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        data: DepositMsg,
    ) -> Result<Response, ContractError> {
        let mut tkx_amount = Uint128::zero();
        let all_denoms = state::config::list_tkx_ibc_token_denoms(deps.storage);

        // check all token that we received. If have any token in our list
        // we will accept all of them (including the one that is not in our list those will be transfer all to `recipient` )
        // if all of them are not in our list, we will reject this transaction.
        let mut should_reject = true;
        let mut not_tkx_tokens: Vec<Coin> = vec![];
        for token in info.funds {
            if all_denoms.contains(&token.denom) {
                should_reject = false;
                tkx_amount += token.amount;
            } else {
                not_tkx_tokens.push(token);
            }
        }

        if should_reject {
            return Err(ContractError::InvalidToken {});
        }

        let recipient = deps.api.addr_validate(&data.recipient)?;
        let mut messages: Vec<CosmosMsg> = vec![];

        // transfer tkx tokens to recipient
        if tkx_amount > Uint128::zero() {
            // check contract balance
            check_contract_tkx_balance(deps.as_ref(), env, tkx_amount)?;

            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![Coin {
                    denom: TKX_NATIVE_DENOM.to_string(),
                    amount: tkx_amount,
                }],
            }));
        }

        // transfer all non-tkx tokens to recipient
        if !not_tkx_tokens.is_empty() {
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: not_tkx_tokens,
            }));
        }

        let resp = Response::new()
            .add_messages(messages)
            .add_attribute("method", "deposit")
            .add_attribute("amount", tkx_amount)
            .add_attribute("recipient", data.recipient);

        Ok(resp)
    }
}
