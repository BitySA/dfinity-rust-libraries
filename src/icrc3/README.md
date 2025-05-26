# ICRC3 - Transaction Management Library for Internet Computer

## What is ICRC3?

ICRC3 is a transaction standard for the Internet Computer that provides a standardized interface for storing, archiving, and retrieving transactions on the IC (Internet Computer) blockchain. It enables consistent management of transaction logs across different types of tokens and applications, while automatically handling concerns like archiving and certification.

The standard creates a unified way to:
- Record various transaction types
- Store transaction history efficiently
- Query transaction history with standardized endpoints
- Archive older transactions to separate canisters when needed
- Certify transaction blocks for verification

## Why use ICRC3?

- **Standardization**: ICRC3 offers a consistent approach to storing and retrieving transaction history on the Internet Computer.
- **Interoperability**: Facilitates interaction between different applications and tokens on the IC.
- **Efficient Archiving**: Automatically manages the archiving of older transactions to optimize memory usage.
- **Certification**: Supports certification of transaction blocks for verification.
- **Flexibility**: Adapts to different types of transactions (transfers, mints, burns, etc.).
- **Scalability**: Handles growing transaction history with automatic archiving.
- **Auditability**: Provides a complete and immutable record of all operations.

## Integration with other ICRC standards

ICRC3 works perfectly with other ICRC standards such as ICRC1 (fungible token transfers) and ICRC7 (NFTs). It can be used to enhance any canister that needs to maintain an auditable history of transactions or state changes.

Examples of integration:
- **ICRC1**: Record token transfers, mints, and burns
- **ICRC2**: Track token approvals and allowance changes for fungible tokens
- **ICRC7**: Track NFT minting, burning, transfers, and metadata updates
- **ICRC37**: Log approval operations for NFTs
- **Custom Ledgers**: Maintain a standardized transaction history for any custom token implementation

## How to implement ICRC3 in your project

### 1. Initial setup

Add the necessary dependencies to your `Cargo.toml`:

```toml
[dependencies]
bity_ic_icrc3 = "0.1.0"  # Make sure to use the latest version
bity_ic_icrc3_macros = "0.1.0"
bity_ic_types = "0.1.0"
icrc_ledger_types = "0.1.0"
```

### 2. Define the transaction type

Create a custom transaction type that implements the `TransactionType` trait:

```rust
use bity_ic_icrc3::transaction::{Hash, TransactionType};
use bity_ic_types::TimestampSeconds;
use candid::{CandidType, Nat};
use icrc_ledger_types::icrc1::account::Account;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use serde_bytes::ByteBuf;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MyTransactionType {
    pub btype: String,  // Transaction type (e.g., "transfer", "mint", "burn")
    pub timestamp: u64,
    pub data: TransactionData,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TransactionData {
    pub id: Nat,  // Transaction or asset ID
    pub from: Option<Account>,  // Sender (if applicable)
    pub to: Option<Account>,    // Recipient (if applicable)
    pub meta: Option<Icrc3Value>,  // Additional metadata
    pub memo: Option<ByteBuf>,  // Optional memo field
    pub created_at_time: Option<Nat>,  // Transaction timestamp
}

impl TransactionType for MyTransactionType {
    fn validate_transaction_fields(&self) -> Result<(), String> {
        // Type-specific validation based on transaction type
        match self.btype.as_str() {
            "mint" => {
                if self.data.from.is_some() {
                    return Err("From is not allowed for mint".to_string());
                }
                if self.data.to.is_none() {
                    return Err("To is required for mint".to_string());
                }
            },
            "burn" => {
                if self.data.from.is_none() {
                    return Err("From is required for burn".to_string());
                }
                if self.data.to.is_some() {
                    return Err("To is not allowed for burn".to_string());
                }
            },
            // Add more transaction types as needed
            _ => return Err(format!("Unknown transaction type: {}", self.btype)),
        }
        Ok(())
    }

    fn timestamp(&self) -> Option<TimestampSeconds> {
        Some(self.timestamp)
    }

    fn hash(&self) -> Hash {
        // Create a hash of the transaction data for uniqueness
        let mut hasher = Sha256::new();
        hasher.update(self.btype.as_bytes());
        hasher.update(self.timestamp.to_le_bytes().as_slice());
        hasher.update(self.data.id.0.to_bytes_le());
        
        if let Some(from) = &self.data.from {
            hasher.update(from.owner.as_slice());
        }
        if let Some(to) = &self.data.to {
            hasher.update(to.owner.as_slice());
        }
        // Include other fields in hash calculation
        
        hasher.finalize().into()
    }

    fn block_type(&self) -> String {
        self.btype.clone()
    }
}
```

