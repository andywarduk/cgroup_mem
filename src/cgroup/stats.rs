pub const STATS: [Stat; 4] = [
    Stat::new("memory.current", "Current Total", "Current total memory usage including descendents"),
    Stat::new("memory.swap.current", "Current Swap", "Current total swap usage including descendents"),
    // from https://www.kernel.org/doc/Documentation/cgroup-v2.txt
    Stat::new("memory.stat/=anon/2", "Anonymous", "Amount of memory used in anonymous mappings."),
    Stat::new("memory.stat/=file/2", "File Cache", "Amount of memory used to cache filesystem data, including tmpfs and shared memory."),
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