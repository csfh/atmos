//! The groups that take a row of their own across the page's columns.

/// The groups whose QML says `wide: true`, by page and group title: a chart or a table that
/// takes the whole width while the page's other groups sit two side by side. A group with no
/// title is the QML page's content outside every group, which takes the whole width too.
/// `tests/catalog_parity.rs` holds this to the QML.
#[rustfmt::skip]
pub const WIDE: &[(&str, &str)] = &[
    ("appearance", "Additional themes"),
    ("home", "Processes"),
    ("monitor", "Now"),
    ("monitor", "Cores"),
    ("monitor", "Hottest tasks"),
    ("monitor/cpu", "IRQ land"),
    ("monitor/cpu", "PSI ridgeline"),
    ("monitor/cpu", "Cores"),
    ("monitor/cpu", "Core frequency"),
    ("monitor/cpu", "Softirq rose"),
    ("monitor/cpu", "Histogram"),
    ("monitor/cpu", "Hottest"),
    ("monitor/disk", "I/O streamgraph"),
    ("monitor/disk", "Devices"),
    ("monitor/disk", "Busiest tasks"),
    ("monitor/memory", "Buddy horizon"),
    ("monitor/memory", "RSS treemap"),
    ("monitor/memory", "Meminfo Sankey"),
    ("monitor/memory", "Cgroup sunburst"),
    ("monitor/memory", "Slab icicle"),
    ("monitor/memory", "Largest"),
    ("monitor/processes", "Parallel coordinates"),
    ("monitor/sensors", "Thermal calendar"),
    ("monitor/sensors", "RAPL waterfall"),
    ("monitor/traffic", "TCP beeswarm"),
    ("monitor/traffic", "Netdev radar"),
    ("monitor/traffic", "Interfaces"),
];

/// Whether `page`'s group `title` takes a row of its own across the page's columns.
pub fn wide(page: &str, title: &str) -> bool {
    title.is_empty() || WIDE.contains(&(page, title))
}
