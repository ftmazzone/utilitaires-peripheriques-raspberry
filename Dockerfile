FROM docker.io/rust:slim

RUN apt-get update
RUN apt-get install gcc-arm-linux-gnueabihf wget libssl-dev pkg-config make perl git -y

RUN adduser --disabled-password --gecos "compilateur" compilateur
USER compilateur

WORKDIR /home/compilateur

RUN git clone https://github.com/abhiTronix/raspberry-pi-cross-compilers.git
RUN wget https://sourceforge.net/projects/raspberry-pi-cross-compilers/files/Raspberry%20Pi%20GCC%20Cross-Compiler%20Toolchains/Bullseye/GCC%2010.3.0/Raspberry%20Pi%201%2C%20Zero/cross-gcc-10.3.0-pi_0-1.tar.gz/download -O cross-gcc-10.3.0-pi_0-1.tar.gz
RUN tar -xf cross-gcc-10.3.0-pi_0-1.tar.gz
RUN rm cross-gcc-10.3.0-pi_0-1.tar.gz

RUN rustup target add arm-unknown-linux-gnueabihf

WORKDIR /home/compilateur/bme280

# Créer le cache des dépendances
COPY --chown=compilateur:compilateur ./.cargo/config .cargo/
COPY --chown=compilateur:compilateur Cargo.toml Cargo.lock ./
COPY --chown=compilateur:compilateur ./afficher_temperature/Cargo.toml ./afficher_temperature/
COPY --chown=compilateur:compilateur ./bme_280/Cargo.toml ./bme_280/
COPY --chown=compilateur:compilateur ./lire_temperature/Cargo.toml ./lire_temperature/
RUN mkdir ./afficher_temperature/src &&  \
    mkdir ./bme_280/src &&\
    mkdir ./lire_temperature/src && mkdir ./lire_temperature/src/bin &&\
    echo "fn main() {}" > ./afficher_temperature/src/main.rs &&\
    echo "fn main() {}" > ./bme_280/src/main.rs &&\
    echo "fn main() {}" > ./lire_temperature/src/bin/lire_ejp.rs &&\
    echo "fn main() {}" > ./lire_temperature/src/bin/lire_temperature.rs
RUN --mount=type=cache,target=$HOME/.cargo \
    --mount=type=cache,target=$HOME/target \
    cargo build --release --target arm-unknown-linux-gnueabihf -p lire_temperature

COPY --chown=compilateur:compilateur . .
RUN cargo build --release --target arm-unknown-linux-gnueabihf -p lire_temperature