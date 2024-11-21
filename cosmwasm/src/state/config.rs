use cosmwasm_std::{Empty, Order, StdResult, Storage};
use cw_storage_plus::Map;

const TKX_IBC_TOKEN_DENOM: Map<String, Empty> = Map::new("tkx_ibc_token_denom");

pub fn add_tkx_ibc_token_denom(store: &mut dyn Storage, denom: String) -> StdResult<()> {
    TKX_IBC_TOKEN_DENOM.save(store, denom, &Empty {})
}

pub fn remove_tkx_ibc_token_denom(store: &mut dyn Storage, denom: String) {
    TKX_IBC_TOKEN_DENOM.remove(store, denom)
}

pub fn is_tkx_ibc_token_denom(store: &dyn Storage, denom: String) -> bool {
    TKX_IBC_TOKEN_DENOM.has(store, denom)
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

        let denom = "tkx_denom".to_string();
        add_tkx_ibc_token_denom(&mut deps.storage, denom.clone()).unwrap();

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom.clone());
        assert!(exists);
    }

    #[test]
    fn test_remove_tkx_ibc_token_denom() {
        let mut deps = mock_dependencies();

        let denom = "tkx_denom".to_string();
        add_tkx_ibc_token_denom(&mut deps.storage, denom.clone()).unwrap();

        remove_tkx_ibc_token_denom(&mut deps.storage, denom.clone());

        let exists = is_tkx_ibc_token_denom(&deps.storage, denom.clone());
        assert!(!exists);
    }

    #[test]
    fn test_list_tkx_ibc_token_denoms() {
        let mut deps = mock_dependencies();

        let denom1 = "tkx_denom1".to_string();
        let denom2 = "tkx_denom2".to_string();
        add_tkx_ibc_token_denom(&mut deps.storage, denom1.clone()).unwrap();
        add_tkx_ibc_token_denom(&mut deps.storage, denom2.clone()).unwrap();

        let denoms = list_tkx_ibc_token_denoms(&deps.storage);
        assert_eq!(denoms.len(), 2);
        assert!(denoms.contains(&denom1));
        assert!(denoms.contains(&denom2));
    }
}
