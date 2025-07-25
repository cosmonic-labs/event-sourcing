#!/bin/bash

./build.sh
cd bank_account
cargo test test_bank_account_basic -- --nocapture 