### 3. Configure ICRC3 state in your canister

Use the `icrc3_state!()` macro to add ICRC3 state to your canister:

```rust
use bity_ic_icrc3_macros::icrc3_state;

// Add this to your main module to set up ICRC3 state
icrc3_state!();
```

The macro adds necessary ICRC3 state to your canister, which manages:
- Transaction blocks and archives
- Archiving logic
- Indexing for efficient querying

### 4. Initialize the ICRC3 system

In your canister initialization function:

```rust
use bity_ic_icrc3::config::{ICRC3Config, ICRC3Properties};
use icrc_ledger_types::icrc3::blocks::SupportedBlockType;
use std::time::Duration;

fn init() {
    // ICRC3 Configuration - customize for your specific use case
    let icrc3_config = ICRC3Config {
        supported_blocks: vec![
            SupportedBlockType {
                block_type: "mint".to_string(),
                url: "https://github.com/your-org/your-project/docs/mint-schema".to_string(),
            },
            SupportedBlockType {
                block_type: "burn".to_string(),
                url: "https://github.com/your-org/your-project/docs/burn-schema".to_string(),
            },
            SupportedBlockType {
                block_type: "transfer".to_string(),
                url: "https://github.com/your-org/your-project/docs/transfer-schema".to_string(),
            },
            // Define all transaction types your system supports
        ],
        constants: ICRC3Properties::new(
            Duration::from_secs(24 * 60 * 60),  // Transaction window: how long transactions are valid
            10,  // Max retries for archiving operations
            6 * 1024 * 1024,  // Max segment size (6MB)
            2 * 1024 * 1024,  // Max archive size (2MB)
            2_000_000_000_000_000_000,  // Cycles to create an archive canister
            2_000_000_000_000_000_000,  // Cycles to store a segment in archive
            25  // Archive threshold - when to trigger archiving
        ),
    };
    
    // Initialize ICRC3 with your configuration
    init_icrc3(icrc3_config);
    start_default_archive_job();
}
```

### 5. Managing canister upgrades

In the upgrade functions:

```rust
// Pre-upgrade handler
#[pre_upgrade]
fn pre_upgrade() {
    // Get the current ICRC3 state
    let icrc3 = take_icrc3();
    
    // Serialize ICRC3 state along with your main state
    let stable_state = (my_runtime_state, logs, traces, icrc3);
    
    // Write logic to serialize and store data on stable memory during upgrade
    // Here's an example using bity's library
    let mut memory = get_upgrades_memory();
    let writer = get_writer(&mut memory);
    bity_ic_serializer::serialize(stable_state, writer).unwrap();
}

// Post-upgrade handler
#[post_upgrade]
fn post_upgrade() {
    // Write logic to Deserialize and get data from stable memory after upgrade
    // Here's an example using bity's library
    let memory = get_upgrades_memory();
    let reader = get_reader(&memory);
    let (mut state, logs, traces, icrc3) = bity_ic_serializer::deserialize(reader).unwrap();
    
    // Restore ICRC3 state
    replace_icrc3(icrc3);
    start_default_archive_job();

    
    // Continue with other initialization steps
    // ...
}
```

### 6. Expose ICRC3 endpoints

Create query endpoints to expose standard ICRC3 interfaces:

```rust
#[query]
async fn icrc3_get_archives(_: GetArchivesArg) -> GetArchivesResponse {
    icrc3_get_archives_impl::<MyTransactionType>()
}

#[query]
async fn icrc3_get_blocks(args: GetBlocksArg) -> GetBlocksResult {
    icrc3_get_blocks_impl::<MyTransactionType>(args).await
}

#[query]
async fn icrc3_get_properties(_: GetArchivePropsArg) -> GetArchivePropsResponse {
    icrc3_get_properties_impl::<MyTransactionType>()
}

#[query]
async fn icrc3_get_tip_certificate(_: GetTipCertificateArg) -> GetTipCertificateResponse {
    icrc3_get_tip_certificate_impl::<MyTransactionType>()
}

#[query]
async fn icrc3_supported_block_types(_: GetSupportedBlockTypesArg) -> GetSupportedBlockTypesResponse {
    icrc3_supported_block_types_impl::<MyTransactionType>()
}
```

