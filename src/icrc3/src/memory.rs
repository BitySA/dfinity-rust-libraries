//! Stable memory for the block log.
//!
//! The block log is a `StableBTreeMap` in memory id 1. A host canister that runs
//! its own `MemoryManager` must hand that memory over with
//! [`init_block_log_memory`] before it creates or deserializes an `ICRC3`.
//! Otherwise the library opens a second `MemoryManager` over the same stable
//! memory. Each manager keeps its own count of allocated buckets and its own
//! copy of every memory's size, and saves both over the shared header whenever
//! it grows a memory, so the two hand out the same buckets and record stale
//! sizes. [`repair_memory_manager_header`] cleans up a header left in that
//! state.

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, Memory,
};
use std::cell::{Cell, RefCell};

pub type VM = VirtualMemory<DefaultMemoryImpl>;
const BLOCK_LOG_DATA_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static HOST_BLOCK_LOG_MEMORY: RefCell<Option<VM>> = const { RefCell::new(None) };
    static OWN_MANAGER_IN_USE: Cell<bool> = const { Cell::new(false) };
    static MEMORY_MANAGER: MemoryManager<DefaultMemoryImpl> = MemoryManager::init(
        DefaultMemoryImpl::default()
    );
}

/// Hands the library the memory its block log lives in, taken from the host
/// canister's own `MemoryManager`. Pass memory id 1 to keep the blocks of a
/// canister that ran earlier versions of this library.
///
/// Call it at the start of `init` and `post_upgrade`, before anything creates
/// or deserializes an `ICRC3`.
///
/// # Panics
///
/// If the library already opened its own `MemoryManager` in this run.
pub fn init_block_log_memory(memory: VM) {
    if OWN_MANAGER_IN_USE.with(|in_use| in_use.get()) {
        panic!("init_block_log_memory: the block log already opened its own memory manager");
    }
    HOST_BLOCK_LOG_MEMORY.with(|host| *host.borrow_mut() = Some(memory));
}

pub fn get_block_log_data_memory() -> VM {
    if let Some(memory) = HOST_BLOCK_LOG_MEMORY.with(|host| host.borrow().clone()) {
        return memory;
    }
    OWN_MANAGER_IN_USE.with(|in_use| in_use.set(true));
    MEMORY_MANAGER.with(|m| m.get(BLOCK_LOG_DATA_MEMORY_ID))
}

// Layout version 1 of the ic-stable-structures memory manager header.
const MAGIC: &[u8; 3] = b"MGR";
const LAYOUT_VERSION: u8 = 1;
const ALLOCATED_BUCKETS_OFFSET: u64 = 4;
const MEMORY_SIZES_OFFSET: u64 = 40;
const MAX_NUM_MEMORIES: usize = 255;
const BUCKET_TABLE_OFFSET: u64 = MEMORY_SIZES_OFFSET + 8 * MAX_NUM_MEMORIES as u64;
const MAX_NUM_BUCKETS: usize = 32768;
const UNALLOCATED_BUCKET: u8 = 255;

/// What [`repair_memory_manager_header`] changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderRepair {
    /// The allocated bucket count before and after.
    pub allocated_buckets: (u16, u16),
    /// Memories whose recorded size was raised: id, old size and new size in pages.
    pub raised_sizes: Vec<(u8, u64, u64)>,
}

