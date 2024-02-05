#!/bin/bash
cd base64-to-u8s
cargo build
cd ..

cd wallet
cargo build
cd ..

cd cosmwasm
docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.13.0
cd ..
