#!/bin/bash

cargo build --target wasm32-wasip2 --release -p http-api-gateway
cargo build --target wasm32-wasip2 --release -p bank-account-command-handler
cargo build --target wasm32-wasip2 --release -p event-sourcer
cargo build --target wasm32-wasip2 --release -p filesystem-event-store