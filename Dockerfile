FROM docker.io/rust:slim

RUN dpkg --add-architecture armel
RUN apt-get update
RUN apt-get install gcc-arm-linux-gnueabihf wget libssl-dev pkg-config make perl git -y
RUN apt-get install libcairo2-dev:armel libpango1.0-dev:armel libjpeg-dev:armel libgif-dev:armel librsvg2-dev:armel -y
RUN apt-get install build-essential libcairo2-dev libpango1.0-dev libjpeg-dev libgif-dev librsvg2-dev -y

RUN adduser --disabled-password --gecos "compilateur" compilateur
USER compilateur

WORKDIR /home/compilateur

RUN git clone https://github.com/abhiTronix/raspberry-pi-cross-compilers.git
RUN wget https://sourceforge.net/projects/raspberry-pi-cross-compilers/files/Raspberry%20Pi%20GCC%20Cross-Compiler%20Toolchains/Bullseye/GCC%2010.3.0/Raspberry%20Pi%201%2C%20Zero/cross-gcc-10.3.0-pi_0-1.tar.gz/download -O cross-gcc-10.3.0-pi_0-1.tar.gz
RUN tar -xf cross-gcc-10.3.0-pi_0-1.tar.gz
RUN rm cross-gcc-10.3.0-pi_0-1.tar.gz

RUN rustup target add arm-unknown-linux-gnueabihf

WORKDIR /home/compilateur/bme280

COPY --chown=compilateur:compilateur . .

# ENV PKG_CONFIG_PATH=""
RUN cargo build --release --target arm-unknown-linux-gnueabihf --example afficher_jour
