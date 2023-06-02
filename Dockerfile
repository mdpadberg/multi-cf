FROM rust:latest AS build

# Download cf cli
RUN curl -L "https://packages.cloudfoundry.org/stable?release=linux64-binary&version=v8&source=github" | tar -zx && \
    mv cf8 /usr/bin && \
    mv cf /usr/bin

# Cache layer that downloads+builds the dependencies
RUN mkdir cache
RUN mkdir -p cache/cli/src
RUN mkdir -p cache/lib/src
COPY Cargo.toml /cache
COPY cli/Cargo.toml /cache/cli
COPY lib/Cargo.toml /cache/lib
RUN \
    echo 'fn main() {}' > /cache/cli/src/main.rs && \
    echo '#![crate_type = "lib"]' > /cache/lib/src/lib.rs && \
    echo 'fn main() {}' > /cache/lib/build.rs && \
    cd /cache && \
    cargo build --release && \
    rm -Rvf /cache/cli/main.rs && \
    rm -Rvf /cache/lib/main.rs

# COPY all the code to cache
COPY cli/ /cache/cli
COPY lib/ /cache/lib
COPY entrypoint.sh /cache
WORKDIR cache

ENTRYPOINT ["./entrypoint.sh"]