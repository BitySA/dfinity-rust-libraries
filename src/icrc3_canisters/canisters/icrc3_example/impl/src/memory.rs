use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};

const UPGRADES: MemoryId = MemoryId::new(0);
const BLOCK_LOG: MemoryId = MemoryId::new(1);

pub type VM = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: MemoryManager<DefaultMemoryImpl> = MemoryManager::init(
        DefaultMemoryImpl::default()
    );
}

pub fn get_upgrades_memory() -> VM {
    get_memory(UPGRADES)
}

/// Gives the ICRC3 block log its memory from this canister's manager. Call it
/// first in `init` and `post_upgrade`.
pub fn init_memory() {
    bity_ic_icrc3::memory::init_block_log_memory(get_memory(BLOCK_LOG));
}

fn get_memory(id: MemoryId) -> VM {
    MEMORY_MANAGER.with(|m| m.get(id))
}
