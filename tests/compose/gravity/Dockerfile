FROM fedora:38
ENV GOPATH=/go
ENV PATH=$PATH:/go/bin

RUN dnf install -y git make gcc gcc-c++ which iproute iputils procps-ng vim-minimal tmux net-tools htop tar jq npm openssl-devel perl golang wget

COPY --from=repo module /gravity/module
RUN pushd /gravity/module/ && PATH=$PATH:/usr/local/go/bin GOPROXY=https://proxy.golang.org make && PATH=$PATH:/usr/local/go/bin make install

ADD setup.sh /setup.sh
ADD start.sh /start.sh

ENV NODES=1

RUN /setup.sh $NODES

CMD exec /start.sh ${NODES}
