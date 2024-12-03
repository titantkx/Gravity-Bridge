FROM fedora:38 AS base
ENV GOPATH=/go
ENV PATH=$PATH:/go/bin

RUN dnf install -y git make gcc gcc-c++ which iproute iputils procps-ng vim-minimal tmux net-tools htop tar jq npm openssl-devel perl rust cargo golang wget

RUN cargo install cargo-chef --locked

# only required for deployment script
RUN npm install -g ts-node && npm install -g typescript

# Setup Hermes for IBC connections between chains
ADD https://github.com/informalsystems/ibc-rs/releases/download/v1.7.0/hermes-v1.7.0-x86_64-unknown-linux-gnu.tar.gz /tmp/
RUN cd /tmp/ && tar -xvf hermes-v1.7.0-x86_64-unknown-linux-gnu.tar.gz && mv hermes /usr/bin/
RUN mkdir /ibc-relayer-logs && touch /ibc-relayer-logs/hermes-logs && touch /ibc-relayer-logs/channel-creation

#
#
#

FROM base AS planner
COPY --from=repo orchestrator /gravity/orchestrator
COPY --from=repo cosmwasm /gravity/cosmwasm
RUN pushd /gravity/orchestrator && cargo chef prepare --recipe-path /recipe.json --bin test_runner

#
#
#

FROM base AS builder

RUN mkdir -p /gravity/orchestrator
COPY --from=planner /recipe.json /gravity/orchestrator/recipe.json
COPY --from=repo cosmwasm /gravity/cosmwasm
RUN pushd /gravity/orchestrator && cargo chef cook --release --recipe-path recipe.json --bin test-runner

COPY --from=repo solidity /gravity/solidity
RUN rm -rf /gravity/solidity/.openzeppelin
RUN pushd /gravity/solidity/ && HUSKY_SKIP_INSTALL=1 npm install
RUN pushd /gravity/solidity/ && npm run compile

ADD set-env.sh /set-env.sh
ADD setup.sh /setup.sh
ADD run-test.sh /run-test.sh

ENTRYPOINT [ "/bin/bash" ]

# CMD for sleeping keeping the container running
CMD [ "-c", "while true; do sleep 30; done;" ]
