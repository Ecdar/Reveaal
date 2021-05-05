FROM debian:bullseye-slim

# Install dependencies and tools
RUN apt-get update && \
	apt-get install -y automake libxml2-dev gperf build-essential libboost-all-dev git curl libclang-11-dev llvm-11

# Compile UDBM
RUN git clone --depth 1 https://github.com/UPPAALModelChecker/UDBM.git /usr/src/udbm
WORKDIR /usr/src/udbm

RUN ./autogen.sh && ./configure
RUN make
RUN ./scripts/mergelibs.sh

RUN curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal

# Compile Reveaal
RUN mkdir /usr/src/reveaal
WORKDIR /usr/src/reveaaal
ADD . ./
RUN mkdir ./dbm/objectFiles && \
    cp /usr/src/udbm/dbm/*.o ./dbm/objectFiles/ && \
    cp /usr/src/udbm/base/*.o ./dbm/objectFiles/ && \
    cp /usr/src/udbm/debug/*.o ./dbm/objectFiles/ && \
    cp /usr/src/udbm/hash/*.o ./dbm/objectFiles/ && \
    cp /usr/src/udbm/io/*.o ./dbm/objectFiles/
ENV LLVM_CONFIG_PATH=/usr/lib/llvm-11/bin/llvm-config
RUN cd dbm && ./recompile.sh
RUN /root/.cargo/bin/cargo build --release

ENTRYPOINT ./target/release/Reveaal
