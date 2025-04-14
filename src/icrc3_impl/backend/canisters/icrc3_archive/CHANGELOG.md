# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## Versions

### [unreleased]

### [1.0.0] - 2025-02-18

#### Description
This marks the initial release of the archive canister, which is designed to work alongside the ICRC-3 core canister.
The archive canister **stores historical transactions**, enabling scalability by allowing the ICRC-3 core canister to **spawn new archives** as needed.
This architecture ensures efficient transaction storage and retrieval while maintaining performance.

#### Added
- **Transaction Archiving**: Stores historical transactions offloaded from the core ICRC-3 canister.
- **Efficient Querying**: Provides methods to retrieve stored transactions efficiently.
- **Metadata Management**: Supports fetching archive metadata for monitoring and indexing.
- **Data Integrity**: Ensures consistency between the core canister and its archives.