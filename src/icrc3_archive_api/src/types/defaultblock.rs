use crate::types::block_interface::Block;
use crate::types::encoded_blocks::EncodedBlock;
use crate::types::hash::HashOf;
use crate::types::sha256;

use candid::{Decode, Encode};
use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DefaultBlock {
    parent_hash: Option<HashOf<EncodedBlock>>,
    timestamp: u128,
    pub transaction: ICRC3Value,
}

impl Block for DefaultBlock {
    fn from_transaction(
        parent_hash: Option<HashOf<EncodedBlock>>,
        tx: ICRC3Value,
        block_timestamp: u128,
    ) -> Self {
        Self {
            parent_hash,
            timestamp: block_timestamp,
            transaction: tx,
        }
    }

    fn encode(self) -> EncodedBlock {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(
            self.parent_hash
                .as_ref()
                .unwrap_or(&HashOf::<EncodedBlock>::new([0u8; 32]))
                .as_slice(),
        );
        encoded.extend_from_slice(&self.timestamp.to_le_bytes());

        encoded.extend_from_slice(&Encode!(&self.transaction).unwrap());
        EncodedBlock::from_vec(encoded)
    }

    fn decode(encoded: EncodedBlock) -> Result<Self, String> {
        let mut decoded = Vec::new();
        decoded.extend_from_slice(encoded.as_slice());
        let parent_hash = if decoded[0..32] == [0u8; 32] {
            None
        } else {
            Some(HashOf::<EncodedBlock>::new(
                decoded[0..32].try_into().unwrap(),
            ))
        };
        let timestamp = u128::from_le_bytes(decoded[32..48].try_into().unwrap());
        let transaction: ICRC3Value = Decode!(&mut &decoded[48..], ICRC3Value).unwrap();
        Ok(Self {
            parent_hash,
            timestamp,
            transaction,
        })
    }

    fn block_hash(encoded: &EncodedBlock) -> HashOf<EncodedBlock> {
        let mut state = sha256::Sha256::new();
        let mut buffer = Vec::new();
        minicbor::encode(encoded, &mut buffer).expect("failed to encode EncodedBlock");
        state.write(&buffer);
        HashOf::new(state.finish())
    }

    fn parent_hash(&self) -> Option<HashOf<EncodedBlock>> {
        self.parent_hash
    }

    fn timestamp(&self) -> u128 {
        self.timestamp
    }
}
