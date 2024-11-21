package keeper

import (
	"crypto/sha256"
	"fmt"
	"strings"
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
		name                        string
		minChainFeeBasis            uint64
		chainFeeAuctionPoolFraction sdk.Dec
		totalAmount                 sdk.Int
		getPacket                   func() channeltypes.Packet
		ackSuccess                  bool
		expAck                      channeltypes.Acknowledgement
		expectedRes                 types.QueryPendingSendToEthResponse
	}{
		{
			name:                        "ibc conversion - auto forward to evm chain: zero chain fee, zero bridge fee",
			minChainFeeBasis:            0,
			chainFeeAuctionPoolFraction: sdk.NewDec(0),
			totalAmount:                 sdk.NewInt(100),
			getPacket: func() channeltypes.Packet {
				// Send bsc from Oraichain to OraiBridge in SendPacket method, the denom is extracted by calling DenomPathFromHash()
				transfer := transfertypes.NewFungibleTokenPacketData(myTokenDenom, "100", cosmosAddr, gravityAddr.String())
				// set destination in memo
				transfer.Memo = fmt.Sprintf(`{"send_to_eth":{"evm_chain_prefix":"%s","eth_dest":"%s","amount":"100","bridge_fee":"0"}}`, evmChain.EvmChainPrefix, ethDestAddr)

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
							Amount:   sdk.NewInt(int64(0)),
						},
					},
				},

				UnbatchedTransfers: []types.OutgoingTransferTx{},
			},
		},
		{
			name:                        "ibc conversion - auto forward to evm chain: have chain fee, no bridge fee",
			minChainFeeBasis:            0,
			chainFeeAuctionPoolFraction: sdk.NewDec(0),
			totalAmount:                 sdk.NewInt(110),
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
			name:                        "ibc conversion - auto forward to evm chain: have chain fee, have bridge fee",
			minChainFeeBasis:            0,
			chainFeeAuctionPoolFraction: sdk.NewDec(0),
			totalAmount:                 sdk.NewInt(110),
			getPacket: func() channeltypes.Packet {
				// Send bsc from Oraichain to OraiBridge in SendPacket method, the denom is extracted by calling DenomPathFromHash()
				transfer := transfertypes.NewFungibleTokenPacketData(myTokenDenom, "110", cosmosAddr, gravityAddr.String())
				// set destination in memo
				transfer.Memo = fmt.Sprintf(`{"send_to_eth":{"evm_chain_prefix":"%s","eth_dest":"%s","amount":"90","bridge_fee":"10"}}`, evmChain.EvmChainPrefix, ethDestAddr)

				bz := transfertypes.ModuleCdc.MustMarshalJSON(&transfer)
				return channeltypes.NewPacket(bz, 1, transfertypes.PortID, sourceChannel, transfertypes.PortID, gravityChannel, timeoutHeight, 0)
			},

			ackSuccess: true,
			expAck:     expAck,
			expectedRes: types.QueryPendingSendToEthResponse{
				TransfersInBatches: []types.OutgoingTransferTx{
					{
						Id:          3,
						Sender:      gravityAddr.String(),
						DestAddress: ethDestAddr,
						Erc20Token: types.ERC20Token{
							Contract: tokenContractAddr,
							Amount:   sdk.NewInt(90),
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
			name:                        "ibc conversion - auto forward to evm chain: have chain fee, no bridge fee, chain fee less than min",
			minChainFeeBasis:            2000,
			chainFeeAuctionPoolFraction: sdk.NewDec(0),
			totalAmount:                 sdk.NewInt(110),
			getPacket: func() channeltypes.Packet {
				// Send bsc from Oraichain to OraiBridge in SendPacket method, the denom is extracted by calling DenomPathFromHash()
				transfer := transfertypes.NewFungibleTokenPacketData(myTokenDenom, "110", cosmosAddr, gravityAddr.String())
				// set destination in memo
				transfer.Memo = fmt.Sprintf(`{"send_to_eth":{"evm_chain_prefix":"%s","eth_dest":"%s","amount":"100","bridge_fee":"0"}}`, evmChain.EvmChainPrefix, ethDestAddr)

				bz := transfertypes.ModuleCdc.MustMarshalJSON(&transfer)
				return channeltypes.NewPacket(bz, 1, transfertypes.PortID, sourceChannel, transfertypes.PortID, gravityChannel, timeoutHeight, 0)
			},

			ackSuccess: false,
			expAck:     channeltypes.NewErrorAcknowledgement(sdkerrors.ErrInsufficientFee),
			expectedRes: types.QueryPendingSendToEthResponse{
				TransfersInBatches: []types.OutgoingTransferTx{},
				UnbatchedTransfers: []types.OutgoingTransferTx{},
			},
		},
		{
			name:                        "ibc conversion - auto forward to evm chain: have chain fee, have bridge fee, larger than amount",
			minChainFeeBasis:            0,
			chainFeeAuctionPoolFraction: sdk.NewDec(0),
			totalAmount:                 sdk.NewInt(110),
			getPacket: func() channeltypes.Packet {
				// Send bsc from Oraichain to OraiBridge in SendPacket method, the denom is extracted by calling DenomPathFromHash()
				transfer := transfertypes.NewFungibleTokenPacketData(myTokenDenom, "110", cosmosAddr, gravityAddr.String())
				// set destination in memo
				transfer.Memo = fmt.Sprintf(`{"send_to_eth":{"evm_chain_prefix":"%s","eth_dest":"%s","amount":"100","bridge_fee":"101"}}`, evmChain.EvmChainPrefix, ethDestAddr)

				bz := transfertypes.ModuleCdc.MustMarshalJSON(&transfer)
				return channeltypes.NewPacket(bz, 1, transfertypes.PortID, sourceChannel, transfertypes.PortID, gravityChannel, timeoutHeight, 0)
			},

			ackSuccess: false,
			expAck:     channeltypes.NewErrorAcknowledgement(sdkerrors.Wrapf(types.ErrInvalid, "total amount is less than amount to send plus bridge fee")),
			expectedRes: types.QueryPendingSendToEthResponse{
				TransfersInBatches: []types.OutgoingTransferTx{},

				UnbatchedTransfers: []types.OutgoingTransferTx{},
			},
		},
		{
			name:                        "ibc conversion - auto forward to evm chain: negative amount",
			minChainFeeBasis:            0,
			chainFeeAuctionPoolFraction: sdk.NewDec(0),
			totalAmount:                 sdk.NewInt(100),
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
		t.Run(tc.name, func(t *testing.T) {
			packet := tc.getPacket()

			// set chain fee basis
			oldParams := input.GravityKeeper.GetParams(ctx)
			newParams := oldParams
			newParams.MinChainFeeBasisPoints = tc.minChainFeeBasis
			newParams.ChainFeeAuctionPoolFraction = tc.chainFeeAuctionPoolFraction
			input.GravityKeeper.SetParams(ctx, newParams)

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
			fmt.Println(ack)

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

			// revert params
			input.GravityKeeper.SetParams(ctx, oldParams)
		})
	}
}

func TestValidateAndParseMemo(t *testing.T) {
	tests := []struct {
		name              string
		memo              string
		expectedRouted    bool
		expectedDest      string
		expectedAmount    string
		expectedBridgeFee string
		expectedPrefix    string
		expectErr         bool
	}{
		{
			name:              "valid memo",
			memo:              `{"send_to_eth":{"eth_dest":"0x1234567890abcdef1234567890abcdef12345678","amount":"1000","bridge_fee":"10","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "0x1234567890abcdef1234567890abcdef12345678",
			expectedAmount:    "1000",
			expectedBridgeFee: "10",
			expectedPrefix:    "eth",
			expectErr:         false,
		},
		{
			name:              "invalid JSON",
			memo:              `{"send_to_eth":{"eth_dest":"0x1234567890abcdef1234567890abcdef12345678","amount":"1000","bridge_fee":"10","evm_chain_prefix":"eth"`,
			expectedRouted:    false,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         false,
		},
		{
			name:              "missing send_to_eth",
			memo:              `{}`,
			expectedRouted:    false,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         false,
		},
		{
			name:              "missing send_to_eth in another format",
			memo:              `{"another_module":{"another_data":123}}`,
			expectedRouted:    false,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         false,
		},
		{
			name:              "invalid send_to_eth string",
			memo:              `{"send_to_eth":""}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "invalid send_to_eth number",
			memo:              `{"send_to_eth":123}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "invalid eth_dest wrong type",
			memo:              `{"send_to_eth":{"eth_dest":123,"amount":"1000","bridge_fee":"100.23","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "invalid eth dest",
			memo:              `{"send_to_eth":{"eth_dest":"invalid","amount":"1000","bridge_fee":"10","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "missing eth dest",
			memo:              `{"send_to_eth":{"amount":"1000","bridge_fee":"10","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "missing amount",
			memo:              `{"send_to_eth":{"eth_dest":"invalid","bridge_fee":"10","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "invalid amount",
			memo:              `{"send_to_eth":{"eth_dest":"0x1234567890abcdef1234567890abcdef12345678","amount":"invalid","bridge_fee":"10","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "invalid amount have decimal",
			memo:              `{"send_to_eth":{"eth_dest":"0x1234567890abcdef1234567890abcdef12345678","amount":"100.23","bridge_fee":"10","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "missing bridge fee => valid with bridge fee is 0",
			memo:              `{"send_to_eth":{"eth_dest":"0x1234567890abcdef1234567890abcdef12345678","amount":"1000","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "0x1234567890abcdef1234567890abcdef12345678",
			expectedAmount:    "1000",
			expectedBridgeFee: "0",
			expectedPrefix:    "eth",
			expectErr:         false,
		},
		{
			name:              "invalid bridge fee",
			memo:              `{"send_to_eth":{"eth_dest":"0x1234567890abcdef1234567890abcdef12345678","amount":"1000","bridge_fee":"invalid","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
		{
			name:              "invalid bridge fee have decimals",
			memo:              `{"send_to_eth":{"eth_dest":"0x1234567890abcdef1234567890abcdef12345678","amount":"1000","bridge_fee":"100.23","evm_chain_prefix":"eth"}}`,
			expectedRouted:    true,
			expectedDest:      "",
			expectedAmount:    "<nil>",
			expectedBridgeFee: "<nil>",
			expectedPrefix:    "",
			expectErr:         true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			isSendToEthRouted, dest, amount, bridgeFee, evmChainPrefix, err := ValidateAndParseMemo(tt.memo)
			if tt.expectErr {
				fmt.Println(err)
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
			require.Equal(t, tt.expectedRouted, isSendToEthRouted)
			if isSendToEthRouted {
				if dest != nil {
					require.Equal(t, strings.ToLower(tt.expectedDest), strings.ToLower(dest.GetAddress().String()))
				} else {
					require.Nil(t, dest)
				}
				require.Equal(t, tt.expectedAmount, amount.String())
				require.Equal(t, tt.expectedBridgeFee, bridgeFee.String())
				require.Equal(t, tt.expectedPrefix, evmChainPrefix)
			}
		})
	}
}
