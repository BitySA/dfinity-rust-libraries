# ICRC-3 Rust Implementation

This repository contains a Rust implementation of the ICRC-3 standard for the Internet Computer (ICP). ICRC-3 defines a generic value type for interoperable token transactions and ledger interactions.

## Project Structure

### Backend canisters

All the backend canisters are included in the folder [`backend/canisters`](backend/canisters/). This includes

- [`icrc3`](backend/canisters/icrc3/): The core logic for the archive creation, blocks archiving.
- [`icrc3_archive`](backend/canisters/icrc3_archive/): The archive canister that stores archived blocks.

## Local development instructions

### Clone this repository

```sh
git clone "ADD LINK HERE"
```

(Or from the Origyn internal Gitlab url, from which this Github repo is automatically mirrored)

### Install the dependencies

First, ensure that you have [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html) and [`ic-wasm`](https://github.com/dfinity/ic-wasm) installed, as well as [`wasmtime`](https://wasmtime.dev).  
Then from the repository's root folder:

```sh
npm install
```

## Technical documentation

- Developers documentation still :construction: WIP (See code comments for now. Documentation will be automatically generated and published at a later time)

## DevOps documentation

- :construction: WIP on Origyn's internal Gitlab wiki