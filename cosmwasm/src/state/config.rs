use cosmwasm_std::{Empty, Order, StdResult, Storage};
use cw_storage_plus::Map;

const TKX_IBC_TOKEN_DENOM: Map<String, Empty> = Map::new("tkx_ibc_token_denom");

pub fn add_tkx_ibc_token_denom(store: &mut dyn Storage, denom: &str) -> StdResult<()> {
    TKX_IBC_TOKEN_DENOM.save(store, denom.to_string(), &Empty {})
}

pub fn remove_tkx_ibc_token_denom(store: &mut dyn Storage, denom: &str) {
    TKX_IBC_TOKEN_DENOM.remove(store, denom.to_string())
}

pub fn is_tkx_ibc_token_denom(store: &dyn Storage, denom: &str) -> bool {
    TKX_IBC_TOKEN_DENOM.has(store, denom.to_string())
}

pub fn list_tkx_ibc_token_denoms(store: &dyn Storage) -> Vec<String> {
    TKX_IBC_TOKEN_DENOM
        .range(store, None, None, Order::Ascending)
        .map(|item| {
            let (key, _) = item.unwrap();
            key
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

        let denom = "tkx_denom";
        add_tkx_ibc_token_denom(&mut deps.storage, denom).unwrap();

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom);
        assert!(exists);
    }

    #[test]
    fn test_remove_tkx_ibc_token_denom() {
        let mut deps = mock_dependencies();

        let denom = "tkx_denom";
        add_tkx_ibc_token_denom(&mut deps.storage, denom).unwrap();

        remove_tkx_ibc_token_denom(&mut deps.storage, denom);

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom);
        assert!(!exists);
    }

    #[test]
    fn test_list_tkx_ibc_token_denoms() {
        let mut deps = mock_dependencies();

        let denom1 = "tkx_denom1";
        let denom2 = "tkx_denom2";
        add_tkx_ibc_token_denom(&mut deps.storage, denom1).unwrap();
        add_tkx_ibc_token_denom(&mut deps.storage, denom2).unwrap();

        let denoms = list_tkx_ibc_token_denoms(&deps.storage);
        assert_eq!(denoms.len(), 2);
        assert!(denoms.contains(&denom1.to_string()));
        assert!(denoms.contains(&denom2.to_string()));
    }
}
