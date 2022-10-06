pub const STATS: [Stat; 17] = [
    Stat::new("memory.current", "Current Total", "Current total memory usage including descendents", StatType::MemQtyCumul),
    Stat::new("memory.swap.current", "Current Swap", "Current total swap usage including descendents", StatType::MemQtyCumul),
    Stat::new("memory.stat/=anon/2", "Anonymous", "Amount of memory used in anonymous mappings.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=file/2", "File Cache", "Amount of memory used to cache filesystem data, including tmpfs and shared memory.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=kernel_stack/2", "Kernel Stack", "Amount of memory allocated to kernel stacks.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=pagetables/2", "Page Table", "Amount of memory used for page tables.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=percpu/2", "Per CPU", "Amount of memory used for per-cpu data structures.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=sock/2", "Socket", "Amount of memory used in network transmission buffers.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=shmem/2", "Swap Backed", "Amount of cached filesystem data that is swap-backed.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=file_mapped/2", "File Mapped", "Amount of cached filesystem data mapped.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=file_dirty/2", "File Dirty", "Amount of cached filesystem data that was modified but not yet written back to disk.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=file_writeback/2", "File Writeback", "Amount of cached filesystem data that was modified and is currently being written back to disk", StatType::MemQtyCumul),
    Stat::new("memory.stat/=swapcached/2", "", "Amount of memory cached in swap.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=unevictable/2", "Unevictable", "Amount of unevictable memory.", StatType::MemQtyCumul),
    Stat::new("memory.stat/=slab/2", "Slab", "Amount of memory used for storing in-kernel data structures.", StatType::MemQtyCumul),
    Stat::new("cgroup.procs/#", "Processes", "Number of processes.", StatType::Qty),
    Stat::new("cgroup.threads/#", "Threads", "Number of threads.", StatType::Qty),
];

#[derive(Clone, Copy)]
pub enum StatType {
    MemQtyCumul, // Cumulative memory quantity
    Qty,         // Count, non-cumulative
}

pub struct Stat<'a> {
    def: &'a str,
    short_desc: &'a str,
    desc: &'a str,
    stype: StatType,
}

impl<'a> Stat<'a> {
    const fn new(def: &'a str, short_desc: &'a str, desc: &'a str, stype: StatType) -> Self {
        Self {
            def,
            short_desc,
            desc,
            stype,
        }
    }

    pub fn def(&self) -> &str {
        self.def
    }

    pub fn short_desc(&self) -> &str {
        self.short_desc
    }

    pub fn desc(&self) -> &str {
        self.desc
    }

    pub fn stat_type(&self) -> StatType {
        self.stype
    }
}