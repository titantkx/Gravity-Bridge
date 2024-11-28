use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::Map;

use crate::types::state::TKXIbcInfo;

// This map store chain_prefix -> ibc denom
const TKX_IBC_TOKEN_INFO: Map<String, TKXIbcInfo> = Map::new("tkx_ibc_token_info");

pub fn add_tkx_ibc_token_denom(
    store: &mut dyn Storage,
    chain_prefix: &str,
    denom: &str,
    channel_id: &str,
) -> StdResult<()> {
    TKX_IBC_TOKEN_INFO.save(
        store,
        chain_prefix.to_string(),
        &TKXIbcInfo {
            denom: denom.to_string(),
            channel_id: channel_id.to_string(),
        },
    )
}

pub fn remove_tkx_chain(store: &mut dyn Storage, chain_prefix: &str) {
    TKX_IBC_TOKEN_INFO.remove(store, chain_prefix.to_string())
}

#[allow(dead_code)]
pub fn is_tkx_ibc_token_denom(store: &dyn Storage, denom: &str) -> bool {
    // get
    let all_denoms = list_tkx_ibc_token_denoms(store);
    all_denoms.contains(&denom.to_string())
}

pub fn list_tkx_ibc_token_denoms(store: &dyn Storage) -> Vec<String> {
    TKX_IBC_TOKEN_INFO
        .range(store, None, None, Order::Ascending)
        .map(|item| {
            let (_, info) = item.unwrap();
            info.denom
        })
        .collect()
}

pub fn list_tkx_chain_with_ibc_token_denoms(store: &dyn Storage) -> Vec<(String, TKXIbcInfo)> {
    TKX_IBC_TOKEN_INFO
        .range(store, None, None, Order::Ascending)
        .map(|item| {
            let (chain_prefix, denom) = item.unwrap();
            (chain_prefix, denom)
        })
        .collect()
}

pub fn get_tkx_ibc_token_by_chain_prefix(
    store: &dyn Storage,
    chain_prefix: &str,
) -> StdResult<TKXIbcInfo> {
    TKX_IBC_TOKEN_INFO.load(store, chain_prefix.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn test_add_and_is_tkx_ibc_token_denom() {
        let mut deps = mock_dependencies();

        let chain_prefix = "eth";
        let denom = "tkx_denom";
        let channel_id = "channel-0";
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix, denom, channel_id).unwrap();

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom);
        assert!(exists);
    }

    #[test]
    fn test_remove_tkx_ibc_token_denom() {
        let mut deps = mock_dependencies();

        let chain_prefix = "eth";
        let denom = "tkx_denom";
        let channel_id = "channel-0";
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix, denom, channel_id).unwrap();

        remove_tkx_chain(&mut deps.storage, chain_prefix);

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom);
        assert!(!exists);
    }

    #[test]
    fn test_list_tkx_ibc_token_denoms() {
        let mut deps = mock_dependencies();

        let chain_prefix1 = "eth";
        let denom1 = "tkx_denom1";
        let channel_id = "channel-0";
        let chain_prefix2 = "bsc";
        let denom2 = "tkx_denom2";
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix1, denom1, channel_id).unwrap();
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix2, denom2, channel_id).unwrap();

        let denoms = list_tkx_ibc_token_denoms(&deps.storage);
        assert_eq!(denoms.len(), 2);
        assert!(denoms.contains(&denom1.to_string()));
        assert!(denoms.contains(&denom2.to_string()));
    }

    #[test]
    fn test_list_tkx_chain_with_ibc_token_denoms() {
        let mut deps = mock_dependencies();

        let chain_prefix1 = "eth";
        let denom1 = "tkx_denom1";
        let channel_id = "channel-0";
        let chain_prefix2 = "bsc";
        let denom2 = "tkx_denom2";
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix1, denom1, channel_id).unwrap();
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix2, denom2, channel_id).unwrap();

        let denoms = list_tkx_chain_with_ibc_token_denoms(&deps.storage);
        assert_eq!(denoms.len(), 2);
        assert!(denoms.contains(&(
            chain_prefix1.to_string(),
            TKXIbcInfo {
                denom: denom1.to_string(),
                channel_id: channel_id.to_string(),
            }
        )));
        assert!(denoms.contains(&(
            chain_prefix2.to_string(),
            TKXIbcInfo {
                denom: denom2.to_string(),
                channel_id: channel_id.to_string(),
            }
        )));
    }

    #[test]
    fn test_get_tkx_ibc_token_by_chain_prefix() {
        let mut deps = mock_dependencies();

        let chain_prefix = "eth";
        let denom = "tkx_denom";
        let channel_id = "channel-0";
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix, denom, channel_id).unwrap();

        let result = get_tkx_ibc_token_by_chain_prefix(&deps.storage, chain_prefix).unwrap();
        assert_eq!(
            result,
            TKXIbcInfo {
                denom: denom.to_string(),
                channel_id: channel_id.to_string(),
            }
        );

        let result = get_tkx_ibc_token_by_chain_prefix(&deps.storage, "bsc");
        assert!(result.is_err());
    }
}
