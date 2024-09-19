package keeper

import (
	"encoding/json"
	"fmt"

	"github.com/Gravity-Bridge/Gravity-Bridge/module/x/gravity/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	capabilitytypes "github.com/cosmos/cosmos-sdk/x/capability/types"
	transfertypes "github.com/cosmos/ibc-go/v4/modules/apps/transfer/types"
	channeltypes "github.com/cosmos/ibc-go/v4/modules/core/04-channel/types"

	ibcexported "github.com/cosmos/ibc-go/v4/modules/core/exported"
)

const BatchFeesRate int64 = 0 // 0 / 1000

// IsModuleAccount returns true if the given account is a module account
func IsModuleAccount(acc authtypes.AccountI) bool {
	_, isModuleAccount := acc.(authtypes.ModuleAccountI)
	return isModuleAccount
}

// GetReceivedCoin returns the transferred coin from an ICS20 FungibleTokenPacketData
// as seen from the destination chain.
// If the receiving chain is the source chain of the tokens, it removes the prefix
// path added by source (i.e sender) chain to the denom. Otherwise, it adds the
// prefix path from the destination chain to the denom.
func GetReceivedCoin(srcPort, srcChannel, dstPort, dstChannel, rawDenom, rawAmt string) sdk.Coin {
	// NOTE: Denom and amount are already validated
	amount, _ := sdk.NewIntFromString(rawAmt)

	if transfertypes.ReceiverChainIsSource(srcPort, srcChannel, rawDenom) {
		// remove prefix added by sender chain
		voucherPrefix := transfertypes.GetDenomPrefix(srcPort, srcChannel)
		unprefixedDenom := rawDenom[len(voucherPrefix):]

		// coin denomination used in sending from the escrow address
		denom := unprefixedDenom

		// The denomination used to send the coins is either the native denom or the hash of the path
		// if the denomination is not native.
		denomTrace := transfertypes.ParseDenomTrace(unprefixedDenom)
		if denomTrace.Path != "" {
			denom = denomTrace.IBCDenom()
		}

		return sdk.Coin{
			Denom:  denom,
			Amount: amount,
		}
	}

	// since SendPacket did not prefix the denomination, we must prefix denomination here
	sourcePrefix := transfertypes.GetDenomPrefix(dstPort, dstChannel)
	// NOTE: sourcePrefix contains the trailing "/"
	prefixedDenom := sourcePrefix + rawDenom

	// construct the denomination trace from the full raw denomination
	denomTrace := transfertypes.ParseDenomTrace(prefixedDenom)
	voucherDenom := denomTrace.IBCDenom()

	return sdk.Coin{
		Denom:  voucherDenom,
		Amount: amount,
	}
}

// WriteAcknowledgement wraps IBC ICS4Wrapper GetAppVersion function.
func (k *Keeper) GetAppVersion(
	ctx sdk.Context,
	portID,
	channelID string,
) (string, bool) {
	return k.ics4Wrapper.GetAppVersion(ctx, portID, channelID)
}

func isIcs20Packet(data []byte) (isIcs20 bool, ics20data transfertypes.FungibleTokenPacketData) {
	var packetdata transfertypes.FungibleTokenPacketData
	if err := json.Unmarshal(data, &packetdata); err != nil {
		return false, packetdata
	}
	return true, packetdata
}

// OnRecvPacket performs the ICS20 middleware receive callback for automatically
// converting an IBC Coin to their ERC20 representation.
// For the conversion to succeed, the IBC denomination must have previously been
// registered via governance. Note that the native staking denomination (e.g. "aevmos"),
// is excluded from the conversion.
//
// CONTRACT: This middleware MUST be executed transfer after the ICS20 OnRecvPacket
// Return acknowledgement and continue with the next layer of the IBC middleware
// stack if:
// - memo is not EthDest
// - The base denomination is not registered as ERC20
func (k Keeper) OnRecvPacket(
	ctx sdk.Context,
	packet channeltypes.Packet,
	ack ibcexported.Acknowledgement,
) ibcexported.Acknowledgement {
	// must success to be here

	isIcs20, data := isIcs20Packet(packet.GetData())
	if !isIcs20 {
		return ack
	}

	// nothing to do
	if len(data.Memo) == 0 {
		return ack
	}

	// Validate the memo
	isSendToEthRouted, dest, amount, evmChainPrefix, err := ValidateAndParseMemo(data.Memo)
	if !isSendToEthRouted {
		return ack
	}
	if err != nil {
		return channeltypes.NewErrorAcknowledgement(err)
	}

	// Receiver become sender when send evm_prefix + contract_address token to evm
	sender, err := sdk.AccAddressFromBech32(data.Receiver)
	if err != nil {
		return channeltypes.NewErrorAcknowledgement(err)
	}

	senderAcc := k.accountKeeper.GetAccount(ctx, sender)

	// return error acknowledgement if sender is a module account
	if IsModuleAccount(senderAcc) {
		return channeltypes.NewErrorAcknowledgement(sdkerrors.Wrap(types.ErrInvalid, "transfer address cannot be a module account"))
	}

	// parse the transferred denom
	coin := GetReceivedCoin(
		packet.SourcePort, packet.SourceChannel,
		packet.DestinationPort, packet.DestinationChannel,
		data.Denom, data.Amount,
	)

	_, erc20, err := k.DenomToERC20Lookup(ctx, evmChainPrefix, coin.Denom)
	if err != nil {
		return channeltypes.NewErrorAcknowledgement(err)
	}

	amountToSendCoin := sdk.NewCoin(coin.Denom, amount)

	// verify coin is larger than amountToSendCoin
	if coin.Amount.LT(amountToSendCoin.Amount) {
		return channeltypes.NewErrorAcknowledgement(sdkerrors.Wrap(types.ErrInvalid, "total amount is less than amount to send"))
	}

	bridgeFeeCoin := coin.Sub(amountToSendCoin)

	if k.InvalidSendToEthAddress(ctx, evmChainPrefix, *dest, *erc20) {
		return channeltypes.NewErrorAcknowledgement(sdkerrors.Wrap(types.ErrInvalid, "destination address is invalid or blacklisted"))
	}

	// finally add to outgoing pool and waiting for gbt to submit it via MsgRequestBatch
	txID, err := k.AddToOutgoingPool(ctx, evmChainPrefix, sender, *dest, amountToSendCoin, bridgeFeeCoin)
	if err != nil {
		return channeltypes.NewErrorAcknowledgement(err)
	}

	err = ctx.EventManager().EmitTypedEvent(
		&types.EventOutgoingTxId{
			Message: "send_to_eth",
			TxId:    fmt.Sprint(txID),
		},
	)
	if err != nil {
		return channeltypes.NewErrorAcknowledgement(err)
	}

	return ack
}

