use crate::types::block_interface::Block;
use crate::types::encoded_blocks::EncodedBlock;
use crate::types::hash::HashOf;
use crate::types::sha256;

use candid::{Int, Nat};
use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use minicbor::{Decoder, Encoder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DefaultBlock {
    parent_hash: Option<HashOf<EncodedBlock>>,
    timestamp: u128,
    pub transaction: ICRC3Value,
}

// Helper functions to encode/decode ICRC3Value with minicbor
fn encode_icrc3_value(value: &ICRC3Value) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    let mut encoder = Encoder::new(&mut buffer);
    encode_icrc3_value_impl(value, &mut encoder)
        .map_err(|e| format!("failed to encode ICRC3Value: {:?}", e))?;
    Ok(buffer)
}

fn encode_icrc3_value_impl<W: minicbor::encode::Write>(
    value: &ICRC3Value,
    e: &mut Encoder<W>,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    match value {
        ICRC3Value::Blob(b) => {
            e.array(2)?.u8(0)?.bytes(b.as_slice())?;
        }
        ICRC3Value::Text(t) => {
            e.array(2)?.u8(1)?.str(t)?;
        }
        ICRC3Value::Nat(n) => {
            e.array(2)?.u8(2)?.u64(
                u64::try_from(n.0.clone())
                    .map_err(|_| minicbor::encode::Error::message("Nat too large for u64"))?,
            )?;
        }
        ICRC3Value::Int(i) => {
            e.array(2)?.u8(3)?.i64(
                i64::try_from(i.0.clone())
                    .map_err(|_| minicbor::encode::Error::message("Int too large for i64"))?,
            )?;
        }
        ICRC3Value::Array(arr) => {
            e.array(2)?.u8(4)?;
            e.array(arr.len() as u64)?;
            for val in arr {
                encode_icrc3_value_impl(val, e)?;
            }
        }
        ICRC3Value::Map(map) => {
            e.array(2)?.u8(5)?;
            e.map(map.len() as u64)?;
            for (k, v) in map.iter() {
                e.str(k)?;
                encode_icrc3_value_impl(v, e)?;
            }
        }
    }
    Ok(())
}

fn decode_icrc3_value(bytes: &[u8]) -> Result<ICRC3Value, String> {
    let mut decoder = Decoder::new(bytes);
    decode_icrc3_value_impl(&mut decoder)
        .map_err(|e| format!("failed to decode ICRC3Value: {:?}", e))
}

fn decode_icrc3_value_impl<'b>(d: &mut Decoder<'b>) -> Result<ICRC3Value, minicbor::decode::Error> {
    let len = d
        .array()?
        .ok_or_else(|| minicbor::decode::Error::message("expected array of length 2"))?;
    if len != 2 {
        return Err(minicbor::decode::Error::message(
            "expected array of length 2",
        ));
    }

    let tag = d.u8()?;

    let value = match tag {
        0 => {
            let bytes = d.bytes()?.to_vec();
            ICRC3Value::Blob(bytes.into())
        }
        1 => {
            let s = d.str()?.to_string();
            ICRC3Value::Text(s)
        }
        2 => {
            let n = d.u64()?;
            ICRC3Value::Nat(Nat::from(n))
        }
        3 => {
            let i = d.i64()?;
            ICRC3Value::Int(Int::from(i))
        }
        4 => {
            let array_len = d
                .array()?
                .ok_or_else(|| minicbor::decode::Error::message("expected array length"))?;
            let mut values = Vec::with_capacity(array_len as usize);
            for _ in 0..array_len {
                let v = decode_icrc3_value_impl(d)?;
                values.push(v);
            }
            ICRC3Value::Array(values)
        }
        5 => {
            let map_len = d
                .map()?
                .ok_or_else(|| minicbor::decode::Error::message("expected map length"))?;
            let mut map = std::collections::BTreeMap::new();
            for _ in 0..map_len {
                let k = d.str()?.to_string();
                let v = decode_icrc3_value_impl(d)?;
                map.insert(k, v);
            }
            ICRC3Value::Map(map)
        }
        _ => return Err(minicbor::decode::Error::message("invalid tag")),
    };

    Ok(value)
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

        let transaction_bytes =
            encode_icrc3_value(&self.transaction).expect("failed to encode transaction");
        encoded.extend_from_slice(&transaction_bytes);
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
        let transaction = decode_icrc3_value(&decoded[48..])
            .map_err(|e| format!("failed to decode transaction: {}", e))?;
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