### 7. Adding transactions

To record transactions in the ICRC3 log, you'll typically follow this pattern:

```rust
use bity_ic_icrc3::transaction::add_transaction;

// In your update method that performs operations:
#[update]
async fn perform_operation(args: OperationArgs) -> Result<OperationResponse, Error> {
    // Perform your business logic
    // ...
    
    // Create a transaction record
    let transaction = MyTransactionType {
        btype: "some_operation".to_string(),
        timestamp: ic_cdk::api::time(),
        data: TransactionData {
            id: operation_id,
            from: Some(from_account),
            to: Some(to_account),
            meta: Some(metadata),
            memo: args.memo.clone(),
            created_at_time: Some(Nat::from(ic_cdk::api::time())),
        },
    };
    
    // Add the transaction to ICRC3 log
    match icrc3_add_transaction(transaction).await {
        Ok(hash) => {
            // Transaction logged successfully
            // Return success response
            Ok(OperationResponse { /* ... */ })
        },
        Err(e) => {
            // Handle error
            Err(Error::TransactionLogFailed(e))
        }
    }
}
```

#### Real-world examples (from ICRC7 implementation):

Here's how it's used in the context of an NFT transfer function:

```rust
#[update]
pub async fn icrc7_transfer(args: icrc7::icrc7_transfer::Args) -> icrc7::icrc7_transfer::Response {
    // Business logic and validation...
    
    // When the transfer is valid, record the transaction
    let transaction = Icrc3Transaction {
        btype: "7xfer".to_string(),
        timestamp: ic_cdk::api::time(),
        tx: TransactionData {
            tid: arg.token_id.clone(),
            from: Some(nft.token_owner.clone()),
            to: Some(arg.to.clone()),
            meta: None,
            memo: arg.memo.clone(),
            created_at_time: Some(Nat::from(time)),
        },
    };

    match icrc3_add_transaction(transaction).await {
        Ok(_) => {
            // Transaction logged successfully, update state
            txn_results[index] = Some(Ok(()));
            
            // Update the NFT state
            mutate_state(|state| state.data.update_token_by_id(&nft.token_id, &nft));
        },
        Err(e) => {
            // Handle error
            txn_results[index] = Some(Err((
                RejectionCode::CanisterError,
                format!("Failed to log transaction: {}", e),
            )));
        }
    }
    
    // Return response
    txn_results
}
```

And for NFT minting:

```rust
#[update]
pub async fn mint(req: management::mint::Args) -> management::mint::Response {
    // Business logic and validation...
    
    // Create a new token
    let new_token = nft::Icrc7Token::new(/* ... */);

    // Record the mint transaction
    let transaction = Icrc3Transaction {
        btype: "7mint".to_string(),
        timestamp: ic_cdk::api::time(),
        tx: TransactionData {
            tid: token_name_hash.clone(),
            from: None,  // No sender for minting
            to: Some(req.token_owner.clone()),
            meta: None,
            memo: req.memo.clone(),
            created_at_time: Some(Nat::from(ic_cdk::api::time())),
        },
    };

    match icrc3_add_transaction(transaction).await {
        Ok(_) => {
            // Add the token to state
            mutate_state(|state| {
                state.data.tokens_list.insert(token_name_hash.clone(), new_token);
            });
            
            Ok(token_name_hash.clone())
        },
        Err(e) => {
            Err((
                RejectionCode::CanisterError,
                format!("Failed to log transaction: {}", e),
            ))
        }
    }
}
```

## Types of transactions with ICRC7

With ICRC7 NFTs, you typically handle these transaction types:

