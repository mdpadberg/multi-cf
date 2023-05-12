#!/bin/bash
docker build -t mcf-integration . && \
docker run \
  -v /Users/mike/Github/multi-cf/:/github/workspace \
  -v $HOME/.cargo/registry:/usr/local/cargo/registry \
  mcf-integration
