use std::fmt::Debug;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use cosmwasm_std::{Addr, Api, Binary, BlockInfo, Empty, IbcMsg, IbcQuery, Querier, Storage};
use cw_multi_test::error::{bail, AnyResult};
use cw_multi_test::{AppResponse, CosmosRouter, Ibc, Module};
use prost::Message;
use titan_cosmos_sdk_proto::ibc::applications::transfer::v1::MsgTransferResponse;

/// Implementation of IBC module
pub type MockIbcModule = MockIbcKeeper<IbcMsg, IbcQuery, Empty>;
impl Ibc for MockIbcModule {}

pub struct MockIbcKeeper<ExecT, QueryT, SudoT> {
    captured_calls: Arc<Mutex<Vec<(Addr, ExecT)>>>,
    _phantom: PhantomData<(ExecT, QueryT, SudoT)>,
}

impl<ExecT, QueryT, SudoT> MockIbcKeeper<ExecT, QueryT, SudoT> {
    /// Creates an instance of an accepting module.
    pub fn new() -> Self {
        Self {
            captured_calls: Arc::new(Mutex::new(Vec::new())),
            _phantom: PhantomData,
        }
    }

    pub fn get_captured_calls(&self) -> Arc<Mutex<Vec<(Addr, ExecT)>>> {
        Arc::clone(&self.captured_calls)
    }
}

impl<ExecT, QueryT, SudoT> Default for MockIbcKeeper<ExecT, QueryT, SudoT> {
    /// Creates an instance of an accepting module with default settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<ExecT, QueryT, SudoT> Module for MockIbcKeeper<ExecT, QueryT, SudoT>
where
    ExecT: Debug + Clone,
    QueryT: Debug,
    SudoT: Debug,
{
    type ExecT = ExecT;
    type QueryT = QueryT;
    type SudoT = SudoT;

    /// Runs any [ExecT](Self::ExecT) message, always returns a default response.
    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        sender: Addr,
        msg: Self::ExecT,
    ) -> AnyResult<AppResponse> {
        self.captured_calls
            .lock()
            .unwrap()
            .push((sender, msg.clone()));

        let mut response = AppResponse::default();
        let msg_transfer_response = MsgTransferResponse { sequence: 1 };
        response.data = Some(msg_transfer_response.encode_to_vec().into());

        Ok(response)
    }

    /// Runs any [QueryT](Self::QueryT) message, always returns a default (empty) binary.
    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: Self::QueryT,
    ) -> AnyResult<Binary> {
        // Ok(Binary::default())
        bail!("Unexpected custom query {:?}", request)
    }

    /// Runs any [SudoT](Self::SudoT) privileged action, always returns a default response.
    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        msg: Self::SudoT,
    ) -> AnyResult<AppResponse> {
        // Ok(AppResponse::default())
        bail!("Unexpected sudo msg {:?}", msg)
    }
}
