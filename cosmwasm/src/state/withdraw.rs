use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Item, Map};

use crate::types::state::{SendingWithdrawInfo, WithdrawInfo};

const SENDING_WITHDRAW: Item<SendingWithdrawInfo> = Item::new("sending_withdraw");

/// This map store `[channel_id]/[sequence]` -> withdraw info
const WITHDRAW_BY_CHANNEL_SEQUENCE: Map<String, WithdrawInfo> =
    Map::new("withdraw_by_channel_sequence");

pub fn set_sending_withdraw(store: &mut dyn Storage, data: &SendingWithdrawInfo) -> StdResult<()> {
    SENDING_WITHDRAW.save(store, data)
}

pub fn set_sended_withdraw(store: &mut dyn Storage, sequence: u64) -> StdResult<()> {
    let data = SENDING_WITHDRAW.load(store)?;
    let withdraw_info = WithdrawInfo {
        ibc_channel_id: data.ibc_channel_id.clone(),
        sequence,
        chain_prefix: data.chain_prefix,
        sender: data.sender,
        recipient: data.recipient,
        forwarder: data.forwarder,
        total_amount: data.total_amount,
        amount: data.amount,
        bridge_fee: data.bridge_fee,
    };
    let key = format!("{}/{}", data.ibc_channel_id, sequence.to_string());
    // move sending withdraw to withdraw map
    SENDING_WITHDRAW.remove(store);
    WITHDRAW_BY_CHANNEL_SEQUENCE.save(store, key, &withdraw_info)
}

pub fn remove_withdraw_info(
    store: &mut dyn Storage,
    channel: &str,
    sequence: u64,
) -> StdResult<WithdrawInfo> {
    let key = format!("{}/{}", channel, sequence.to_string());
    let withdraw_info = WITHDRAW_BY_CHANNEL_SEQUENCE.load(store, key.clone())?;
    WITHDRAW_BY_CHANNEL_SEQUENCE.remove(store, key);
    Ok(withdraw_info)
}

pub fn get_withdraw_info(
    store: &dyn Storage,
    channel: &str,
    sequence: u64,
) -> StdResult<WithdrawInfo> {
    let key = format!("{}/{}", channel, sequence.to_string());
    WITHDRAW_BY_CHANNEL_SEQUENCE.load(store, key)
}

#[cfg(test)]
mod tests {

    use super::*;
    use cosmwasm_std::{testing::mock_dependencies, Addr};

    #[test]
    fn test_set_sending_withdraw() {
        let mut deps = mock_dependencies();
        let data = SendingWithdrawInfo {
            ibc_channel_id: "channel-0".to_string(),
            chain_prefix: "eth".to_string(),
            sender: Addr::unchecked("sender"),
            recipient: "recipient".to_string(),
            forwarder: "forwarder".to_string(),
            total_amount: 1000u128.into(),
            amount: 900u128.into(),
            bridge_fee: 100u128.into(),
        };

        set_sending_withdraw(&mut deps.storage, &data).unwrap();

        let stored_data = SENDING_WITHDRAW.load(&deps.storage).unwrap();
        assert_eq!(stored_data, data);
    }

    #[test]
    fn test_set_sended_withdraw() {
        let mut deps = mock_dependencies();
        let data = SendingWithdrawInfo {
            ibc_channel_id: "channel-0".to_string(),
            chain_prefix: "eth".to_string(),
            sender: Addr::unchecked("sender"),
            recipient: "recipient".to_string(),
            forwarder: "forwarder".to_string(),
            total_amount: 1000u128.into(),
            amount: 900u128.into(),
            bridge_fee: 100u128.into(),
        };

        set_sending_withdraw(&mut deps.storage, &data).unwrap();
        set_sended_withdraw(&mut deps.storage, 1).unwrap();

        let key = format!("{}/{}", data.ibc_channel_id, 1);
        let stored_data = WITHDRAW_BY_CHANNEL_SEQUENCE
            .load(&deps.storage, key)
            .unwrap();
        let expected_data = WithdrawInfo {
            ibc_channel_id: data.ibc_channel_id,
            sequence: 1,
            chain_prefix: data.chain_prefix,
            sender: data.sender,
            recipient: data.recipient,
            forwarder: data.forwarder,
            total_amount: data.total_amount,
            amount: data.amount,
            bridge_fee: data.bridge_fee,
        };
        assert_eq!(stored_data, expected_data);

        let stored_data = SENDING_WITHDRAW.load(&deps.storage);
        assert!(stored_data.is_err());
    }

    #[test]
    fn test_remove_withdraw_info() {
        let mut deps = mock_dependencies();
        let data = SendingWithdrawInfo {
            ibc_channel_id: "channel-0".to_string(),
            chain_prefix: "eth".to_string(),
            sender: Addr::unchecked("sender"),
            recipient: "recipient".to_string(),
            forwarder: "forwarder".to_string(),
            total_amount: 1000u128.into(),
            amount: 900u128.into(),
            bridge_fee: 100u128.into(),
        };

        set_sending_withdraw(&mut deps.storage, &data).unwrap();
        set_sended_withdraw(&mut deps.storage, 1).unwrap();

        let key = format!("{}/{}", data.ibc_channel_id, 1);
        let stored_data = remove_withdraw_info(&mut deps.storage, &data.ibc_channel_id, 1).unwrap();
        let expected_data = WithdrawInfo {
            ibc_channel_id: data.ibc_channel_id,
            sequence: 1,
            chain_prefix: data.chain_prefix,
            sender: data.sender,
            recipient: data.recipient,
            forwarder: data.forwarder,
            total_amount: data.total_amount,
            amount: data.amount,
            bridge_fee: data.bridge_fee,
        };
        assert_eq!(stored_data, expected_data);

        let stored_data = WITHDRAW_BY_CHANNEL_SEQUENCE.load(&deps.storage, key);
        assert!(stored_data.is_err());
    }
}
