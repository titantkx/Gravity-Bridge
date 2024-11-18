package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
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

	amountInt, err := ValidateAmountString(msg.Amount)
	if err != nil {
		return sdkerrors.Wrap(ErrBadMetadataFormat, "amount")
	}
	if amountInt.IsZero() {
		return sdkerrors.Wrap(ErrBadMetadataFormat, "amount must be positive")
	}

	_, err = ValidateAmountString(msg.BridgeFee)
	if err != nil {
		return sdkerrors.Wrap(ErrBadMetadataFormat, "bridge fee")
	}

	if err := ValidateEthAddress(msg.EthDest); err != nil {
		return sdkerrors.Wrap(err, "ethereum address")
	}

	return nil
}

func ValidateAmountString(amount string) (sdk.Int, error) {
	amountInt, ok := sdk.NewIntFromString(amount)
	if !ok {
		return sdk.Int{}, sdkerrors.Wrapf(ErrBadMetadataFormat, "error parsing amount : %s", amount)
	}
	// amountToSendInt must be positive or zero
	if amountInt.IsNegative() {
		return sdk.Int{}, sdkerrors.Wrapf(ErrBadMetadataFormat, "amount must be positive")
	}

	return amountInt, nil
}