// SendPacket wraps IBC ChannelKeeper's SendPacket function
func (k Keeper) SendPacket(ctx sdk.Context, chanCap *capabilitytypes.Capability, packet ibcexported.PacketI) error {
	return k.ics4Wrapper.SendPacket(ctx, chanCap, packet)
}

// WriteAcknowledgement writes the packet execution acknowledgement to the state,
// which will be verified by the counterparty chain using AcknowledgePacket.
func (k Keeper) WriteAcknowledgement(ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	packet ibcexported.PacketI,
	ack ibcexported.Acknowledgement,
) error {
	return k.ics4Wrapper.WriteAcknowledgement(ctx, chanCap, packet, ack)
}

// jsonStringHasKey parses the memo as a json object and checks if it contains the key.
func jsonStringHasKey(memo, key string) (found bool, jsonObject map[string]interface{}) {
	jsonObject = make(map[string]interface{})

	// If there is no memo, the packet was either sent with an earlier version of IBC, or the memo was
	// intentionally left blank. Nothing to do here. Ignore the packet and pass it down the stack.
	if len(memo) == 0 {
		return false, jsonObject
	}

	// the jsonObject must be a valid JSON object
	err := json.Unmarshal([]byte(memo), &jsonObject)
	if err != nil {
		return false, jsonObject
	}

	// If the key doesn't exist, there's nothing to do on this hook. Continue by passing the packet
	// down the stack
	_, ok := jsonObject[key]
	if !ok {
		return false, jsonObject
	}

	return true, jsonObject
}

func ValidateAndParseMemo(memo string) (isSendToEthRouted bool, dest *types.EthAddress, amount sdk.Int, evmChainPrefix string, err error) {
	isSendToEthRouted, metadata := jsonStringHasKey(memo, "send_to_eth")
	if !isSendToEthRouted {
		return isSendToEthRouted, nil, sdk.Int{}, "", nil
	}

	sendToEthRaw := metadata["send_to_eth"]

	// Make sure the wasm key is a map. If it isn't, ignore this packet
	wasm, ok := sendToEthRaw.(map[string]interface{})
	if !ok {
		return isSendToEthRouted, nil, sdk.Int{}, "",
			sdkerrors.Wrapf(types.ErrBadMetadataFormat, "send_to_eth metadata not properly formatted for: '%v'. %s", memo, "wasm metadata is not a valid JSON map object")
	}

	// Get the eth_dest
	ethDest, ok := wasm["eth_dest"].(string)
	if !ok {
		// The tokens will be returned
		return isSendToEthRouted, nil, sdk.Int{}, "",
			sdkerrors.Wrapf(types.ErrBadMetadataFormat, `Could not find key send_to_eth["eth_dest"]`)
	}

	dest, err = types.NewEthAddress(ethDest)
	if err != nil {
		return isSendToEthRouted, nil, sdk.Int{}, "", sdkerrors.Wrapf(types.ErrBadMetadataFormat, `invalid eth dest`)
	}

	amountToSend, ok := wasm["amount"].(string)
	if !ok {
		// The tokens will be returned
		return isSendToEthRouted, nil, sdk.Int{}, "",
			sdkerrors.Wrapf(types.ErrBadMetadataFormat, `Could not find key send_to_eth["amount"]`)
	}
	amountToSendInt, ok := sdk.NewIntFromString(amountToSend)
	if !ok {
		return isSendToEthRouted, nil, sdk.Int{}, "",
			sdkerrors.Wrapf(types.ErrBadMetadataFormat, "error parsing amount : %s", amountToSend)
	}
	// amountToSendInt must be positive
	if amountToSendInt.IsNegative() {
		return isSendToEthRouted, nil, sdk.Int{}, "",
			sdkerrors.Wrapf(types.ErrBadMetadataFormat, "amount must be positive")
	}

	evmChainPrefix, ok = wasm["evm_chain_prefix"].(string)
	if !ok {
		// The tokens will be returned
		return isSendToEthRouted, nil, sdk.Int{}, "",
			sdkerrors.Wrapf(types.ErrBadMetadataFormat, `Could not find key send_to_eth["evm_chain_prefix"]`)
	}

	return isSendToEthRouted, dest, amountToSendInt, evmChainPrefix, nil
}
