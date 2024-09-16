FROM fedora:38 AS base
ENV GOPATH=/go
ENV PATH=$PATH:/go/bin

RUN dnf install -y git make gcc gcc-c++ which iproute iputils procps-ng vim-minimal tmux net-tools htop tar jq npm openssl-devel perl rust cargo golang wget
# RUN cargo install sccache --locked
RUN cargo install cargo-chef --locked
# ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

# only required for deployment script
RUN npm install -g ts-node && npm install -g typescript
ADD https://gethstore.blob.core.windows.net/builds/geth-linux-amd64-1.10.10-bb74230f.tar.gz /geth/
RUN cd /geth && tar -xvf * && mv /geth/**/geth /usr/bin/geth
# Download the althea gaia fork as a IBC test chain
ADD https://github.com/althea-net/ibc-test-chain/releases/download/v9.1.5/gaiad-v9.1.5-linux-amd64 /usr/bin/gaiad
# Setup Hermes for IBC connections between chains
ADD https://github.com/informalsystems/ibc-rs/releases/download/v1.7.0/hermes-v1.7.0-x86_64-unknown-linux-gnu.tar.gz /tmp/
RUN cd /tmp/ && tar -xvf hermes-v1.7.0-x86_64-unknown-linux-gnu.tar.gz && mv hermes /usr/bin/
RUN mkdir /ibc-relayer-logs && touch /ibc-relayer-logs/hermes-logs && touch /ibc-relayer-logs/channel-creation

#
#
#

FROM base AS planner
ADD gravity.tar.gz /
# --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  pushd /gravity/orchestrator && cargo chef prepare --recipe-path /recipe.json --bin test_runner

#
#
#

FROM base AS builder

RUN mkdir -p /gravity/orchestrator
COPY --from=planner /recipe.json /gravity/orchestrator/recipe.json
# --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  pushd /gravity/orchestrator && cargo chef cook --release --recipe-path recipe.json --bin test-runner

# the actual source code for this repo, this **only** includes checked in files!
# this is a bit of a pain but it does speed things up a lot
ADD gravity.tar.gz /

RUN pushd /gravity/module/ && PATH=$PATH:/usr/local/go/bin GOPROXY=https://proxy.golang.org make && PATH=$PATH:/usr/local/go/bin make install

RUN pushd /gravity/solidity/ && HUSKY_SKIP_INSTALL=1 npm install && npm run typechain

# build the test runner specifically to cache a release artifact
# --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  pushd /gravity/orchestrator/test_runner && PATH=$PATH:$HOME/.cargo/bin cargo build --bin test-runner --release

RUN git config --global --add safe.directory /gravity
