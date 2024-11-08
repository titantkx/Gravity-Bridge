package types

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// ValidateBasic checks the ForeignReceiver is valid and foreign, the Amount is non-zero, the IbcChannel is
// non-empty, and the EventNonce is non-zero
func (p FailedIbcAutoForward) ValidateBasic() error {
	err := p.IbcPacket.ValidateBasic()
	if err != nil {
		return err
	}

	if p.EvmChainPrefix == "" {
		return sdkerrors.Wrap(ErrInvalid, "EvmChainPrefix must not be an empty string")
	}

	return nil
}
