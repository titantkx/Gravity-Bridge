use deep_space::{
    error::PrivateKeyError, Address, CosmosPrivateKey, EthermintPrivateKey, MessageArgs, Msg,
    PrivateKey,
};
use gravity_proto::cosmos_sdk_proto::cosmos::tx::v1beta1::Tx;

use crate::IBC_ADDRESS_TYPE;

#[derive(Debug, PartialEq)]
pub enum IBCChainAddressType {
    Cosmos,
    Ethermint,
}

impl std::str::FromStr for IBCChainAddressType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cosmos" => Ok(Self::Cosmos),
            "ethermint" => Ok(Self::Ethermint),
            _ => Err(format!("Invalid IBCChainAddressType: {}", s)),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum IBCPrivateKey {
    Cosmos(CosmosPrivateKey),
    Ethermint(EthermintPrivateKey),
}

impl PrivateKey for IBCPrivateKey {
    fn from_secret(secret: &[u8]) -> IBCPrivateKey {
        match *IBC_ADDRESS_TYPE {
            IBCChainAddressType::Cosmos => {
                IBCPrivateKey::Cosmos(CosmosPrivateKey::from_secret(secret))
            }
            IBCChainAddressType::Ethermint => {
                IBCPrivateKey::Ethermint(EthermintPrivateKey::from_secret(secret))
            }
        }
    }

    fn from_phrase(phrase: &str, passphrase: &str) -> Result<IBCPrivateKey, PrivateKeyError> {
        match *IBC_ADDRESS_TYPE {
            IBCChainAddressType::Cosmos => {
                let key = CosmosPrivateKey::from_phrase(phrase, passphrase)?;
                Ok(IBCPrivateKey::Cosmos(key))
            }
            IBCChainAddressType::Ethermint => {
                let key = EthermintPrivateKey::from_phrase(phrase, passphrase)?;
                Ok(IBCPrivateKey::Ethermint(key))
            }
        }
    }

    fn from_hd_wallet_path(
        path: &str,
        phrase: &str,
        passphrase: &str,
    ) -> Result<Self, PrivateKeyError> {
        match *IBC_ADDRESS_TYPE {
            IBCChainAddressType::Cosmos => {
                let key = CosmosPrivateKey::from_hd_wallet_path(path, phrase, passphrase)?;
                Ok(IBCPrivateKey::Cosmos(key))
            }
            IBCChainAddressType::Ethermint => {
                let key = EthermintPrivateKey::from_hd_wallet_path(path, phrase, passphrase)?;
                Ok(IBCPrivateKey::Ethermint(key))
            }
        }
    }

    fn to_address(&self, prefix: &str) -> Result<Address, PrivateKeyError> {
        match self {
            IBCPrivateKey::Cosmos(key) => key.to_address(prefix),
            IBCPrivateKey::Ethermint(key) => key.to_address(prefix),
        }
    }

    fn get_signed_tx(
        &self,
        messages: &[Msg],
        args: MessageArgs,
        memo: &str,
    ) -> Result<Tx, PrivateKeyError> {
        match self {
            IBCPrivateKey::Cosmos(key) => key.get_signed_tx(messages, args, memo),
            IBCPrivateKey::Ethermint(key) => key.get_signed_tx(messages, args, memo),
        }
    }

    fn sign_std_msg(
        &self,
        messages: &[Msg],
        args: MessageArgs,
        memo: &str,
    ) -> Result<Vec<u8>, PrivateKeyError> {
        match self {
            IBCPrivateKey::Cosmos(key) => key.sign_std_msg(messages, args, memo),
            IBCPrivateKey::Ethermint(key) => key.sign_std_msg(messages, args, memo),
        }
    }
}
