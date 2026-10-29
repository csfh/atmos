//! The Home pages' groups and rows, as the QML pages have them in order. Search reads
//! the same words (`tests/catalog_parity.rs` holds them to `search.query`).

use super::control::Control::{Custom, Go};
use super::custom::Custom::{InList, Live, Meter};
use super::{Group, Page, line};

#[rustfmt::skip]
pub const PAGES: &[Page] = &[
    Page {
        id: "home",
        file: "HomePage.qml",
        description: "How this machine is doing right now. Monitor is the full process-manager. Machine under System is the identity page.",
        groups: &[
            Group { title: "Quick access", detail: "", rows: &[
            ] },
            Group { title: "Activity", detail: "Samples stay in this window. Monitor keeps the same poller and adds per-core, disk, traffic, and sensors. Nothing is written to disk.", rows: &[
                line("Processor", "Share of time the CPUs were busy, from /proc/stat.", "/proc/stat", &["cpu", "load", "sparkline"]).with(Custom(Live("cpu"))),
                line("Memory", "In use against what is fitted.", "/proc/meminfo", &["memory", "ram"]).with(Custom(Live("mem"))),
                line("Network", "Receive and transmit on non-loopback interfaces.", "/proc/net/dev", &["network", "bandwidth", "rx", "tx"]).with(Custom(Live("net"))),
            ] },
            Group { title: "Thermal", detail: "Package and GPU sensors. GPU names come from Hardware. A missing reading stays unknown.", rows: &[
                line("Processor", "CPU package temperature.", "/sys/class/hwmon · /sys/class/thermal", &["temperature", "cpu", "heat", "thermal", "package"]).with(Custom(Live("temp"))),
            ] },
            Group { title: "Processes", detail: "This user's processes. End sends SIGTERM. Force quit sends SIGKILL. Pid 1 and Atmos itself are refused.", rows: &[
                line("Monitor", "Per-core load, memory composition, disk I/O, traffic, sensors, and a full process table.", "", &["monitor", "htop", "btop", "process"]).with(Go("Configure…", "monitor")),
            ] },
        ],
    },
    Page {
        id: "monitor",
        file: "MonitorPage.qml",
        description: "Live load, memory, disks, traffic, sensors, and processes. Home is the short glance. Samples stay in this window.",
        groups: &[
            Group { title: "Sampling", detail: "Samples stay in this window. Pause freezes the graphs. Interval is how often /proc is read.", rows: &[
            ] },
            Group { title: "Cores", detail: "One bar per logical CPU from /proc/stat. Height is busy percent since the last sample.", rows: &[
            ] },
            Group { title: "Pages", detail: "Each page is a process-manager surface: filters, rates, and the same live sample.", rows: &[
            ] },
            Group { title: "Hottest tasks", detail: "This user's processes, sorted by CPU. The full table is on Processes.", rows: &[
                line("Full table", "Every filter, sort, and signal is on Processes.", "", &["process", "kill", "pid"]).with(Go("Configure…", "monitor/processes")),
            ] },
        ],
    },
    Page {
        id: "monitor/processes",
        file: "monitor/ProcessesPage.qml",
        description: "Search, filter, sort, and signal tasks. End sends SIGTERM. Force quit sends SIGKILL. Pid 1 and Atmos itself are refused.",
        groups: &[
            Group { title: "Parallel coordinates", detail: "One polyline per listed process across CPU%, RSS, FDs, threads, and nice. Rows without those fields stay off the chart.", rows: &[
            ] },
        ],
    },
    Page {
        id: "monitor/cpu",
        file: "monitor/CpuPage.qml",
        description: "Per-core busy time, frequency, load averages, and pressure stall from /proc.",
        groups: &[
            Group { title: "Package", detail: "Aggregate busy percent from /proc/stat. Load is runnable tasks over 1, 5, and 15 minutes.", rows: &[
                line("Busy", "Share of time every CPU was not idle or in iowait.", "/proc/stat", &["cpu", "load", "busy"]).with(Custom(Meter("busy"))),
                line("Load", "1, 5, and 15 minute load averages.", "/proc/loadavg", &["loadavg", "runnable"]).with(Custom(Meter("load"))),
                line("Pressure", "Share of time some tasks stalled on CPU. From PSI when the kernel publishes it.", "/proc/pressure/cpu", &["psi", "stall", "pressure"]).with(Custom(Meter("cpupsi"))),
            ] },
            Group { title: "IRQ land", detail: "Per-CPU interrupt rate since the last sample. Fill is relative to the hottest cell. Missing /proc/interrupts stays empty.", rows: &[
            ] },
            Group { title: "PSI ridgeline", detail: "cpu, memory, and io some avg10 overlapping on one time scale.", rows: &[
            ] },
            Group { title: "Cores", detail: "Each bar is one logical CPU. Frequency is scaling_cur_freq when cpufreq is present.", rows: &[
                line("CPU ", "", "/sys/devices/system/cpu", &["core", "frequency", "governor"]).with(Custom(InList)),
            ] },
            Group { title: "Core frequency", detail: "Violin of scaling_cur_freq P-state spread per core over the sample window.", rows: &[
            ] },
            Group { title: "Softirq rose", detail: "Nightingale rose of /proc/softirqs rates. Wedge area encodes the rate.", rows: &[
            ] },
            Group { title: "Histogram", detail: "How process CPU is distributed across the current sample, including other users.", rows: &[
            ] },
            Group { title: "Hottest", detail: "Tasks using at least 1% of a core in the last interval.", rows: &[
                line("No hot tasks", "Nothing is using at least 1% of a core right now.", "", &["empty", "cpu"]).with(Custom(Meter("nohot"))),
            ] },
        ],
    },
    Page {
        id: "monitor/memory",
        file: "monitor/MemoryPage.qml",
        description: "Used, cache, buffers, swap, and pressure. Values come from /proc/meminfo.",
        groups: &[
            Group { title: "Composition", detail: "Used is total minus free, buffers, and cached. Available is what the kernel will still give to apps.", rows: &[
                line("Used", "Total minus MemAvailable.", "/proc/meminfo", &["used", "rss"]).with(Custom(Meter("used"))),
                line("Available", "What can still be given to userspace without reclaiming too hard.", "MemAvailable", &["available", "free"]).with(Custom(Meter("avail"))),
                line("Free", "Completely unused pages.", "MemFree", &["free"]).with(Custom(Meter("free"))),
                line("Buffers", "Block device buffers.", "Buffers", &["buffers"]).with(Custom(Meter("buffers"))),
                line("Cached", "Page cache. Reclaimable under pressure.", "Cached", &["cache", "cached"]).with(Custom(Meter("cached"))),
                line("Anonymous", "Anonymous pages, mostly process heap and stack.", "AnonPages", &["anon", "heap"]).with(Custom(Meter("anon"))),
                line("Shared", "tmpfs and shared memory.", "Shmem", &["shared", "tmpfs", "shmem"]).with(Custom(Meter("shared"))),
                line("Dirty", "Pages waiting to be written back.", "Dirty", &["dirty", "writeback"]).with(Custom(Meter("dirty"))),
            ] },
            Group { title: "Buddy horizon", detail: "Free pages from /proc/buddyinfo, weighted by order, folded into horizon bands. Darker is a larger free pool.", rows: &[
            ] },
            Group { title: "RSS treemap", detail: "Squarified treemap of VmRSS nested by user then comm. Area is proportional to resident set.", rows: &[
            ] },
            Group { title: "Meminfo Sankey", detail: "MemTotal flowing into Anon, File, Slab, PageTables, KernelStack, Shmem, and Swap. Link width is the kilobyte flow.", rows: &[
            ] },
            Group { title: "Cgroup sunburst", detail: "Concentric rings of memory.current. Angle is proportional to bytes; radius is cgroup depth. Unreadable cgroup files stay empty.", rows: &[
            ] },
            Group { title: "Slab icicle", detail: "Cascading rectangles of /proc/slabinfo occupancy. Length is active objects; depth is the cache prefix. Unreadable slabinfo stays empty.", rows: &[
            ] },
            Group { title: "Swap", detail: "Swap used against SwapTotal. Zero total means swap is off.", rows: &[
                line("Used", "Pages currently on swap.", "SwapTotal SwapFree", &["swap"]).with(Custom(Meter("swap"))),
                line("Pressure", "Share of time some tasks stalled on memory.", "/proc/pressure/memory", &["psi", "stall"]).with(Custom(Meter("mempsi"))),
            ] },
            Group { title: "Largest", detail: "Resident set of at least 10 MB in the current sample.", rows: &[
                line("No large tasks", "Nothing is holding 10 MB of RSS right now.", "", &["empty", "memory"]).with(Custom(Meter("nolarge"))),
            ] },
        ],
    },
    Page {
        id: "monitor/disk",
        file: "monitor/DiskPage.qml",
        description: "Read and write rates from /proc/diskstats. Partitions are skipped. Capacity lives on Disks.",
        groups: &[
            Group { title: "I/O streamgraph", detail: "Stacked bandwidth about a centerline, one layer per whole disk from /proc/diskstats.", rows: &[
            ] },
            Group { title: "Pressure", detail: "Share of time some tasks stalled on I/O.", rows: &[
                line("I/O stall", "PSI some avg10 for io.", "/proc/pressure/io", &["psi", "stall", "io"]).with(Custom(Meter("iopsi"))),
            ] },
            Group { title: "Devices", detail: "Whole disks only. Dual sparkline is read over write.", rows: &[
                line("No disks", "", "", &["empty", "disk"]).with(Custom(Meter("nodisks"))),
            ] },
            Group { title: "Busiest tasks", detail: "Process read and write bytes from /proc/pid/io when that file is readable.", rows: &[
                line("No I/O samples", "Process I/O waits for a second sample, and some tasks hide /proc/pid/io.", "", &["empty", "io"]).with(Custom(Meter("noio"))),
            ] },
        ],
    },
    Page {
        id: "monitor/traffic",
        file: "monitor/TrafficPage.qml",
        description: "Per-interface rates from /proc/net/dev and TCP socket counts. Loopback is skipped. Wi-Fi lives on Network.",
        groups: &[
            Group { title: "Total", detail: "Receive and transmit across every non-loopback interface.", rows: &[
                line("Bandwidth", "Aggregate of /proc/net/dev, skipping lo.", "/proc/net/dev", &["bandwidth", "rx", "tx"]).with(Custom(Meter("bandwidth"))),
            ] },
            Group { title: "TCP beeswarm", detail: "One point per socket from /proc/net/tcp and tcp6, jittered and grouped by TCP state.", rows: &[
            ] },
            Group { title: "Netdev radar", detail: "One polygon per NIC on shared rx/tx bytes, packets, drops, and errs axes from /proc/net/dev.", rows: &[
            ] },
            Group { title: "Sockets", detail: "IPv4 and IPv6 TCP from /proc/net/tcp and tcp6.", rows: &[
                line("Established", "Active TCP connections.", "/proc/net/tcp", &["tcp", "established"]).with(Custom(Meter("established"))),
                line("Listen", "Sockets waiting for a client.", "st 0A", &["listen", "port"]).with(Custom(Meter("listen"))),
                line("Time-wait", "Sockets in TIME_WAIT.", "st 06", &["time-wait", "tcp"]).with(Custom(Meter("timewait"))),
                line("Close-wait", "Sockets in CLOSE_WAIT.", "st 08", &["close-wait"]).with(Custom(Meter("closewait"))),
            ] },
            Group { title: "Interfaces", detail: "One dual sparkline per interface. Receive is the accent stroke. Transmit is muted.", rows: &[
                line("No interfaces", "", "", &["empty", "interface"]).with(Custom(Meter("noifaces"))),
            ] },
        ],
    },
    Page {
        id: "monitor/sensors",
        file: "monitor/SensorsPage.qml",
        description: "hwmon temperatures and fans, plus GPU busy and VRAM when the driver publishes them.",
        groups: &[
            Group { title: "Package", detail: "CPU package temperature, the same reading Home uses.", rows: &[
                line("Processor", "CPU package from hwmon or thermal_zone.", "/sys/class/hwmon", &["cpu", "temperature", "package"]).with(Custom(Meter("temp"))),
            ] },
            Group { title: "Thermal calendar", detail: "Week × weekday cells of daily-max °C from hwmon temp*_input in this window's sample ring. No private history store.", rows: &[
            ] },
            Group { title: "RAPL waterfall", detail: "Sequential joules since the last sample from powercap energy_uj. This AMD host may have no RAPL nodes; the chart stays empty instead of inventing zeros.", rows: &[
            ] },
            Group { title: "Graphics", detail: "Busy and VRAM come from the DRM device or nvidia-smi. Missing stays unknown.", rows: &[
                line("No GPUs", "Hardware inventory did not name a graphics device.", "", &["empty", "gpu"]).with(Custom(Meter("nogpus"))),
            ] },
            Group { title: "hwmon", detail: "Every readable temp*_input and fan*_input. Zero and missing stay off the list.", rows: &[
                line("No sensors", "", "", &["empty", "sensor"]).with(Custom(Meter("nosensors"))),
            ] },
        ],
    },
];
