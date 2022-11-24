pub const STATS: [Stat; 17] = [
    Stat::new(
        "memory.current",
        "Current Total",
        "Current total memory usage including descendents",
        StatType::MemQtyCumul,
        // root needed        "smaps_rollup/=Rss:/2", "RSS",
        "status/=/1/VmRSS:/2",
        "RSS",
        ProcStatType::MemQtyKb,
    ),
    Stat::new(
        "memory.swap.current",
        "Current Swap",
        "Current total swap usage including descendents",
        StatType::MemQtyCumul,
        // root needed        "smaps_rollup/=Swap:/2", "Swap",
        "status/=/1/VmSwap:/2",
        "Swap",
        ProcStatType::MemQtyKb,
    ),
    Stat::new(
        "memory.stat/=/1/anon/2",
        "Anonymous",
        "Amount of memory used in anonymous mappings.",
        StatType::MemQtyCumul,
        // root needed        "smaps_rollup/=Anonymous:/2", "Anonymous",
        "status/=/1/RssAnon:/2",
        "Anonymous",
        ProcStatType::MemQtyKb,
    ),
    Stat::new(
        "memory.stat/=/1/file/2",
        "File Cache",
        "Amount of memory used to cache filesystem data, including tmpfs and shared memory.",
        StatType::MemQtyCumul,
        "",
        "",
        ProcStatType::None,
    ),
    Stat::new(
        "memory.stat/=/1/kernel_stack/2",
        "Kernel Stack",
        "Amount of memory allocated to kernel stacks.",
        StatType::MemQtyCumul,
        "",
        "",
        ProcStatType::None,
    ),
    Stat::new(
        "memory.stat/=/1/pagetables/2",
        "Page Table",
        "Amount of memory used for page tables.",
        StatType::MemQtyCumul,
        "status/=/1/VmPTE:/2",
        "VM PTE",
        ProcStatType::MemQtyKb,
    ),
    Stat::new(
        "memory.stat/=/1/percpu/2",
        "Per CPU",
        "Amount of memory used for per-cpu data structures.",
        StatType::MemQtyCumul,
        "",
        "",
        ProcStatType::None,
    ),
    Stat::new(
        "memory.stat/=/1/sock/2",
        "Socket",
        "Amount of memory used in network transmission buffers.",
        StatType::MemQtyCumul,
        "",
        "",
        ProcStatType::None,
    ),
    Stat::new(
        "memory.stat/=/1/shmem/2",
        "Swap Backed",
        "Amount of cached filesystem data that is swap-backed.",
        StatType::MemQtyCumul,
        "status/=RssShmem:/2",
        "RSS ShMem",
        ProcStatType::MemQtyKb,
    ),
    Stat::new(
        "memory.stat/=/1/file_mapped/2",
        "File Mapped",
        "Amount of cached filesystem data mapped.",
        StatType::MemQtyCumul,
        "status/=/1/RssFile:/2",
        "RSS File",
        ProcStatType::MemQtyKb,
    ),
    Stat::new(
        "memory.stat/=/1/file_dirty/2",
        "File Dirty",
        "Amount of cached filesystem data that was modified but not yet written back to disk.",
        StatType::MemQtyCumul,
        "",
        "",
        ProcStatType::None,
    ),
    Stat::new(
        "memory.stat/=/1/file_writeback/2",
        "File Writeback",
        "Amount of cached filesystem data that was modified and is currently being written back to disk",
        StatType::MemQtyCumul,
        "",
        "",
        ProcStatType::None,
    ),
    Stat::new(
        "memory.stat/=/1/swapcached/2",
        "Swap Cached",
        "Amount of memory cached in swap.",
        StatType::MemQtyCumul,
        "",
        "",
        ProcStatType::None,
    ),
    Stat::new(
        "memory.stat/=/1/unevictable/2",
        "Unevictable",
        "Amount of unevictable memory.",
        StatType::MemQtyCumul,
        "status/=/1/VmPin:/2",
        "VM Pin",
        ProcStatType::MemQtyKb,
    ),
    Stat::new(
        "memory.stat/=/1/slab/2",
        "Slab",
        "Amount of memory used for storing in-kernel data structures.",
        StatType::MemQtyCumul,
        "",
        "",
        ProcStatType::None,
    ),
    Stat::new("cgroup.procs/#", "Processes", "Number of processes.", StatType::Qty, "", "", ProcStatType::None),
    Stat::new("cgroup.threads/#", "Threads", "Number of threads.", StatType::Qty, "", "", ProcStatType::None),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatType {
    MemQtyCumul, // Cumulative memory quantity
    Qty,         // Count, non-cumulative
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcStatType {
    None,
    MemQtyKb,
}

pub struct Stat<'a> {
    def: &'a str,
    short_desc: &'a str,
    desc: &'a str,
    stype: StatType,
    proc_def: &'a str,
    proc_short_desc: &'a str,
    proc_stype: ProcStatType,
}

impl<'a> Stat<'a> {
    const fn new(
        def: &'a str,
        short_desc: &'a str,
        desc: &'a str,
        stype: StatType,
        proc_def: &'a str,
        proc_short_desc: &'a str,
        proc_stype: ProcStatType,
    ) -> Self {
        Self { def, short_desc, desc, stype, proc_def, proc_short_desc, proc_stype }
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

    pub fn proc_def(&self) -> &str {
        self.proc_def
    }

    pub fn proc_short_desc(&self) -> &str {
        self.proc_short_desc
    }

    pub fn proc_stat_type(&self) -> ProcStatType {
        self.proc_stype
    }
}
