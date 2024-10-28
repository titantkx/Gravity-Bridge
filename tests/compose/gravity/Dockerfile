FROM fedora:38 AS base
ENV GOPATH=/go
ENV PATH=$PATH:/go/bin

RUN dnf install -y git make gcc gcc-c++ which iproute iputils procps-ng vim-minimal tmux net-tools htop tar jq npm openssl-devel perl rust cargo golang wget

RUN cargo install cargo-chef --locked

# Setup Hermes for IBC connections between chains
ADD https://github.com/informalsystems/ibc-rs/releases/download/v1.7.0/hermes-v1.7.0-x86_64-unknown-linux-gnu.tar.gz /tmp/
RUN cd /tmp/ && tar -xvf hermes-v1.7.0-x86_64-unknown-linux-gnu.tar.gz && mv hermes /usr/bin/
RUN mkdir /ibc-relayer-logs && touch /ibc-relayer-logs/hermes-logs && touch /ibc-relayer-logs/channel-creation

#
#
#

FROM base AS planner
COPY --from=repo orchestrator /gravity/orchestrator

RUN pushd /gravity/orchestrator && cargo chef prepare --recipe-path /recipe.json --bin test_runner

#
#
#

FROM base AS builder

RUN mkdir -p /gravity/orchestrator
COPY --from=planner /recipe.json /gravity/orchestrator/recipe.json

RUN pushd /gravity/orchestrator && cargo chef cook --release --recipe-path recipe.json --bin test-runner

COPY --from=repo solidity /gravity/solidity
RUN pushd /gravity/solidity/ && HUSKY_SKIP_INSTALL=1 npm install && npm run typechain

COPY --from=repo module /gravity/module
RUN pushd /gravity/module/ && PATH=$PATH:/usr/local/go/bin GOPROXY=https://proxy.golang.org make && PATH=$PATH:/usr/local/go/bin make install

COPY --from=repo orchestrator /gravity/orchestrator
# build the test runner specifically to cache a release artifact

RUN pushd /gravity/orchestrator/test_runner && PATH=$PATH:$HOME/.cargo/bin cargo build --bin test-runner --release

# the actual source code for this repo, this **only** includes checked in files!
# this is a bit of a pain but it does speed things up a lot
ADD gravity.tar.gz /
RUN git config --global --add safe.directory /gravity
