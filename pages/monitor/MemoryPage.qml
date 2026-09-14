import QtQuick
import "../../components"
import "../../services"
import "../../services/Charts.js" as ChartsJs
import "../../services/LiveStats.js" as LiveStatsJs
import "../../services/Monitor.js" as MonitorJs
import "../../services/Processes.js" as ProcessesJs
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "monitor/memory"
  title: "Memory"
  description: "Used, cache, buffers, swap, and pressure. Values come from /proc/meminfo."

  readonly property var latest: LiveStatsStore.latest
  readonly property var history: LiveStatsStore.history
  readonly property var memValues: LiveStatsJs.series(root.history, "mem")
  readonly property var swapValues: LiveStatsJs.series(root.history, "swap")
  readonly property var parts: LiveStatsJs.memParts(root.latest)
  readonly property var buddyValues: ChartsJs.buddySeries(root.history)
  readonly property var rssTree: ChartsJs.rssTree(root.latest && root.latest.processes)
  readonly property var memSankey: ChartsJs.meminfoSankey(root.latest)
  readonly property var cgroupTree: ChartsJs.cgroupTree(root.latest && root.latest.cgroups)
  readonly property var slabTree: ChartsJs.slabTree(root.latest && root.latest.slabs)
  readonly property var fatProcs: ProcessesJs.list(
    root.latest && root.latest.processes ? root.latest.processes : [],
    "",
    "memory",
    { scope: "all", uid: root.latest ? root.latest.uid : null, minRssKb: 10240, cap: 16 }
  )

  function kbText(kb) {
    if (kb == null) return "unknown"
    return RichUi.formatBytes(LiveStatsJs.memBytes(kb))
  }

  function waiting() {
    return LiveStatsStore.waiting ? "waiting for samples" : "unknown"
  }

  PrefsGroup {
    title: "Composition"
    query: root.query
    detail: "Used is total minus free, buffers, and cached. Available is what the kernel will still give to apps."

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset
      spacing: Theme.headingGap

      PrefsStackedBar {
        width: parent.width
        parts: root.parts
        valueText: LiveStatsJs.formatPercent(root.latest && root.latest.mem)
      }

      PrefsSparkline {
        width: parent.width
        values: root.memValues
        valueText: LiveStatsJs.formatPercent(root.latest && root.latest.mem)
        alert: MonitorJs.alertLevel(root.latest && root.latest.mem) === "hot"
      }
    }

    SettingRow {
      label: "Used"
      description: "Total minus MemAvailable."
      hint: "/proc/meminfo"
      query: root.query
      keywords: ["used", "rss"]
      valueText: root.latest ? root.kbText(root.latest.memUsed) : root.waiting()
    }

    SettingRow {
      label: "Available"
      description: "What can still be given to userspace without reclaiming too hard."
      hint: "MemAvailable"
      query: root.query
      keywords: ["available", "free"]
      valueText: root.latest ? root.kbText(root.latest.memAvail) : root.waiting()
    }

    SettingRow {
      label: "Free"
      description: "Completely unused pages."
      hint: "MemFree"
      query: root.query
      keywords: ["free"]
      valueText: root.latest ? root.kbText(root.latest.memFree) : root.waiting()
    }

    SettingRow {
      label: "Buffers"
      description: "Block device buffers."
      hint: "Buffers"
      query: root.query
      keywords: ["buffers"]
      valueText: root.latest ? root.kbText(root.latest.memBuffers) : root.waiting()
    }

    SettingRow {
      label: "Cached"
      description: "Page cache. Reclaimable under pressure."
      hint: "Cached"
      query: root.query
      keywords: ["cache", "cached"]
      valueText: root.latest ? root.kbText(root.latest.memCached) : root.waiting()
    }

    SettingRow {
      label: "Anonymous"
      description: "Anonymous pages, mostly process heap and stack."
      hint: "AnonPages"
      query: root.query
      keywords: ["anon", "heap"]
      valueText: root.latest ? root.kbText(root.latest.memAnon) : root.waiting()
    }

    SettingRow {
      label: "Shared"
      description: "tmpfs and shared memory."
      hint: "Shmem"
      query: root.query
      keywords: ["shared", "tmpfs", "shmem"]
      valueText: root.latest ? root.kbText(root.latest.memShared) : root.waiting()
    }

    SettingRow {
      label: "Dirty"
      description: "Pages waiting to be written back."
      hint: "Dirty"
      query: root.query
      keywords: ["dirty", "writeback"]
      valueText: root.latest ? root.kbText(root.latest.memDirty) : root.waiting()
    }
  }

  PrefsGroup {
    title: "Buddy horizon"
    query: root.query
    wide: true
    detail: "Free pages from /proc/buddyinfo, weighted by order, folded into horizon bands. Darker is a larger free pool."
    hint: "/proc/buddyinfo"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        width: parent.width
        kind: "horizon"
        model: root.buddyValues
        valueText: "Buddy-order horizon"
      }
    }
  }

  PrefsGroup {
    title: "RSS treemap"
    query: root.query
    wide: true
    detail: "Squarified treemap of VmRSS nested by user then comm. Area is proportional to resident set."
    hint: "/proc/*/status"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        width: parent.width
        kind: "treemap"
        model: root.rssTree
        valueText: "Process RSS treemap"
      }
    }
  }

  PrefsGroup {
    title: "Meminfo Sankey"
    query: root.query
    wide: true
    detail: "MemTotal flowing into Anon, File, Slab, PageTables, KernelStack, Shmem, and Swap. Link width is the kilobyte flow."
    hint: "/proc/meminfo"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        width: parent.width
        kind: "sankey"
        model: root.memSankey
        valueText: "Meminfo composition Sankey"
      }
    }
  }

  PrefsGroup {
    title: "Cgroup sunburst"
    query: root.query
    wide: true
    detail: "Concentric rings of memory.current. Angle is proportional to bytes; radius is cgroup depth. Unreadable cgroup files stay empty."
    hint: "/sys/fs/cgroup/**/memory.current"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        width: parent.width
        kind: "sunburst"
        model: root.cgroupTree
        valueText: "Cgroup memory sunburst"
      }
    }
  }

  PrefsGroup {
    title: "Slab icicle"
    query: root.query
    wide: true
    detail: "Cascading rectangles of /proc/slabinfo occupancy. Length is active objects; depth is the cache prefix. Unreadable slabinfo stays empty."
    hint: "/proc/slabinfo"

    Column {
      width: parent.width - Theme.copyInset * 2
      x: Theme.copyInset

      PrefsChart {
        width: parent.width
        kind: "icicle"
        model: root.slabTree
        valueText: "Slab cache icicle"
      }
    }
  }

  PrefsGroup {
    title: "Swap"
    query: root.query
    detail: "Swap used against SwapTotal. Zero total means swap is off."

    SettingRow {
      label: "Used"
      description: "Pages currently on swap."
      hint: "SwapTotal SwapFree"
      query: root.query
      keywords: ["swap"]
      valueText: {
        if (!root.latest) return root.waiting()
        if (root.latest.swapTotal === 0) return "off"
        var used = root.kbText(root.latest.swapUsed)
        var total = root.kbText(root.latest.swapTotal)
        var pct = LiveStatsJs.formatPercent(root.latest.swap)
        return used + " / " + total + (pct ? "  " + pct : "")
      }
      stretchControl: true

      PrefsSparkline {
        width: parent.width
        values: root.swapValues
        valueText: LiveStatsJs.formatPercent(root.latest && root.latest.swap)
        alert: MonitorJs.alertLevel(root.latest && root.latest.swap) === "hot"
      }
    }

    SettingRow {
      label: "Pressure"
      description: "Share of time some tasks stalled on memory."
      hint: "/proc/pressure/memory"
      query: root.query
      keywords: ["psi", "stall"]
      valueText: root.latest && root.latest.psi ? LiveStatsJs.formatPsi(root.latest.psi.memory) || "unknown" : root.waiting()
    }
  }

  PrefsGroup {
    title: "Largest"
    query: root.query
    wide: true
    detail: "Resident set of at least 10 MB in the current sample."

    Repeater {
      model: root.fatProcs

      ProcessRow {
        required property var modelData
        procRow: modelData
        query: root.query
        onActed: function(action) {
          if (action === "copy") Omarchy.copyText(String(modelData.pid))
          else if (action === "copyCmd") Omarchy.copyText(String(modelData.cmdline || modelData.comm))
        }
      }
    }

    SettingRow {
      available: root.fatProcs.length === 0
      sectionHelp: false
      label: "No large tasks"
      description: "Nothing is holding 10 MB of RSS right now."
      query: root.query
      keywords: ["empty", "memory"]
    }
  }
}
