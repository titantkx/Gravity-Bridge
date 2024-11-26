pub mod execute {

    use cosmwasm_std::{
        BalanceResponse, BankQuery, Coin, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcTimeout,
        MessageInfo, Response, StdResult, Uint128,
    };

    use crate::{
        error::ContractError,
        state,
        types::msg::{IbcAutoSendEth, IbcAutoSendEthMemo, WithdrawMsg},
        TKX_NATIVE_DENOM,
    };

    fn get_contract_balance(deps: Deps, env: &Env, denom: &str) -> StdResult<Uint128> {
        let address = env.contract.address.clone();

        let balance_query = BankQuery::Balance {
            denom: denom.to_string(),
            address: address.to_string(),
        };
        let balance_response: BalanceResponse = deps.querier.query(&balance_query.into())?;
        let balance_u128 = balance_response.amount.amount.u128();
        Ok(Uint128::from(balance_u128))
    }

    fn check_contract_balance(
        deps: Deps,
        env: &Env,
        denom: &str,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        let contract_balance = get_contract_balance(deps, env, denom)?;
        if contract_balance < amount {
            return Err(ContractError::InsufficientContractBalance {});
        }
        Ok(())
    }

    pub fn withdraw(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        data: WithdrawMsg,
    ) -> Result<Response, ContractError> {
        // info.funds should only contain tkx native coin
        if info.funds.len() != 1 {
            return Err(ContractError::InvalidToken {});
        }
        let tkx_token = info.funds[0].clone();
        if tkx_token.denom != TKX_NATIVE_DENOM {
            return Err(ContractError::InvalidToken {});
        }

        // check valid amount received should eq `data.amount + data.bridge_fee`
        let expected_amount = data.amount + data.bridge_fee;
        if tkx_token.amount != expected_amount {
            return Err(ContractError::InvalidAmount {});
        }

        // check `data.chain_prefix` is valid
        let tkx_ibc_info =
            state::config::get_tkx_ibc_token_by_chain_prefix(deps.storage, &data.chain_prefix)?;

        let tkx_ibc_token = Coin {
            denom: tkx_ibc_info.denom,
            amount: tkx_token.amount,
        };

        // check contract balance
        check_contract_balance(deps.as_ref(), &env, &tkx_token.denom, tkx_token.amount)?;

        // build ibc transfer message
        // build memo
        let memo_data = IbcAutoSendEthMemo {
            send_to_eth: IbcAutoSendEth {
                evm_chain_prefix: data.chain_prefix.clone(),
                eth_dest: data.recipient.clone(),
                amount: data.amount,
                bridge_fee: data.bridge_fee,
            },
            ibc_callback: env.contract.address.to_string(),
        };

        let memo_str = serde_json::to_string(&memo_data).unwrap();

        // timeout 30 days from now
        let current_time = env.block.time;
        let ibc_timeout = IbcTimeout::with_timestamp(current_time.plus_days(30));
        let ibc_transfer_msg: CosmosMsg = CosmosMsg::Ibc(IbcMsg::Transfer {
            channel_id: tkx_ibc_info.channel_id,
            to_address: data.forwarder.clone(),
            amount: tkx_ibc_token,
            timeout: ibc_timeout,
            memo: Some(memo_str),
        });

        let resp = Response::new()
            .add_message(ibc_transfer_msg)
            .add_attribute("method", "withdraw")
            .add_attribute("chain_prefix", data.chain_prefix.clone())
            .add_attribute("recipient", data.recipient.clone())
            .add_attribute("forwarder", data.forwarder.clone())
            .add_attribute("total_amount", tkx_token.amount.to_string())
            .add_attribute("amount", data.amount.to_string())
            .add_attribute("bridge_fee", data.bridge_fee.to_string());

        Ok(resp)
    }
}
