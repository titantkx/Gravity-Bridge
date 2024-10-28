FROM fedora:38

RUN dnf install -y which iproute iputils procps-ng tmux net-tools htop tar jq npm openssl-devel perl wget

# only required for deployment script
RUN npm install -g ts-node && npm install -g typescript
ADD https://gethstore.blob.core.windows.net/builds/geth-linux-amd64-1.10.10-bb74230f.tar.gz /geth/
RUN cd /geth && tar -xvf * && mv /geth/**/geth /usr/bin/geth

ADD run-eth.sh /run-eth.sh
ADD start.sh /start.sh
COPY --from=repo tests/assets/ETHGenesis.json /ETHGenesis.json

CMD ["/start.sh"]
