use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Item;

use crate::error::ContractError;

const ADMIN: Item<Addr> = Item::new("admin");

pub fn get_admin(store: &dyn Storage) -> StdResult<Addr> {
    ADMIN.load(store)
}

pub fn set_admin(store: &mut dyn Storage, address: &Addr) -> StdResult<()> {
    ADMIN.save(store, address)
}

pub fn check_admin(store: &dyn Storage, address: &Addr) -> Result<(), ContractError> {
    let admin = get_admin(store)?;

    if address != admin {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn test_set_and_get_admin() {
        let mut deps = mock_dependencies();

        let admin_addr = Addr::unchecked("admin");
        set_admin(&mut deps.storage, &admin_addr).unwrap();

        let loaded_admin = get_admin(&deps.storage).unwrap();
        assert_eq!(loaded_admin, admin_addr);
    }

    #[test]
    fn test_check_admin_success() {
        let mut deps = mock_dependencies();

        let admin_addr = Addr::unchecked("admin");
        set_admin(&mut deps.storage, &admin_addr).unwrap();

        let result = check_admin(&deps.storage, &admin_addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_admin_failure() {
        let mut deps = mock_dependencies();

        let admin_addr = Addr::unchecked("admin");
        let other_addr = Addr::unchecked("other");
        set_admin(&mut deps.storage, &admin_addr).unwrap();

        let result = check_admin(&deps.storage, &other_addr);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::Unauthorized {});
    }
}
