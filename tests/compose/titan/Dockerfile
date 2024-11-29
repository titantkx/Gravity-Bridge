FROM ubuntu:22.04

RUN apt update && apt install -y jq curl wget tar

ADD https://github.com/titantkx/titan/releases/download/v3.0.0-rc.0/titan_3.0.0-rc.0_Linux_amd64.tar.gz /titan_3.0.0-rc.0_Linux_amd64.tar.gz
RUN mkdir -p /titan
RUN tar -xvf /titan_3.0.0-rc.0_Linux_amd64.tar.gz -C /titan

ADD setup.sh /setup.sh
ADD start.sh /start.sh
COPY --from=cosmwasm artifacts/cosmwasm.wasm /cosmwasm.wasm
ADD deploy-contract.sh /deploy-contract.sh

RUN /setup.sh

CMD [ "/start.sh" ]
