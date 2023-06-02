# syntax=docker/dockerfile:1.3-labs

# The above line is so we can use can use heredocs in Dockerfiles. No more && and \!
# https://www.docker.com/blog/introduction-to-heredocs-in-dockerfiles/

FROM rust:latest AS build

RUN mkdir app

# We create a new lib and then use our own Cargo.toml
RUN cargo new --lib /app/lib
COPY lib/Cargo.toml /app/lib/
COPY lib/build.rs /app/lib/build.rs

# We do the same for our mcf cli
RUN cargo new /app/cli
COPY cli/Cargo.toml /app/cli/

# This step compiles only our dependencies and saves them in a layer. This is the most impactful time savings
# Note the use of --mount=type=cache. On subsequent runs, we'll have the crates already downloaded
WORKDIR /app/cli
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build

# Copy our sources
COPY ./cli /app/cli
COPY ./lib /app/lib
COPY ./integration /app/integration

# A bit of magic here!
# * We're mounting that cache again to use during the build, otherwise it's not present and we'll have to download those again - bad!
# * EOF syntax is neat but not without its drawbacks. We need to `set -e`, otherwise a failing command is going to continue on
# * Rust here is a bit fiddly, so we'll touch the files (even though we copied over them) to force a new build
RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  # update timestamps to force a new build
  touch /app/lib/src/lib.rs /app/cli/src/main.rs
  cargo build
EOF

FROM rust:latest AS runner

# Download cf cli
RUN curl -L "https://packages.cloudfoundry.org/stable?release=linux64-binary&version=v8&source=github" | tar -zx && \
    mv cf8 /usr/bin && \
    mv cf /usr/bin

# Copy and build integration  
COPY --from=build /app/integration /integration
COPY ./entrypoint.sh /integration/entrypoint.sh
WORKDIR integration

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build

# Copy mcf binary into integration
COPY --from=build /app/cli/target/debug/mcf /integration/target/debug/mcf

ENTRYPOINT ["./entrypoint.sh"]