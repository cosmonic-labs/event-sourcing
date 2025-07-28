# Cosmonic Event Sourcing

> ⚠️ **Experimental**  
> This repository is experimental as a proof of concept.

A pure implementation of event sourcing using Wasm components and persisting state to a filesystem.

This event sourced system uses WIT to describe the interfaces between the different components. At the moment these are:

1. API Gateway: Receives commands somehow and invokes the event sourcer to handle the command
1. Event sourcer: Implementation of the event sourced system, invoking command handlers to handle commands and the event store to append events
1. Business logic command handler: All business domain logic, implemented by handling commands and emitting events
1. Event store: Implements functionality to append events and query the raw event log using a backing store.

The example included in the `bank_account` folder uses the HTTP API gateway as an entrypoint, the event sourcer component, and the filesystem event store for simplicity. At rest, the bank account command handler serializes its events using the protobuf wire format.

## Building

Requirements:

- cargo (Rust toolchain) + wasm32-wasip2 toolchain installed
- protoc

```bash
./build.sh
```

## Running test

Composes and runs a simple test of the bank account command handler.

Requirements:

- cargo (Rust toolchain) + wasm32-wasip2 toolchain installed
- protoc
- wasmtime

```bash
./test.sh
```
