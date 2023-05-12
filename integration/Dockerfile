FROM rust:latest

RUN curl -L "https://packages.cloudfoundry.org/stable?release=linux64-binary&version=v8&source=github" | tar -zx && \
    mv cf8 /usr/bin && \
    mv cf /usr/bin

## https://github.com/orgs/community/discussions/26855#discussioncomment-3253651
## The repo is checkout in the /github/workspace folder
WORKDIR "/github/workspace/"

ENTRYPOINT ["./integration/entrypoint.sh"]