pub const STATS: [Stat; 15] = [
    Stat::new("memory.current", "Current Total", "Current total memory usage including descendents"),
    Stat::new("memory.swap.current", "Current Swap", "Current total swap usage including descendents"),
    // from https://www.kernel.org/doc/Documentation/cgroup-v2.txt
    Stat::new("memory.stat/=anon/2", "Anonymous", "Amount of memory used in anonymous mappings."),
    Stat::new("memory.stat/=file/2", "File Cache", "Amount of memory used to cache filesystem data, including tmpfs and shared memory."),
    Stat::new("memory.stat/=kernel_stack/2", "Kernel Stack", "Amount of memory allocated to kernel stacks."),
    Stat::new("memory.stat/=pagetables/2", "Page Table", "Amount of memory used for page tables."),
    Stat::new("memory.stat/=percpu/2", "Per CPU", "Amount of memory used for per-cpu data structures."),
    Stat::new("memory.stat/=sock/2", "Socket", "Amount of memory used in network transmission buffers."),
    Stat::new("memory.stat/=shmem/2", "Swap Backed", "Amount of cached filesystem data that is swap-backed."),
    Stat::new("memory.stat/=file_mapped/2", "File Mapped", "Amount of cached filesystem data mapped."),
    Stat::new("memory.stat/=file_dirty/2", "File Dirty", "Amount of cached filesystem data that was modified but not yet written back to disk."),
    Stat::new("memory.stat/=file_writeback/2", "File Writeback", "Amount of cached filesystem data that was modified and is currently being written back to disk"),
    Stat::new("memory.stat/=swapcached/2", "", "Amount of memory cached in swap."),
    Stat::new("memory.stat/=unevictable/2", "Unevictable", "Amount of unevictable memory."),
    Stat::new("memory.stat/=slab/2", "Slab", "Amount of memory used for storing in-kernel data structures."),
];

pub struct Stat<'a> {
    def: &'a str,
    short_desc: &'a str,
    desc: &'a str
}

impl<'a> Stat<'a> {
    const fn new(def: &'a str, short_desc: &'a str, desc: &'a str) -> Self {
        Self {
            def,
            short_desc,
            desc
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
}