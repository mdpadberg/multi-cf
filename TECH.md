# Tech related info for Multi-cf (mcf)

## What software do you need for this project
- The rust language ([rustup](https://rustup.rs/) is an easy tool to install rust and its tooling)
- Docker
- Docker-compose

## How to build the software
You can run this command in the root folder:  
`cargo build --release`  

## How to test this project
You can run this command in the root folder:  
`docker-compose up --exit-code-from mcf --build`   