```rust
// ICRC7 transaction types
let icrc3_config = ICRC3Config {
    supported_blocks: vec![
        SupportedBlockType {
            block_type: "7mint".to_string(),  // Minting a new NFT
            url: "https://github.com/dfinity/ICRC/blob/main/ICRCs/ICRC-7/ICRC-7.md#mint-block-schema",
        },
        SupportedBlockType {
            block_type: "7burn".to_string(),  // Burning (destroying) an NFT
            url: "https://github.com/dfinity/ICRC/blob/main/ICRCs/ICRC-7/ICRC-7.md#burn-block-schema",
        },
        SupportedBlockType {
            block_type: "7xfer".to_string(),  // Transferring an NFT
            url: "https://github.com/dfinity/ICRC/blob/main/ICRCs/ICRC-7/ICRC-7.md#icrc7_transfer-block-schema",
        },
        SupportedBlockType {
            block_type: "7update_token".to_string(),  // Updating token metadata
            url: "https://github.com/dfinity/ICRC/blob/main/ICRCs/ICRC-7/ICRC-7.md#update-token-block-schema",
        },
        // Other transaction types can be added as needed
    ],
};
```

Each transaction type has specific validation rules:

```rust
impl TransactionType for Icrc3Transaction {
    fn validate_transaction_fields(&self) -> Result<(), String> {
        match self.btype.as_str() {
            "7mint" => {
                if self.tx.from.is_some() {
                    return Err("From is not allowed for mint".to_string());
                }
                if self.tx.to.is_none() {
                    return Err("To is required for mint".to_string());
                }
            },
            "7burn" => {
                if self.tx.from.is_none() {
                    return Err("From is required for burn".to_string());
                }
                if self.tx.to.is_some() {
                    return Err("To is not allowed for burn".to_string());
                }
            },
            "7xfer" => {
                if self.tx.from.is_none() {
                    return Err("From is required for transfer".to_string());
                }
                if self.tx.to.is_none() {
                    return Err("To is required for transfer".to_string());
                }
            },
            "7update_token" => {
                if self.tx.meta.is_none() {
                    return Err("Meta is required for update_token".to_string());
                }
            },
            // Add validation for other types
            _ => {}
        }
        Ok(())
    }
}
```

## Advanced usage: Transaction metadata

ICRC3 transactions can include metadata to store additional information. For example, when updating NFT metadata:

```rust
// Example from update_nft_metadata function
let mut metadata_map = BTreeMap::new();

// Add metadata fields
if let Some(name) = req.token_name {
    token.token_name = name.clone();
    metadata_map.insert(
        "icrc7:token_name".to_string(),
        Icrc3Value::Text(name.clone()),
    );
}

// Create the transaction with metadata
let transaction = Icrc3Transaction {
    btype: "7update_token".to_string(),
    timestamp: ic_cdk::api::time(),
    tx: TransactionData {
        tid: token_id.clone(),
        from: Some(Account { owner: ic_cdk::caller(), subaccount: None }),
        to: None,
        meta: Some(Icrc3Value::Map(metadata_map)),  // Include the metadata
        memo: None,
        created_at_time: Some(Nat::from(ic_cdk::api::time())),
    },
};

// Log the transaction
icrc3_add_transaction(transaction).await?;
```

## Querying transaction history

Once you've implemented the ICRC3 endpoints, clients can query transaction history:

```typescript
// TypeScript example of querying transaction history
const blocks = await canister.icrc3_get_blocks({
  start: 0n,  // Start from the beginning
  length: 10n // Get 10 transactions
});

// Filter blocks by type
const mintBlocks = blocks.blocks.filter(block => 
  block.transaction.operation === "7mint"
);

// Get all available archives
const archives = await canister.icrc3_get_archives();
```

## Benefits for the Dfinity ecosystem

- **Reduction of code duplication**: Developers don't have to reimplement transaction management logic.
- **Better interoperability**: Facilitates interaction between different applications and services.
- **Standardization**: Encourages the adoption of consistent standards on the Internet Computer.
- **Scalability**: Provides a solution for automatic archiving of historical transactions.
- **Simplified auditing**: Offers a uniform interface for transaction auditing.
- **Enhanced user experience**: Allows wallets and explorers to display transaction history consistently.
- **Reduced development time**: Developers can focus on business logic rather than transaction infrastructure.

## Conclusion

ICRC3 is an essential library for any developer wishing to implement efficient and standardized transaction management on the Internet Computer. By following this guide, you can quickly integrate ICRC3 into your canisters and benefit from its advanced transaction management features.

The implementation is flexible enough to accommodate various transaction types while providing a standardized interface for clients to interact with. This makes it particularly valuable for token standards like ICRC1 and ICRC7, as well as any other canister that needs to maintain an auditable history of operations. 