package types

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func (msg IbcAutoSendEthMemo) ValidateBasic() error {
	if err := msg.SendToEth.ValidateBasic(); err != nil {
		return err
	}

	return nil
}

func (msg IbcAutoSendEth) ValidateBasic() error {
	if msg.EvmChainPrefix == "" {
		return sdkerrors.Wrap(ErrBadMetadataFormat, "EvmChainPrefix must not be an empty string")
	}

	if msg.Amount.IsNil() || !msg.Amount.IsPositive() {
		return sdkerrors.Wrap(ErrBadMetadataFormat, "amount must be positive")
	}

	if msg.BridgeFee != nil {
		if msg.BridgeFee.IsNil() || msg.BridgeFee.IsNegative() {
			return sdkerrors.Wrap(ErrBadMetadataFormat, "bridge fee must be positive or zero")
		}
	}

	if err := ValidateEthAddress(msg.EthDest); err != nil {
		return sdkerrors.Wrap(err, "ethereum address")
	}

	return nil
}
