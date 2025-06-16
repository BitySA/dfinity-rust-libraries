use crate::hash::HashOf;
use crate::types::encoded_blocks::EncodedBlock;

use icrc_ledger_types::icrc::generic_value::ICRC3Value;

/// Position of a block in the chain. The first block has position 0.
pub type BlockIndex = u64;

pub trait Block: Sized + Clone {
    /// Constructs a new block containing the given transaction.
    ///
    /// Law:
    ///
    /// ```text
    /// forall PH, TX, TS, FEE:
    ///     from_transaction(PH, TX, TS, FEE).parent_hash() = PH
    ///   ∧ from_transaction(PH, TX, TS, FEE).timestamp() = TS
    /// ```
    fn from_transaction(
        parent_hash: Option<HashOf<EncodedBlock>>,
        tx: ICRC3Value,
        block_timestamp: u128,
    ) -> Self;

    /// Encodes this block into a binary representation.
    ///
    /// NB. the binary representation is not guaranteed to be stable over time.
    /// I.e., there is no guarantee that
    ///
    /// ```text
    /// forall B: encode(B) == encode(decode(encode(B))).unwrap()
    /// ```
    ///
    /// One practical implication is that we can encode each block at most once before appending it
    /// to a blockchain.
    fn encode(self) -> EncodedBlock;

    /// Decodes a block from a binary representation.
    ///
    /// Law: forall B: decode(encode(B)) == Ok(B)
    fn decode(encoded: EncodedBlock) -> Result<Self, String>;

    /// Returns the hash of the encoded block.
    ///
    /// NB. it feels more natural and safe to compute the hash of typed blocks, i.e.,
    /// define `fn block_hash(&self) -> HashOf<EncodedBlock>`.
    /// This does not work in practice because the hash is usually defined only on the encoded
    /// representation, and the encoding is not guaranteed to be stable.
    ///
    /// # Panics
    ///
    /// This method can panic if the `encoded` block was not obtained
    /// by calling [encode] on the same block type.
    fn block_hash(encoded: &EncodedBlock) -> HashOf<EncodedBlock>;

    /// Returns the hash of the parent block.
    ///
    /// NB. Only the first block in a chain can miss a parent block hash.
    fn parent_hash(&self) -> Option<HashOf<EncodedBlock>>;

    /// Returns the time at which the ledger constructed this block.
    fn timestamp(&self) -> u128;
}
