use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::Map;

use crate::types::state::TKXIbcInfo;

// This map store chain_prefix -> ibc denom
const TKX_IBC_TOKEN_INFO: Map<String, TKXIbcInfo> = Map::new("tkx_ibc_token_info");
const TKX_IBC_TOKEN_INFO_DENOM_INDEX: Map<String, String> =
    Map::new("tkx_ibc_token_info_denom_index");

pub fn add_tkx_ibc_token_denom(
    store: &mut dyn Storage,
    chain_prefix: &str,
    address_regex: &str,
    denom: &str,
    decimals: u8,
    channel_id: &str,
) -> StdResult<()> {
    TKX_IBC_TOKEN_INFO.save(
        store,
        chain_prefix.to_string(),
        &TKXIbcInfo {
            address_regex: address_regex.to_string(),
            denom: denom.to_string(),
            decimals,
            channel_id: channel_id.to_string(),
        },
    )?;

    // update denom index
    TKX_IBC_TOKEN_INFO_DENOM_INDEX.save(store, denom.to_string(), &chain_prefix.to_string())
}

pub fn remove_tkx_chain(store: &mut dyn Storage, chain_prefix: &str) {
    // load current
    let info = TKX_IBC_TOKEN_INFO
        .may_load(store, chain_prefix.to_string())
        .unwrap();
    if info.is_none() {
        return;
    }
    let info = info.unwrap();
    // remove denom index
    TKX_IBC_TOKEN_INFO_DENOM_INDEX.remove(store, info.denom.to_string());
    // remove chain prefix
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

pub fn get_tkx_ibc_token_by_denom(store: &dyn Storage, denom: &str) -> StdResult<TKXIbcInfo> {
    let chain_prefix = TKX_IBC_TOKEN_INFO_DENOM_INDEX.load(store, denom.to_string())?;
    TKX_IBC_TOKEN_INFO.load(store, chain_prefix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn test_add_and_is_tkx_ibc_token_denom() {
        let mut deps = mock_dependencies();

        let chain_prefix = "eth";
        let address_regex = "0x[a-fA-F0-9]{40}";
        let denom = "tkx_denom";
        let decimals = 8;
        let channel_id = "channel-0";
        add_tkx_ibc_token_denom(
            &mut deps.storage,
            chain_prefix,
            address_regex,
            denom,
            decimals,
            channel_id,
        )
        .unwrap();

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom);
        assert!(exists);
    }

    #[test]
    fn test_remove_tkx_ibc_token_denom() {
        let mut deps = mock_dependencies();

        let chain_prefix = "eth";
        let address_regex = "0x[a-fA-F0-9]{40}";
        let denom = "tkx_denom";
        let decimals = 8;
        let channel_id = "channel-0";
        add_tkx_ibc_token_denom(
            &mut deps.storage,
            chain_prefix,
            address_regex,
            denom,
            decimals,
            channel_id,
        )
        .unwrap();

        remove_tkx_chain(&mut deps.storage, chain_prefix);

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom);
        assert!(!exists);
    }

    #[test]
    fn test_list_tkx_ibc_token_denoms() {
        let mut deps = mock_dependencies();

        let chain_prefix1 = "eth";
        let address_regex1 = "0x[a-fA-F0-9]{40}";
        let denom1 = "tkx_denom1";
        let decimals1 = 8;
        let channel_id = "channel-0";
        let chain_prefix2 = "bsc";
        let address_regex2 = "0x[a-fA-F0-9]{40}";
        let decimals2 = 6;
        let denom2 = "tkx_denom2";
        add_tkx_ibc_token_denom(
            &mut deps.storage,
            chain_prefix1,
            address_regex1,
            denom1,
            decimals1,
            channel_id,
        )
        .unwrap();
        add_tkx_ibc_token_denom(
            &mut deps.storage,
            chain_prefix2,
            address_regex2,
            denom2,
            decimals2,
            channel_id,
        )
        .unwrap();

        let denoms = list_tkx_ibc_token_denoms(&deps.storage);
        assert_eq!(denoms.len(), 2);
        assert!(denoms.contains(&denom1.to_string()));
        assert!(denoms.contains(&denom2.to_string()));
    }

    #[test]
    fn test_list_tkx_chain_with_ibc_token_denoms() {
        let mut deps = mock_dependencies();

        let chain_prefix1 = "eth";
        let address_regex1 = "0x[a-fA-F0-9]{40}";
        let denom1 = "tkx_denom1";
        let decimals1 = 8;
        let channel_id = "channel-0";
        let chain_prefix2 = "bsc";
        let address_regex2 = "0x[a-fA-F0-9]{40}";
        let denom2 = "tkx_denom2";
        let decimals2 = 6;
        add_tkx_ibc_token_denom(
            &mut deps.storage,
            chain_prefix1,
            address_regex1,
            denom1,
            decimals1,
            channel_id,
        )
        .unwrap();
        add_tkx_ibc_token_denom(
            &mut deps.storage,
            chain_prefix2,
            address_regex2,
            denom2,
            decimals2,
            channel_id,
        )
        .unwrap();

        let denoms = list_tkx_chain_with_ibc_token_denoms(&deps.storage);
        assert_eq!(denoms.len(), 2);
        assert!(denoms.contains(&(
            chain_prefix1.to_string(),
            TKXIbcInfo {
                address_regex: address_regex1.to_string(),
                denom: denom1.to_string(),
                decimals: decimals1,
                channel_id: channel_id.to_string(),
            }
        )));
        assert!(denoms.contains(&(
            chain_prefix2.to_string(),
            TKXIbcInfo {
                address_regex: address_regex2.to_string(),
                denom: denom2.to_string(),
                decimals: decimals2,
                channel_id: channel_id.to_string(),
            }
        )));
    }

    #[test]
    fn test_get_tkx_ibc_token_by_chain_prefix() {
        let mut deps = mock_dependencies();

        let chain_prefix = "eth";
        let address_regex = "0x[a-fA-F0-9]{40}";
        let denom = "tkx_denom";
        let decimals = 8;
        let channel_id = "channel-0";
        add_tkx_ibc_token_denom(
            &mut deps.storage,
            chain_prefix,
            address_regex,
            denom,
            decimals,
            channel_id,
        )
        .unwrap();

        let result = get_tkx_ibc_token_by_chain_prefix(&deps.storage, chain_prefix).unwrap();
        assert_eq!(
            result,
            TKXIbcInfo {
                address_regex: address_regex.to_string(),
                denom: denom.to_string(),
                decimals,
                channel_id: channel_id.to_string(),
            }
        );

        let result = get_tkx_ibc_token_by_chain_prefix(&deps.storage, "bsc");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_tkx_ibc_token_by_denom() {
        let mut deps = mock_dependencies();

        let chain_prefix = "eth";
        let address_regex = "0x[a-fA-F0-9]{40}";
        let denom = "tkx_denom";
        let decimals = 8;
        let channel_id = "channel-0";
        add_tkx_ibc_token_denom(
            &mut deps.storage,
            chain_prefix,
            address_regex,
            denom,
            decimals,
            channel_id,
        )
        .unwrap();

        let result = get_tkx_ibc_token_by_denom(&deps.storage, denom).unwrap();
        assert_eq!(
            result,
            TKXIbcInfo {
                address_regex: address_regex.to_string(),
                denom: denom.to_string(),
                decimals,
                channel_id: channel_id.to_string(),
            }
        );

        let result = get_tkx_ibc_token_by_denom(&deps.storage, "non_existent_denom");
        assert!(result.is_err());
    }
}
