package keeper

import (
	"crypto/sha256"
	"fmt"
	"testing"

	"github.com/Gravity-Bridge/Gravity-Bridge/module/x/gravity/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

	// sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	transfertypes "github.com/cosmos/ibc-go/v4/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v4/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v4/modules/core/04-channel/types"
	ibcmock "github.com/cosmos/ibc-go/v4/testing/mock"
	"github.com/stretchr/testify/require"
)

func TestOnRecvPacket(t *testing.T) {
	input := CreateTestEnv(t)
	defer func() { input.Context.Logger().Info("Asserting invariants at test end"); input.AssertInvariants() }()

	ctx := input.Context

	var (
		// Setup Cosmoschain <=> Gravity Bridge IBC relayer
		sourceChannel     = "channel-0"
		gravityChannel    = "channel-1"
		tokenContractAddr = "0x429881672B9AE42b8EbA0E26cD9C73711b891Ca5"
		ethDestAddr       = "0xd041c41EA1bf0F006ADBb6d2c9ef9D425dE5eaD7"
		// if not create Claim from Deploy Erc20 contract, then denom = prefix + contract
		myTokenDenom = "ethereum" + tokenContractAddr
		ibcDenom     = fmt.Sprintf("ibc/%X", sha256.Sum256([]byte("transfer/"+gravityChannel+"/"+myTokenDenom)))
		evmChain     = input.GravityKeeper.GetEvmChainData(ctx, EthChainPrefix)
	)

	tokenAddr, err := types.NewEthAddress(tokenContractAddr)
	require.NoError(t, err)

	// secp256k1 account for cosmoschain
	secpPk := secp256k1.GenPrivKey()
	gravityAddr := sdk.AccAddress(secpPk.PubKey().Address())
	cosmosAddr := sdk.MustBech32ifyAddressBytes("cosmos", gravityAddr)

	path := fmt.Sprintf("%s/%s", transfertypes.PortID, gravityChannel)

	timeoutHeight := clienttypes.NewHeight(0, 100)
	expAck := ibcmock.MockAcknowledgement

	// add it to the ERC20 registry
	// because this is one way from Cosmoschain to Gravity Bridge so just use the ibc token as default native token and mint some
	for _, evmChain := range input.GravityKeeper.GetEvmChains(ctx) {
		input.GravityKeeper.setCosmosOriginatedDenomToERC20(ctx, evmChain.EvmChainPrefix, ibcDenom, *tokenAddr)
		isCosmosOriginated, addr, err := input.GravityKeeper.DenomToERC20Lookup(ctx, evmChain.EvmChainPrefix, ibcDenom)
		require.True(t, isCosmosOriginated)
		require.NoError(t, err)
		require.Equal(t, tokenAddr.GetAddress().Hex(), tokenContractAddr)
		require.Equal(t, tokenAddr, addr)
	}

	require.NoError(t, input.BankKeeper.MintCoins(input.Context, types.ModuleName, sdk.NewCoins(
		sdk.NewCoin(ibcDenom, sdk.NewInt(1000)), // some IBC coin with a registered token pair
	)))

	testCases := []struct {
		name        string
		totalAmount sdk.Int
		getPacket   func() channeltypes.Packet
		ackSuccess  bool
		expAck      channeltypes.Acknowledgement
		expectedRes types.QueryPendingSendToEthResponse
	}{
		{
			name:        "ibc conversion - auto forward to evm chain: have bridge fee",
			totalAmount: sdk.NewInt(110),
			getPacket: func() channeltypes.Packet {
				// Send bsc from Oraichain to OraiBridge in SendPacket method, the denom is extracted by calling DenomPathFromHash()
				transfer := transfertypes.NewFungibleTokenPacketData(myTokenDenom, "110", cosmosAddr, gravityAddr.String())
				// set destination in memo
				transfer.Memo = fmt.Sprintf(`{"send_to_eth":{"evm_chain_prefix":"%s","eth_dest":"%s","amount":"100"}}`, evmChain.EvmChainPrefix, ethDestAddr)

				bz := transfertypes.ModuleCdc.MustMarshalJSON(&transfer)
				return channeltypes.NewPacket(bz, 1, transfertypes.PortID, sourceChannel, transfertypes.PortID, gravityChannel, timeoutHeight, 0)
			},

			ackSuccess: true,
			expAck:     expAck,
			expectedRes: types.QueryPendingSendToEthResponse{
				TransfersInBatches: []types.OutgoingTransferTx{
					{
						Id:          1,
						Sender:      gravityAddr.String(),
						DestAddress: ethDestAddr,
						Erc20Token: types.ERC20Token{
							Contract: tokenContractAddr,
							Amount:   sdk.NewInt(100),
						},
						Erc20Fee: types.ERC20Token{
							Contract: tokenContractAddr,
							Amount:   sdk.NewInt(int64(10)),
						},
					},
				},

				UnbatchedTransfers: []types.OutgoingTransferTx{},
			},
		},
		{
			name:        "ibc conversion - auto forward to evm chain: zero bridge fee",
			totalAmount: sdk.NewInt(100),
			getPacket: func() channeltypes.Packet {
				// Send bsc from Oraichain to OraiBridge in SendPacket method, the denom is extracted by calling DenomPathFromHash()
				transfer := transfertypes.NewFungibleTokenPacketData(myTokenDenom, "100", cosmosAddr, gravityAddr.String())
				// set destination in memo
				transfer.Memo = fmt.Sprintf(`{"send_to_eth":{"evm_chain_prefix":"%s","eth_dest":"%s","amount":"100"}}`, evmChain.EvmChainPrefix, ethDestAddr)

				bz := transfertypes.ModuleCdc.MustMarshalJSON(&transfer)
				return channeltypes.NewPacket(bz, 1, transfertypes.PortID, sourceChannel, transfertypes.PortID, gravityChannel, timeoutHeight, 0)
			},

			ackSuccess: true,
			expAck:     expAck,
			expectedRes: types.QueryPendingSendToEthResponse{
				TransfersInBatches: []types.OutgoingTransferTx{
					{
						Id:          2,
						Sender:      gravityAddr.String(),
						DestAddress: ethDestAddr,
						Erc20Token: types.ERC20Token{
							Contract: tokenContractAddr,
							Amount:   sdk.NewInt(100),
						},
						Erc20Fee: types.ERC20Token{
							Contract: tokenContractAddr,
							Amount:   sdk.NewInt(int64(0)),
						},
					},
				},

				UnbatchedTransfers: []types.OutgoingTransferTx{},
			},
		},
		{
			name:        "ibc conversion - auto forward to evm chain: negative amount",
			totalAmount: sdk.NewInt(100),
			getPacket: func() channeltypes.Packet {
				// Send bsc from Oraichain to OraiBridge in SendPacket method, the denom is extracted by calling DenomPathFromHash()
				transfer := transfertypes.NewFungibleTokenPacketData(myTokenDenom, "100", cosmosAddr, gravityAddr.String())
				// set destination in memo
				transfer.Memo = fmt.Sprintf(`{"send_to_eth":{"evm_chain_prefix":"%s","eth_dest":"%s","amount":"-100"}}`, evmChain.EvmChainPrefix, ethDestAddr)

				bz := transfertypes.ModuleCdc.MustMarshalJSON(&transfer)
				return channeltypes.NewPacket(bz, 1, transfertypes.PortID, sourceChannel, transfertypes.PortID, gravityChannel, timeoutHeight, 0)
			},

			ackSuccess: false,
			expAck:     channeltypes.NewErrorAcknowledgement(sdkerrors.Wrapf(types.ErrBadMetadataFormat, "amount must be positive")),
			expectedRes: types.QueryPendingSendToEthResponse{
				TransfersInBatches: []types.OutgoingTransferTx{},

				UnbatchedTransfers: []types.OutgoingTransferTx{},
			},
		},
	}

	for _, tc := range testCases {

		packet := tc.getPacket()

		// Set Denom Trace
		denomTrace := transfertypes.DenomTrace{
			Path:      path,
			BaseDenom: myTokenDenom,
		}

		input.IbcTransferKeeper.SetDenomTrace(ctx, denomTrace)

		// Set Cosmos Channel
		channel := channeltypes.Channel{
			State:          channeltypes.INIT,
			Ordering:       channeltypes.UNORDERED,
			Counterparty:   channeltypes.NewCounterparty(transfertypes.PortID, sourceChannel),
			ConnectionHops: []string{sourceChannel},
			Version:        "ics20-1",
		}

		input.IbcKeeper.ChannelKeeper.SetChannel(ctx, transfertypes.PortID, gravityChannel, channel)

		// Set Next Sequence Send
		input.IbcKeeper.ChannelKeeper.SetNextSequenceSend(ctx, transfertypes.PortID, gravityChannel, 1)

		// Perform IBC callback, simulate app.OnRecvPacket by sending coin to receiver
		err = input.BankKeeper.SendCoinsFromModuleToAccount(
			input.Context,
			types.ModuleName,
			gravityAddr,
			sdk.NewCoins(sdk.NewCoin(ibcDenom, tc.totalAmount)))
		require.NoError(t, err)
		ack := input.GravityKeeper.OnRecvPacket(ctx, packet, expAck)

		var batch *types.InternalOutgoingTxBatch
		// Check acknowledgement
		require.Equal(t, tc.expAck, ack)

		if tc.ackSuccess {
			require.True(t, ack.Success(), string(ack.Acknowledgement()))
			batch, err = input.GravityKeeper.BuildOutgoingTXBatch(ctx, evmChain.EvmChainPrefix, *tokenAddr, 1)
			require.NoError(t, err)

		} else {
			require.False(t, ack.Success(), string(ack.Acknowledgement()))
			batch, err = input.GravityKeeper.BuildOutgoingTXBatch(ctx, evmChain.EvmChainPrefix, *tokenAddr, 1)
			require.Error(t, err)
		}

		context := sdk.WrapSDKContext(input.Context)
		response, err := input.GravityKeeper.GetPendingSendToEth(context, &types.QueryPendingSendToEth{SenderAddress: gravityAddr.String(), EvmChainPrefix: evmChain.EvmChainPrefix})
		require.NoError(t, err)

		require.Equal(t, tc.expectedRes, *response)

		if batch != nil {
			input.GravityKeeper.DeleteBatch(ctx, evmChain.EvmChainPrefix, *batch)
		}
	}
}