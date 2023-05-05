#!/bin/bash
SCRIPT_DIR="$(cd -P "$(dirname -- "${BASH_SOURCE}")" >/dev/null 2>&1 && pwd)"
docker run -it --rm -p 8088:8080 -v ${SCRIPT_DIR}/wiremock:/home/wiremock wiremock/wiremock --verbose