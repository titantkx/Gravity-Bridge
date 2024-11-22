use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::Map;

// This map store chain_prefix -> ibc denom
const TKX_IBC_TOKEN_DENOM: Map<String, String> = Map::new("tkx_ibc_token_denom");

pub fn add_tkx_ibc_token_denom(
    store: &mut dyn Storage,
    chain_prefix: &str,
    denom: &str,
) -> StdResult<()> {
    TKX_IBC_TOKEN_DENOM.save(store, chain_prefix.to_string(), &denom.to_string())
}

pub fn remove_tkx_chain(store: &mut dyn Storage, chain_prefix: &str) {
    TKX_IBC_TOKEN_DENOM.remove(store, chain_prefix.to_string())
}

pub fn is_tkx_ibc_token_denom(store: &dyn Storage, denom: &str) -> bool {
    // get
    let all_denoms = list_tkx_ibc_token_denoms(store);
    all_denoms.contains(&denom.to_string())
}

pub fn list_tkx_ibc_token_denoms(store: &dyn Storage) -> Vec<String> {
    TKX_IBC_TOKEN_DENOM
        .range(store, None, None, Order::Ascending)
        .map(|item| {
            let (_, denom) = item.unwrap();
            denom
        })
        .collect()
}

pub fn list_tkx_chain_with_ibc_token_denoms(store: &dyn Storage) -> Vec<(String, String)> {
    TKX_IBC_TOKEN_DENOM
        .range(store, None, None, Order::Ascending)
        .map(|item| {
            let (chain_prefix, denom) = item.unwrap();
            (chain_prefix, denom)
        })
        .collect()
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
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix, denom).unwrap();

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom);
        assert!(exists);
    }

    #[test]
    fn test_remove_tkx_ibc_token_denom() {
        let mut deps = mock_dependencies();

        let chain_prefix = "eth";
        let denom = "tkx_denom";
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix, denom).unwrap();

        remove_tkx_chain(&mut deps.storage, chain_prefix);

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom);
        assert!(!exists);
    }

    #[test]
    fn test_list_tkx_ibc_token_denoms() {
        let mut deps = mock_dependencies();

        let chain_prefix1 = "eth";
        let denom1 = "tkx_denom1";
        let chain_prefix2 = "bsc";
        let denom2 = "tkx_denom2";
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix1, denom1).unwrap();
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix2, denom2).unwrap();

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
        let chain_prefix2 = "bsc";
        let denom2 = "tkx_denom2";
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix1, denom1).unwrap();
        add_tkx_ibc_token_denom(&mut deps.storage, chain_prefix2, denom2).unwrap();

        let denoms = list_tkx_chain_with_ibc_token_denoms(&deps.storage);
        assert_eq!(denoms.len(), 2);
        assert!(denoms.contains(&(chain_prefix1.to_string(), denom1.to_string())));
        assert!(denoms.contains(&(chain_prefix2.to_string(), denom2.to_string())));
    }
}
