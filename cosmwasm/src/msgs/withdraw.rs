pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{state, types::state::WithdrawInfo};

    pub fn get_withdraw_info(
        deps: Deps,
        channel_id: String,
        sequence: u64,
    ) -> StdResult<WithdrawInfo> {
        state::withdraw::get_withdraw_info(deps.storage, &channel_id, sequence)
    }
}

pub mod execute {

    use cosmwasm_std::{
        BalanceResponse, BankQuery, Coin, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcTimeout,
        MessageInfo, Response, StdResult, SubMsg, Uint128,
    };

    use crate::{
        constant::{SUB_MSG_ID_WITHDRAW_IBC_1, TKX_NATIVE_DENOM},
        error::ContractError,
        state,
        types::{
            msg::{IbcAutoSendEth, IbcAutoSendEthMemo, WithdrawMsg},
            state::SendingWithdrawInfo,
        },
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

        // data.amount should be greater than 0
        if data.amount.is_zero() {
            return Err(ContractError::InvalidAmount {});
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
            channel_id: tkx_ibc_info.channel_id.clone(),
            to_address: data.forwarder.clone(),
            amount: tkx_ibc_token,
            timeout: ibc_timeout,
            memo: Some(memo_str),
        });
        let ibc_transfer_sub_msg =
            SubMsg::reply_on_success(ibc_transfer_msg, SUB_MSG_ID_WITHDRAW_IBC_1);

        // save sending withdraw info
        state::withdraw::set_sending_withdraw(
            deps.storage,
            &SendingWithdrawInfo {
                ibc_channel_id: tkx_ibc_info.channel_id.clone(),
                chain_prefix: data.chain_prefix.clone(),
                sender: info.sender.clone(),
                recipient: data.recipient.clone(),
                forwarder: data.forwarder.clone(),
                total_amount: tkx_token.amount,
                amount: data.amount,
                bridge_fee: data.bridge_fee,
            },
        )?;

        let resp = Response::new()
            .add_submessage(ibc_transfer_sub_msg)
            .add_attribute("method", "withdraw")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("chain_prefix", data.chain_prefix.clone())
            .add_attribute("recipient", data.recipient.clone())
            .add_attribute("forwarder", data.forwarder.clone())
            .add_attribute("total_amount", tkx_token.amount.to_string())
            .add_attribute("amount", data.amount.to_string())
            .add_attribute("bridge_fee", data.bridge_fee.to_string());

        Ok(resp)
    }
}

pub mod reply {
    use cosmos_sdk_proto_titan::ibc::applications::transfer::v1::MsgTransferResponse;
    use cosmwasm_std::{DepsMut, Env, Reply, Response, StdError};

    use crate::{constant::SUB_MSG_ID_WITHDRAW_IBC_1, error::ContractError, state};

    pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
        if msg.id != SUB_MSG_ID_WITHDRAW_IBC_1 {
            return Err(ContractError::Logic {
                err: format!("[withdraw] invalid sub msg id {:?}", msg.id),
            });
        }

        let data = match msg.result {
            cosmwasm_std::SubMsgResult::Err(err) => return Err(StdError::generic_err(err).into()),
            #[allow(deprecated)]
            cosmwasm_std::SubMsgResult::Ok(resp) => match resp.data {
                None => {
                    return Err(
                        StdError::generic_err("Receive empty response for MsgCreateDenom").into(),
                    )
                }
                Some(data) => data,
            },
        };

        let msg_transfer_response: MsgTransferResponse =
            prost::Message::decode(data.as_slice()).unwrap();

        state::withdraw::set_sended_withdraw(deps.storage, msg_transfer_response.sequence)?;

        let resp = Response::new();
        // @todo need return attributes or custom event here
        Ok(resp)
    }
}

pub mod sudo {
    use cosmwasm_std::{BankMsg, Coin, CosmosMsg, DepsMut, Env, Response, StdResult};

    use crate::{constant::TKX_NATIVE_DENOM, state, types::sudo::IBCLifecycleComplete};

    fn refund_to_user(deps: DepsMut, channel: &str, sequence: u64) -> StdResult<Response> {
        // get withdraw info
        let withdraw_info = state::withdraw::remove_withdraw_info(deps.storage, channel, sequence)?;
        let refund_tkx = Coin {
            denom: TKX_NATIVE_DENOM.to_string(),
            amount: withdraw_info.total_amount,
        };

        let send_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: withdraw_info.sender.into_string(),
            amount: vec![refund_tkx],
        });

        let resp = Response::new().add_message(send_msg);
        // @todo need return attributes or custom event here

        Ok(resp)
    }

    fn complete_withdraw(deps: DepsMut, channel: &str, sequence: u64) -> StdResult<Response> {
        state::withdraw::remove_withdraw_info(deps.storage, channel, sequence)?;
        let resp = Response::new();
        //@todo need return attributes or custom event here
        Ok(resp)
    }

    pub fn sudo(deps: DepsMut, env: Env, msg: IBCLifecycleComplete) -> StdResult<Response> {
        match msg {
            IBCLifecycleComplete::IBCAck {
                channel,
                sequence,
                ack,
                success,
            } => handle_ibc_ack(deps, env, channel, sequence, ack, success),
            IBCLifecycleComplete::IBCTimeout { channel, sequence } => {
                handle_ibc_timeout(deps, env, channel, sequence)
            }
        }
    }

    fn handle_ibc_ack(
        deps: DepsMut,
        _env: Env,
        channel: String,
        sequence: u64,
        _ack: String,
        success: bool,
    ) -> StdResult<Response> {
        match success {
            true => complete_withdraw(deps, &channel, sequence),
            false => refund_to_user(deps, &channel, sequence),
        }
    }

    fn handle_ibc_timeout(
        deps: DepsMut,
        _env: Env,
        channel: String,
        sequence: u64,
    ) -> StdResult<Response> {
        refund_to_user(deps, &channel, sequence)
    }
}
