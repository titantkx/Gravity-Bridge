package keeper

import (
	"github.com/Gravity-Bridge/Gravity-Bridge/module/x/gravity/types"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// SetEvmChainData sets the EVM chain specific data
// Check if chains exists before calling this method
func (k Keeper) SetEvmChainData(ctx sdk.Context, evmChain types.EvmChain) {
	key := types.GetEvmChainKey(evmChain.EvmChainPrefix)
	ctx.KVStore(k.storeKey).Set(key, k.cdc.MustMarshal(&evmChain))
}

// GetEvmChainData returns data for the specific EVM chain
func (k Keeper) GetEvmChainData(ctx sdk.Context, evmChainPrefix string) *types.EvmChain {
	key := types.GetEvmChainKey(evmChainPrefix)
	store := ctx.KVStore(k.storeKey)

	bytes := store.Get(key)
	if bytes == nil {
		return nil
	}

	var evmChainData types.EvmChain
	k.cdc.MustUnmarshal(bytes, &evmChainData)
	return &evmChainData
}

func (k Keeper) GetEvmChains(ctx sdk.Context) []types.EvmChain {
	store := ctx.KVStore(k.storeKey)
	prefix := types.EvmChainKey
	iter := store.Iterator(prefixRange(prefix))
	defer iter.Close()

	var evmChains []types.EvmChain

	for ; iter.Valid(); iter.Next() {
		value := iter.Value()
		var evmChainData types.EvmChain
		k.cdc.MustUnmarshal(value, &evmChainData)

		evmChains = append(evmChains, evmChainData)
	}

	return evmChains
}

func (k Keeper) IterateEvmChains(ctx sdk.Context, cb func(key []byte, evmChain *types.EvmChain) (stop bool)) {
	store := ctx.KVStore(k.storeKey)
	prefix := types.EvmChainKey
	iter := store.Iterator(prefixRange(prefix))
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		evmChain := new(types.EvmChain)
		value := iter.Value()
		k.cdc.MustUnmarshal(value, evmChain)
		if cb(iter.Key(), evmChain) {
			break
		}
	}
}

func (k Keeper) GetEvmChainsWithLimit(ctx sdk.Context, limit uint64) []types.EvmChain {
	evmChains := []types.EvmChain{}

	k.IterateEvmChains(ctx, func(key []byte, evmChain *types.EvmChain) (stop bool) {
		evmChains = append(evmChains, *evmChain)
		if limit != 0 && uint64(len(evmChains)) >= limit {
			return true
		}
		return false
	})

	return evmChains
}

// RemoveEvmChainFromStore performs in-place store migrations to remove an evm chain
func (k Keeper) RemoveEvmChainFromStore(ctx sdk.Context, evmChainPrefix string) error {
	ctx.Logger().Info("Removing EVM chain from store", "evmChainPrefix", evmChainPrefix)
	store := ctx.KVStore(k.storeKey)

	// delete evmChain data
	store.Delete(types.GetEvmChainKey(evmChainPrefix))

	// single key with chain
	k.removeKeyPrefixFromEvm(ctx, types.KeyLastOutgoingBatchID, evmChainPrefix)
	k.removeKeyPrefixFromEvm(ctx, types.LastObservedEventNonceKey, evmChainPrefix)
	k.removeKeyPrefixFromEvm(ctx, types.LastObservedEthereumBlockHeightKey, evmChainPrefix)
	k.removeKeyPrefixFromEvm(ctx, types.KeyLastTXPoolID, evmChainPrefix)
	k.removeKeyPrefixFromEvm(ctx, types.LastSlashedValsetNonce, evmChainPrefix)
	k.removeKeyPrefixFromEvm(ctx, types.LatestValsetNonce, evmChainPrefix)
	k.removeKeyPrefixFromEvm(ctx, types.LastSlashedBatchBlock, evmChainPrefix)
	k.removeKeyPrefixFromEvm(ctx, types.LastSlashedLogicCallBlock, evmChainPrefix)
	k.removeKeyPrefixFromEvm(ctx, types.LastObservedValsetKey, evmChainPrefix)

	// multi key with chain
	k.removeKeysPrefixFromEvm(ctx, types.ValsetRequestKey, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.ValsetConfirmKey, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.OracleAttestationKey, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.OutgoingTXPoolKey, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.OutgoingTXBatchKey, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.BatchConfirmKey, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.LastEventNonceByValidatorKey, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.KeyOutgoingLogicCall, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.KeyOutgoingLogicConfirm, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.DenomToERC20Key, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.ERC20ToDenomKey, evmChainPrefix)
	k.removeKeysPrefixFromEvm(ctx, types.PastEthSignatureCheckpointKey, evmChainPrefix)
	// PendingIbcAutoForwards is only existed in v3
	k.removeKeysPrefixFromEvm(ctx, types.PendingIbcAutoForwards, evmChainPrefix)

	return nil
}

func (k Keeper) removeKeyPrefixFromEvm(ctx sdk.Context, key []byte, evmChainPrefix string) {
	store := ctx.KVStore(k.storeKey)
	store.Delete(types.AppendChainPrefix(key, evmChainPrefix))
}

func (k Keeper) removeKeysPrefixFromEvm(ctx sdk.Context, key []byte, evmChainPrefix string) {
	store := ctx.KVStore(k.storeKey)
	keyPrefix := types.AppendChainPrefix(key, evmChainPrefix)
	prefixStore := prefix.NewStore(store, keyPrefix)
	storeIter := prefixStore.Iterator(nil, nil)
	defer storeIter.Close()

	for ; storeIter.Valid(); storeIter.Next() {
		prefixStore.Delete(storeIter.Key())
	}
}
