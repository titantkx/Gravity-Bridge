/// AuctionPeriod represents a period of auctions.
/// An AuctionPeriod applies to as many auctionable tokens exist in the auction pool
/// Note: see params for a list of non-auctionable tokens
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuctionPeriod {
    /// Block height at which the AuctionPeriod starts.
    #[prost(uint64, tag = "1")]
    pub start_block_height: u64,
    /// Block height at which the AuctionPeriod end.
    #[prost(uint64, tag = "2")]
    pub end_block_height: u64,
}
/// Auction represents a single auction.
/// An Auction has a unique identifier relative to its Auction Period Id , an amount being auctioned, a status, and a highest bid.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Auction {
    /// Unique identifier for the Auction.
    #[prost(uint64, tag = "1")]
    pub id: u64,
    /// Amount being auctioned.
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
    /// Highest bid on the Auction.
    #[prost(message, optional, tag = "3")]
    pub highest_bid: ::core::option::Option<Bid>,
}
/// Bid represents a bid on an Auction.
/// A Bid includes the identifier of the Auction, the amount of the bid, and the address of the bidder.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bid {
    /// Amount of the bid.
    #[prost(uint64, tag = "1")]
    pub bid_amount: u64,
    /// Address of the bidder.
    #[prost(string, tag = "2")]
    pub bidder_address: ::prost::alloc::string::String,
}
/// AuctionId enables easy storage and retrieval of the most recently used AuctionId
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuctionId {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
/// Params defines the parameters for the GravityBridge auction module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// AuctionLength is the number of blocks that the AuctionPeriod will be active
    #[prost(uint64, tag = "1")]
    pub auction_length: u64,
    /// MinBidFee defines the required minimum fee that must be paid by bidders for their bid to be considered by the module.
    /// This fee is paid out to the auction pool. This fee is separate from the standard Cosmos Tx spam protection fee.
    /// This fee will not be charged unless a bid is successful.
    #[prost(uint64, tag = "2")]
    pub min_bid_fee: u64,
    /// NonAuctionableTokens is a list of token denomss which should never be auctioned from the auction pool
    #[prost(string, repeated, tag = "3")]
    pub non_auctionable_tokens: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// BurnWinningBids controls if we burn the tokens of the winning bidder instead of sending them to the auction pool
    #[prost(bool, tag = "4")]
    pub burn_winning_bids: bool,
    /// Enabled controls whether auctions progress as usual, or are preserved in an inactive halted state.
    /// When Enabled is false, bids will also fail to be processed.
    #[prost(bool, tag = "5")]
    pub enabled: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionPeriodRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionPeriodResponse {
    #[prost(message, optional, tag = "1")]
    pub auction_period: ::core::option::Option<AuctionPeriod>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionsRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub auctions: ::prost::alloc::vec::Vec<Auction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionByIdRequest {
    #[prost(uint64, tag = "1")]
    pub auction_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionByIdResponse {
    #[prost(message, optional, tag = "1")]
    pub auction: ::core::option::Option<Auction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionByDenomRequest {
    #[prost(string, tag = "1")]
    pub auction_denom: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionByDenomResponse {
    #[prost(message, optional, tag = "1")]
    pub auction: ::core::option::Option<Auction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllAuctionsByBidderRequest {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllAuctionsByBidderResponse {
    #[prost(message, repeated, tag = "1")]
    pub auctions: ::prost::alloc::vec::Vec<Auction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionPoolRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuctionPoolResponse {
    #[prost(string, tag = "1")]
    pub account: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub balances: ::prost::alloc::vec::Vec<cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
}
/// Generated client implementations.
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query defines the gRPC querier service.
    #[derive(Debug, Clone)]
    pub struct QueryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            QueryClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Params returns the current module parameters (decided by governance)
        pub async fn params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsRequest>,
        ) -> Result<tonic::Response<super::QueryParamsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auction.v1.Query/Params");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// AuctionPeriod returns the current active auction period, or a future one if no period is active
        pub async fn auction_period(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAuctionPeriodRequest>,
        ) -> Result<tonic::Response<super::QueryAuctionPeriodResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/auction.v1.Query/AuctionPeriod",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Auctions returns all current active auctions
        pub async fn auctions(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAuctionsRequest>,
        ) -> Result<tonic::Response<super::QueryAuctionsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/auction.v1.Query/Auctions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// AuctionById returns an open auction given by its `id`
        pub async fn auction_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAuctionByIdRequest>,
        ) -> Result<tonic::Response<super::QueryAuctionByIdResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/auction.v1.Query/AuctionById",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// AuctionByDenom returns an open auction given by its `denom`
        pub async fn auction_by_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAuctionByDenomRequest>,
        ) -> Result<tonic::Response<super::QueryAuctionByDenomResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/auction.v1.Query/AuctionByDenom",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// AllAuctionsByBidder returns all open auctions with the given highest bidder address
        pub async fn all_auctions_by_bidder(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAllAuctionsByBidderRequest>,
        ) -> Result<
            tonic::Response<super::QueryAllAuctionsByBidderResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/auction.v1.Query/AllAuctionsByBidder",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// AuctionPool returns the auction pool account address and the tokens which will be up for auction next,
        /// (but does not return any amounts from auctions with no bidder)
        pub async fn auction_pool(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAuctionPoolRequest>,
        ) -> Result<tonic::Response<super::QueryAuctionPoolResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/auction.v1.Query/AuctionPool",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// GenesisState defines the auction module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    #[prost(message, optional, tag = "2")]
    pub active_period: ::core::option::Option<AuctionPeriod>,
    #[prost(message, repeated, tag = "3")]
    pub active_auctions: ::prost::alloc::vec::Vec<Auction>,
}
/// MsgBid is a message type for placing a bid on an auction with given `auction_id`
/// `bidder` is the signer of the Msg
/// `amount` is the native token amount locked by the auction module if the bid is accepted as the highest bid
/// `bid_fee` is the native token amount sent to the auction pool, and should be at least equal to the min bid fee param
///
/// Additionally, all bids must meet or exceed `min_bid_amount`
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBid {
    /// ID of the auction to bid on
    #[prost(uint64, tag = "1")]
    pub auction_id: u64,
    /// Address of the bidder
    #[prost(string, tag = "2")]
    pub bidder: ::prost::alloc::string::String,
    /// Amount of the bid
    #[prost(uint64, tag = "3")]
    pub amount: u64,
    /// Fee amount
    #[prost(uint64, tag = "4")]
    pub bid_fee: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBidResponse {}
/// Generated client implementations.
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the state transitions possible within auction
    #[derive(Debug, Clone)]
    pub struct MsgClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MsgClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MsgClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MsgClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            MsgClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn bid(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgBid>,
        ) -> Result<tonic::Response<super::MsgBidResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auction.v1.Msg/Bid");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