/// Makes a `MemoryManager` header consistent with its bucket table, for
/// canisters where two managers shared one stable memory.
///
/// The bucket table records which memory owns each bucket. The allocated count
/// and the memory sizes may be stale, because each manager saved its own view.
/// This raises the count above every owned bucket, so no owned bucket is handed
/// out again, and raises each memory's size to the full capacity of its
/// buckets, so data written past a stale size stays readable. A memory reported
/// larger than it is only grows into new buckets later than it otherwise would.
///
/// Call it before anything opens a `MemoryManager` over `memory`, at the start
/// of `post_upgrade`. It does nothing on a memory without a manager header, and
/// nothing on a consistent header, so it is safe to call on every upgrade.
///
/// Returns an error, and changes nothing, when the header shows two memories
/// sharing data: a memory records more pages than its buckets hold, so another
/// memory took one of its buckets.
pub fn repair_memory_manager_header<M: Memory>(memory: &M) -> Result<Option<HeaderRepair>, String> {
    if memory.size() == 0 {
        return Ok(None);
    }
    let mut start = [0u8; 8];
    memory.read(0, &mut start);
    if &start[0..3] != MAGIC {
        return Ok(None);
    }
    if start[3] != LAYOUT_VERSION {
        return Err(format!(
            "unsupported memory manager layout version {}",
            start[3]
        ));
    }
    let allocated = u16::from_le_bytes([start[4], start[5]]);
    let bucket_pages = u16::from_le_bytes([start[6], start[7]]) as u64;
    if bucket_pages == 0 {
        return Err("the memory manager header records a bucket size of 0 pages".to_string());
    }

    let mut size_bytes = vec![0u8; 8 * MAX_NUM_MEMORIES];
    memory.read(MEMORY_SIZES_OFFSET, &mut size_bytes);
    let mut sizes: Vec<u64> = size_bytes
        .chunks_exact(8)
        .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap()))
        .collect();
    let mut table = vec![0u8; MAX_NUM_BUCKETS];
    memory.read(BUCKET_TABLE_OFFSET, &mut table);

    let physical_buckets = memory.size().saturating_sub(1) / bucket_pages;
    let mut owned = vec![0u64; MAX_NUM_MEMORIES];
    let mut highest_owned: Option<u64> = None;
    for (bucket, owner) in table.iter().enumerate() {
        if *owner == UNALLOCATED_BUCKET {
            continue;
        }
        let bucket = bucket as u64;
        if bucket >= physical_buckets {
            return Err(format!(
                "bucket {bucket} of memory {owner} lies past the end of stable memory"
            ));
        }
        owned[*owner as usize] += 1;
        highest_owned = Some(bucket);
    }

    for (id, size) in sizes.iter().enumerate() {
        let needed = size.div_ceil(bucket_pages);
        if needed > owned[id] {
            return Err(format!(
                "memory {id} records {size} pages but owns {} buckets of {bucket_pages} pages, so another memory took one of its buckets",
                owned[id]
            ));
        }
    }

    let new_allocated = match highest_owned {
        Some(bucket) => allocated.max(bucket as u16 + 1),
        None => allocated,
    };
    let mut raised_sizes = vec![];
    for (id, size) in sizes.iter_mut().enumerate() {
        let capacity = owned[id] * bucket_pages;
        if *size < capacity {
            raised_sizes.push((id as u8, *size, capacity));
            *size = capacity;
        }
    }
    if new_allocated == allocated && raised_sizes.is_empty() {
        return Ok(None);
    }

    memory.write(ALLOCATED_BUCKETS_OFFSET, &new_allocated.to_le_bytes());
    let size_bytes: Vec<u8> = sizes.iter().flat_map(|size| size.to_le_bytes()).collect();
    memory.write(MEMORY_SIZES_OFFSET, &size_bytes);
    Ok(Some(HeaderRepair {
        allocated_buckets: (allocated, new_allocated),
        raised_sizes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_stable_structures::{StableBTreeMap, VectorMemory};

    const BUCKET_BYTES: usize = 128 * 65536;

    fn map(manager: &MemoryManager<VectorMemory>, id: u8) -> StableBTreeMap<u64, Vec<u8>, VirtualMemory<VectorMemory>> {
        StableBTreeMap::init(manager.get(MemoryId::new(id)))
    }

    /// The live core_nft history: the block log's manager opens first, the
    /// host's manager opens at the first mint, an upgrade reloads both, the
    /// host gives a new memory a bucket, and then the block log grows inside
    /// its bucket and saves its stale view.
    fn two_manager_history() -> VectorMemory {
        let stable = VectorMemory::default();
        {
            let block_log = MemoryManager::init(stable.clone());
            let mut blocks = map(&block_log, 1);
            blocks.insert(0, vec![1; 100]);
            let host = MemoryManager::init(stable.clone());
            let mut metadata = map(&host, 5);
            metadata.insert(0, vec![5; 100]);
            let mut upgrades = map(&host, 2);
            upgrades.insert(0, vec![2; 100]);
        }
        // Upgrade: both managers reload the same header.
        let block_log = MemoryManager::init(stable.clone());
        let host = MemoryManager::init(stable.clone());
        let mut approvals = map(&host, 3);
        approvals.insert(7, vec![3; 100]);
        let mut blocks = map(&block_log, 1);
        for index in 1..200 {
            blocks.insert(index, vec![1; 1000]);
        }
        stable
    }

    fn header(stable: &VectorMemory) -> (u16, Vec<u64>) {
        let mut start = [0u8; 8];
        stable.read(0, &mut start);
        let mut size_bytes = vec![0u8; 8 * MAX_NUM_MEMORIES];
        stable.read(MEMORY_SIZES_OFFSET, &mut size_bytes);
        let sizes = size_bytes
            .chunks_exact(8)
            .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap()))
            .collect();
        (u16::from_le_bytes([start[4], start[5]]), sizes)
    }

    #[test]
    fn the_shared_history_leaves_a_stale_header() {
        let stable = two_manager_history();
        let (allocated, sizes) = header(&stable);
        assert_eq!(allocated, 3, "the block log's manager saved its count");
        assert_eq!(sizes[3], 0, "the block log's manager never saw memory 3");
    }

    #[test]
    fn repair_lets_one_manager_read_everything_and_allocate_a_free_bucket() {
        let stable = two_manager_history();

        let repair = repair_memory_manager_header(&stable)
            .expect("the header is repairable")
            .expect("the header needed a repair");
        assert_eq!(repair.allocated_buckets, (3, 4));
        assert!(repair.raised_sizes.iter().any(|(id, old, new)| *id == 3 && *old == 0 && *new == 128));

        let manager = MemoryManager::init(stable.clone());
        assert_eq!(map(&manager, 3).get(&7), Some(vec![3; 100]));
        assert_eq!(map(&manager, 5).get(&0), Some(vec![5; 100]));
        assert_eq!(map(&manager, 2).get(&0), Some(vec![2; 100]));
        let blocks = map(&manager, 1);
        assert_eq!(blocks.len(), 200);
        assert_eq!(blocks.get(&199), Some(vec![1; 1000]));

        // A memory opened after the repair gets bucket 4, not the approvals'.
        let mut collection_approvals = map(&manager, 4);
        collection_approvals.insert(1, vec![4; 100]);
        assert_eq!(map(&manager, 3).get(&7), Some(vec![3; 100]));
        assert_eq!(collection_approvals.get(&1), Some(vec![4; 100]));
        assert_eq!(stable.size(), 1 + 5 * 128);
    }

    #[test]
    fn repair_is_a_no_op_the_second_time() {
        let stable = two_manager_history();
        assert!(repair_memory_manager_header(&stable).unwrap().is_some());
        assert_eq!(repair_memory_manager_header(&stable), Ok(None));
    }

    #[test]
    fn repair_ignores_memory_without_a_manager() {
        assert_eq!(repair_memory_manager_header(&VectorMemory::default()), Ok(None));
        let raw = VectorMemory::default();
        raw.grow(1);
        raw.write(0, b"not a manager");
        assert_eq!(repair_memory_manager_header(&raw), Ok(None));
    }

    #[test]
    fn repair_leaves_a_single_manager_header_alone_once_sizes_are_full() {
        let stable = VectorMemory::default();
        {
            let manager = MemoryManager::init(stable.clone());
            map(&manager, 1).insert(0, vec![1; 10]);
        }
        // The first call raises the size to the bucket's capacity.
        assert!(repair_memory_manager_header(&stable).unwrap().is_some());
        assert_eq!(repair_memory_manager_header(&stable), Ok(None));
    }

    #[test]
    fn repair_refuses_a_memory_that_lost_a_bucket() {
        let stable = VectorMemory::default();
        {
            let manager = MemoryManager::init(stable.clone());
            let mut blocks = map(&manager, 1);
            blocks.insert(0, vec![1; 10]);
            // Grow memory 1 into a second bucket.
            for index in 1..(BUCKET_BYTES / 1000 + 50) as u64 {
                blocks.insert(index, vec![1; 1000]);
            }
            map(&manager, 5).insert(0, vec![5; 10]);
        }
        let (_, sizes) = header(&stable);
        assert!(sizes[1] > 128);
        // Another memory takes memory 1's second bucket.
        let mut table = vec![0u8; MAX_NUM_BUCKETS];
        stable.read(BUCKET_TABLE_OFFSET, &mut table);
        let second = table.iter().enumerate().filter(|(_, owner)| **owner == 1).nth(1).unwrap().0;
        stable.write(BUCKET_TABLE_OFFSET + second as u64, &[5]);
        let before = header(&stable);

        let error = repair_memory_manager_header(&stable).unwrap_err();
        assert!(error.contains("memory 1"), "{error}");
        assert_eq!(header(&stable), before, "nothing is written on refusal");
    }

    #[test]
    fn repair_refuses_an_unknown_layout() {
        let stable = VectorMemory::default();
        let _ = MemoryManager::init(stable.clone());
        stable.write(3, &[2]);
        assert!(repair_memory_manager_header(&stable).is_err());
    }
}
