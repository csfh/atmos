function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {}
}

function asArray(value) {
  return Array.isArray(value) ? value : []
}

function cleanText(value, max) {
  var s = String(value == null ? "" : value).replace(/[\r\n\t]+/g, " ").replace(/^\s+|\s+$/g, "")
  if (!s) return ""
  var lower = s.toLowerCase()
  if (lower === "not specified" || lower === "unknown" || lower === "none" ||
      lower === "to be filled by o.e.m." || lower === "default string" ||
      lower === "fill by oem" || lower === "system product name" ||
      lower === "system manufacturer" || lower === "oem" ||
      lower === "1234567890" || /^0+$/.test(s) || /^f+$/i.test(s))
    return ""
  var limit = max || 240
  if (s.length > limit) s = s.substring(0, limit)
  return s
}

function intOrZero(value) {
  var n = Math.round(Number(value))
  return isFinite(n) && n > 0 ? n : 0
}

function joinParts(parts, sep) {
  var out = []
  var list = Array.isArray(parts) ? parts : []
  for (var i = 0; i < list.length; i++) {
    var bit = cleanText(list[i], 200)
    if (bit) out.push(bit)
  }
  return out.join(sep == null ? ". " : sep)
}

function sentence(parts) {
  var text = joinParts(parts, ". ")
  if (!text) return ""
  return /[.!?]$/.test(text) ? text : text + "."
}

function emptyHardware() {
  return {
    machine: { vendor: "", name: "", family: "", version: "", serial: "", sku: "", chassis: "" },
    board: { vendor: "", name: "", version: "", serial: "" },
    chipset: { name: "", vendor: "", pciId: "", role: "", southbridge: "" },
    bios: { vendor: "", version: "", date: "", uefi: false },
    cpu: {
      model: "", vendor: "", arch: "", cores: 0, threads: 0, sockets: 1,
      mhz: 0, maxMhz: 0, caches: "", flags: [], virtualization: "", hypervisor: ""
    },
    memory: { total: 0, used: 0, available: 0, swapTotal: 0, swapUsed: 0, modules: [] },
    gpus: [],
    npus: [],
    nics: [],
    audio: [],
    usb: [],
    batteries: [],
    thermals: [],
    tpm: { present: false, version: "" },
    secureBoot: { available: false, enabled: false },
    virtualization: { hypervisor: "", guest: false, kvm: false },
    pci: []
  }
}

function parseKeyValues(raw, sep) {
  var out = {}
  var lines = String(raw || "").replace(/\r/g, "").split("\n")
  var delim = sep == null ? ":" : sep
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i]
    var cut = line.indexOf(delim)
    if (cut < 1) continue
    var key = line.substring(0, cut).replace(/^\s+|\s+$/g, "")
    var value = line.substring(cut + delim.length).replace(/^\s+|\s+$/g, "")
    if (key) out[key] = value
  }
  return out
}

function kbToBytes(value) {
  var n = Number(String(value || "").replace(/[^0-9.].*/g, ""))
  if (!isFinite(n) || n < 0) return 0
  return Math.round(n * 1024)
}

function parseMeminfo(raw) {
  var map = parseKeyValues(raw)
  var total = kbToBytes(map.MemTotal)
  var available = kbToBytes(map.MemAvailable)
  if (!available) available = kbToBytes(map.MemFree) + kbToBytes(map.Cached) + kbToBytes(map.Buffers)
  if (available > total) available = total
  var used = total > available ? total - available : 0
  var swapTotal = kbToBytes(map.SwapTotal)
  var swapFree = kbToBytes(map.SwapFree)
  var swapUsed = swapTotal > swapFree ? swapTotal - swapFree : 0
  return {
    total: total,
    used: used,
    available: available,
    swapTotal: swapTotal,
    swapUsed: swapUsed
  }
}

function parseCpuinfo(raw) {
  var text = String(raw || "").replace(/\r/g, "")
  var blocks = text.split(/\n\n+/)
  var model = ""
  var vendor = ""
  var flags = []
  var mhz = 0
  var physical = {}
  var threads = 0
  for (var i = 0; i < blocks.length; i++) {
    var map = parseKeyValues(blocks[i])
    if (!map["model name"] && !map.processor && map.processor !== "0") continue
    threads += 1
    if (!model) model = cleanText(map["model name"] || map.Hardware || map.model, 160)
    if (!vendor) vendor = cleanText(map.vendor_id || map.Hardware, 80)
    if (!flags.length && map.flags)
      flags = String(map.flags).split(/\s+/).filter(function(f) { return !!f })
    var hz = Number(map["cpu MHz"])
    if (isFinite(hz) && hz > mhz) mhz = hz
    var phys = String(map["physical id"] || "0")
    var cores = intOrZero(map["cpu cores"])
    if (!physical[phys] || cores > physical[phys]) physical[phys] = cores || 1
  }
  var sockets = Object.keys(physical).length
  var cores = 0
  var keys = Object.keys(physical)
  for (var k = 0; k < keys.length; k++) cores += physical[keys[k]] || 0
  if (!cores) cores = threads
  if (!sockets) sockets = 1
  return {
    model: model,
    vendor: vendor,
    cores: cores,
    threads: threads,
    sockets: sockets,
    mhz: Math.round(mhz),
    flags: flags
  }
}

function parseLscpu(raw) {
  var map = parseKeyValues(raw)
  function num(key) {
    return intOrZero(String(map[key] || "").replace(/[^0-9.].*/g, ""))
  }
  var flags = []
  var flagText = map.Flags || map.flags || ""
  if (flagText) flags = flagText.split(/\s+/).filter(function(f) { return !!f })
  return {
    model: cleanText(map["Model name"] || map["Model Name"], 160),
    vendor: cleanText(map["Vendor ID"] || map["BIOS Vendor ID"], 80),
    arch: cleanText(map.Architecture, 40),
    cores: num("Core(s) per socket") * (num("Socket(s)") || 1) || num("CPU(s)"),
    threads: num("CPU(s)"),
    sockets: num("Socket(s)") || 1,
    mhz: num("CPU MHz") || num("CPU max MHz"),
    maxMhz: num("CPU max MHz"),
    caches: joinParts([
      map["L1d cache"] ? "L1d " + map["L1d cache"] : "",
      map["L1i cache"] ? "L1i " + map["L1i cache"] : "",
      map["L2 cache"] ? "L2 " + map["L2 cache"] : "",
      map["L3 cache"] ? "L3 " + map["L3 cache"] : ""
    ], ", "),
    flags: flags,
    virtualization: cleanText(map.Virtualization, 40),
    hypervisor: cleanText(map["Hypervisor vendor"], 40)
  }
}

var PCI_CLASS_HOST = "0600"
var PCI_CLASS_ISA = "0601"
var PCI_CLASS_VGA = "0300"
var PCI_CLASS_3D = "0302"
var PCI_CLASS_DISPLAY = "0380"

function pciClassCode(value) {
  var s = String(value || "").toLowerCase().replace(/^0x/, "").replace(/[^0-9a-f]/g, "")
  if (s.length >= 4) return s.substring(0, 4)
  return s
}

function parseLspciLine(line) {
  var text = String(line || "")
  if (!text) return null
  var mm = text.match(/^(\S+)\s+"([^"]*)"\s+"([^"]*)"\s+"([^"]*)"/)
  if (mm) {
    var cls = mm[2]
    var ven = mm[3]
    var dev = mm[4]
    var classId = (cls.match(/\[([0-9a-fA-F]{4})\]/) || [])[1] || ""
    var vendorId = (ven.match(/\[([0-9a-fA-F]{4})\]/) || [])[1] || ""
    var deviceId = (dev.match(/\[([0-9a-fA-F]{4})\]/) || [])[1] || ""
    return {
      slot: mm[1],
      className: cleanText(cls.replace(/\s*\[[0-9a-fA-F]{4}\]\s*$/, ""), 80),
      classId: classId.toLowerCase(),
      vendor: cleanText(ven.replace(/\s*\[[0-9a-fA-F]{4}\]\s*$/, ""), 80),
      name: cleanText(dev.replace(/\s*\[[0-9a-fA-F]{4,8}\]\s*$/, ""), 160),
      pciId: vendorId && deviceId ? vendorId.toLowerCase() + ":" + deviceId.toLowerCase() : ""
    }
  }
  var classic = text.match(/^(\S+)\s+([^:]+):\s+(.*)$/)
  if (!classic) return null
  var ids = classic[3].match(/\[([0-9a-fA-F]{4}):([0-9a-fA-F]{4})\]/)
  return {
    slot: classic[1],
    className: cleanText(classic[2], 80),
    classId: "",
    vendor: "",
    name: cleanText(classic[3].replace(/\s*\[[0-9a-fA-F]{4}:[0-9a-fA-F]{4}\]\s*$/, ""), 160),
    pciId: ids ? ids[1].toLowerCase() + ":" + ids[2].toLowerCase() : ""
  }
}

function parseLspci(raw) {
  var devices = []
  var lines = String(raw || "").replace(/\r/g, "").split("\n")
  for (var i = 0; i < lines.length; i++) {
    var row = parseLspciLine(lines[i])
    if (row) devices.push(row)
  }
  return devices
}

function isGpuClass(classId, className) {
  var id = pciClassCode(classId)
  if (id === PCI_CLASS_VGA || id === PCI_CLASS_3D || id === PCI_CLASS_DISPLAY) return true
  var name = String(className || "").toLowerCase()
  return name.indexOf("vga") !== -1 || name.indexOf("3d controller") !== -1 ||
    name.indexOf("display controller") !== -1
}

function isHostBridge(classId, className) {
  if (pciClassCode(classId) === PCI_CLASS_HOST) return true
  return String(className || "").toLowerCase().indexOf("host bridge") !== -1
}

function isIsaBridge(classId, className) {
  if (pciClassCode(classId) === PCI_CLASS_ISA) return true
  var name = String(className || "").toLowerCase()
  return name.indexOf("isa bridge") !== -1 || name.indexOf("lpc") !== -1
}

function pickChipset(devices) {
  var list = asArray(devices)
  var host = null
  var south = null
  for (var i = 0; i < list.length; i++) {
    var row = list[i]
    if (!host && isHostBridge(row.classId, row.className)) host = row
    if (!south && isIsaBridge(row.classId, row.className)) south = row
  }
  if (!host && !south) return { name: "", vendor: "", pciId: "", role: "", southbridge: "" }
  var primary = host || south
  return {
    name: cleanText(primary && primary.name, 160),
    vendor: cleanText(primary && primary.vendor, 80),
    pciId: cleanText(primary && primary.pciId, 20),
    role: host ? "Host bridge" : "ISA bridge",
    southbridge: host && south && south.name && south.name !== (host && host.name)
      ? cleanText(south.name, 160)
      : ""
  }
}

function pickGpus(devices) {
  var list = asArray(devices)
  var out = []
  for (var i = 0; i < list.length; i++) {
    var row = list[i]
    if (!isGpuClass(row.classId, row.className)) continue
    out.push({
      name: cleanText(row.name, 160),
      vendor: cleanText(row.vendor, 80),
      pciId: cleanText(row.pciId, 20),
      driver: "",
      vram: 0
    })
  }
  return out
}

function isNpuDevice(row) {
  var blob = String((row && row.className) || "") + " " + String((row && row.name) || "") + " " + String((row && row.vendor) || "")
  blob = blob.toLowerCase()
  return blob.indexOf("neural processing") !== -1 || blob.indexOf("npu") !== -1 ||
    blob.indexOf("xdna") !== -1 || blob.indexOf("tpu") !== -1 ||
    blob.indexOf("ai accelerator") !== -1 || blob.indexOf("habana") !== -1 ||
    blob.indexOf("hailo") !== -1 || blob.indexOf("coral") !== -1
}

function pickNpus(devices) {
  var list = asArray(devices)
  var out = []
  for (var i = 0; i < list.length; i++) {
    var row = list[i]
    if (!isNpuDevice(row)) continue
    out.push({
      name: cleanText(row.name, 160),
      vendor: cleanText(row.vendor, 80),
      pciId: cleanText(row.pciId, 20)
    })
  }
  return out
}

var MEMORY_TYPES = {
  1: "Other", 2: "Unknown", 3: "DRAM", 4: "EDRAM", 5: "VRAM", 6: "SRAM",
  7: "RAM", 8: "ROM", 9: "Flash", 10: "EEPROM", 11: "FEPROM", 12: "EPROM",
  13: "CDRAM", 14: "3DRAM", 15: "SDRAM", 16: "SGRAM", 17: "RDRAM",
  18: "DDR", 19: "DDR2", 20: "DDR2 FB-DIMM", 24: "DDR3", 25: "FBD2",
  26: "DDR4", 27: "LPDDR", 28: "LPDDR2", 29: "LPDDR3", 30: "LPDDR4",
  31: "Logical", 32: "HBM", 33: "HBM2", 34: "DDR5", 35: "LPDDR5",
  36: "LPDDR5X", 37: "HBM3"
}

var MEMORY_FORMS = {
  1: "Other", 2: "Unknown", 3: "SIMM", 4: "SIP", 8: "DIMM", 9: "TSOP",
  13: "SODIMM", 15: "FB-DIMM", 16: "Die"
}

var CHASSIS_TYPES = {
  1: "Other", 2: "Unknown", 3: "Desktop", 4: "Low-profile desktop",
  5: "Pizza box", 6: "Mini tower", 7: "Tower", 8: "Portable", 9: "Laptop",
  10: "Notebook", 11: "Handheld", 12: "Docking station", 13: "All-in-one",
  14: "Sub-notebook", 15: "Space-saving", 16: "Lunch box", 17: "Main server",
  18: "Expansion", 19: "Sub-chassis", 20: "Bus expansion", 21: "Peripheral",
  22: "RAID", 23: "Rack mount", 24: "Sealed-case PC", 25: "Multi-system",
  26: "Compact PCI", 27: "Advanced TCA", 28: "Blade", 29: "Blade enclosure",
  30: "Tablet", 31: "Convertible", 32: "Detachable", 33: "IoT gateway",
  34: "Embedded PC", 35: "Mini PC", 36: "Stick PC"
}

function memoryTypeName(code) {
  var n = intOrZero(code)
  return MEMORY_TYPES[n] || ""
}

function memoryFormName(code) {
  var n = intOrZero(code)
  return MEMORY_FORMS[n] || ""
}

function chassisTypeName(code) {
  var n = intOrZero(code)
  if (!n && typeof code === "string" && code && !/^[0-9]+$/.test(code))
    return cleanText(code, 40)
  return CHASSIS_TYPES[n] || ""
}

// SMBIOS type 17 size: bits 0-14 are the value. Bit 15 means kilobytes.
// 0 = empty slot, 0xFFFF = unknown, 0x7FFF = use the extended size (MiB).
function decodeMemorySize(word, extendedMib) {
  var size = Number(word)
  if (!isFinite(size) || size === 0) return 0
  if (size === 0xFFFF) return 0
  if (size === 0x7FFF) {
    var ext = Number(extendedMib)
    if (!isFinite(ext) || ext <= 0) return 0
    return Math.round(ext * 1024 * 1024)
  }
  var value = size & 0x7FFF
  if (size & 0x8000) return value * 1024
  return value * 1024 * 1024
}

function normalizeModule(item) {
  var row = asObject(item)
  return {
    locator: cleanText(row.locator, 40),
    bank: cleanText(row.bank, 40),
    size: intOrZero(row.size),
    type: cleanText(row.type || memoryTypeName(row.typeCode), 20),
    form: cleanText(row.form || memoryFormName(row.formCode), 20),
    speed: intOrZero(row.speed),
    manufacturer: cleanText(row.manufacturer, 80),
    part: cleanText(row.part, 80),
    rank: intOrZero(row.rank)
  }
}

function normalizePci(item) {
  var row = asObject(item)
  return {
    slot: cleanText(row.slot, 20),
    className: cleanText(row.className, 80),
    classId: cleanText(row.classId, 8),
    vendor: cleanText(row.vendor, 80),
    name: cleanText(row.name, 160),
    pciId: cleanText(row.pciId, 20)
  }
}

function normalizeGpu(item) {
  var row = asObject(item)
  return {
    name: cleanText(row.name, 160),
    vendor: cleanText(row.vendor, 80),
    pciId: cleanText(row.pciId, 20),
    driver: cleanText(row.driver, 40),
    vram: intOrZero(row.vram)
  }
}

function normalizeNic(item) {
  var row = asObject(item)
  return {
    name: cleanText(row.name || row.iface, 80),
    iface: cleanText(row.iface, 32),
    mac: cleanText(row.mac, 20),
    driver: cleanText(row.driver, 40),
    speed: intOrZero(row.speed),
    wireless: row.wireless === true
  }
}

function normalizeAudio(item) {
  var row = asObject(item)
  return {
    name: cleanText(row.name, 160),
    driver: cleanText(row.driver, 40),
    id: cleanText(row.id, 20)
  }
}

function normalizeUsb(item) {
  var row = asObject(item)
  return {
    name: cleanText(row.name || row.product, 160),
    vendor: cleanText(row.vendor, 80),
    product: cleanText(row.product, 80),
    speed: cleanText(row.speed, 20)
  }
}

function normalizeBattery(item) {
  var row = asObject(item)
  return {
    name: cleanText(row.name, 80),
    status: cleanText(row.status, 20),
    capacity: intOrZero(row.capacity),
    technology: cleanText(row.technology, 20),
    energy: intOrZero(row.energy),
    voltage: intOrZero(row.voltage)
  }
}

function normalizeThermal(item) {
  var row = asObject(item)
  return {
    name: cleanText(row.name || row.type, 80),
    type: cleanText(row.type, 40),
    temp: Number(row.temp)
  }
}

function normalize(raw) {
  var data = asObject(raw)
  var out = emptyHardware()
  var machine = asObject(data.machine)
  out.machine.vendor = cleanText(machine.vendor, 80)
  out.machine.name = cleanText(machine.name, 160)
  out.machine.family = cleanText(machine.family, 80)
  out.machine.version = cleanText(machine.version, 80)
  out.machine.serial = cleanText(machine.serial, 80)
  out.machine.sku = cleanText(machine.sku, 80)
  out.machine.chassis = chassisTypeName(machine.chassis) || cleanText(machine.chassis, 40)

  var board = asObject(data.board)
  out.board.vendor = cleanText(board.vendor, 80)
  out.board.name = cleanText(board.name, 160)
  out.board.version = cleanText(board.version, 40)
  out.board.serial = cleanText(board.serial, 80)

  var chip = asObject(data.chipset)
  out.chipset.name = cleanText(chip.name, 160)
  out.chipset.vendor = cleanText(chip.vendor, 80)
  out.chipset.pciId = cleanText(chip.pciId, 20)
  out.chipset.role = cleanText(chip.role, 40)
  out.chipset.southbridge = cleanText(chip.southbridge, 160)

  var bios = asObject(data.bios)
  out.bios.vendor = cleanText(bios.vendor, 80)
  out.bios.version = cleanText(bios.version, 80)
  out.bios.date = cleanText(bios.date, 40)
  out.bios.uefi = bios.uefi === true

  var cpu = asObject(data.cpu)
  out.cpu.model = cleanText(cpu.model, 160)
  out.cpu.vendor = cleanText(cpu.vendor, 80)
  out.cpu.arch = cleanText(cpu.arch, 40)
  out.cpu.cores = intOrZero(cpu.cores)
  out.cpu.threads = intOrZero(cpu.threads)
  out.cpu.sockets = intOrZero(cpu.sockets) || 1
  out.cpu.mhz = intOrZero(cpu.mhz)
  out.cpu.maxMhz = intOrZero(cpu.maxMhz)
  out.cpu.caches = cleanText(cpu.caches, 200)
  out.cpu.flags = asArray(cpu.flags).map(function(f) { return cleanText(f, 32) }).filter(function(f) { return !!f })
  out.cpu.virtualization = cleanText(cpu.virtualization, 40)
  out.cpu.hypervisor = cleanText(cpu.hypervisor, 40)

  var memory = asObject(data.memory)
  out.memory.total = intOrZero(memory.total)
  out.memory.used = intOrZero(memory.used)
  out.memory.available = intOrZero(memory.available)
  out.memory.swapTotal = intOrZero(memory.swapTotal)
  out.memory.swapUsed = intOrZero(memory.swapUsed)
  out.memory.modules = asArray(memory.modules).map(normalizeModule).filter(function(m) { return m.size > 0 || m.locator })

  out.gpus = asArray(data.gpus).map(normalizeGpu).filter(function(g) { return g.name || g.pciId })
  out.npus = asArray(data.npus).map(function(item) {
    var row = asObject(item)
    return {
      name: cleanText(row.name, 160),
      vendor: cleanText(row.vendor, 80),
      pciId: cleanText(row.pciId, 20)
    }
  }).filter(function(n) { return n.name || n.pciId })
  out.nics = asArray(data.nics).map(normalizeNic).filter(function(n) { return n.iface || n.name })
  out.audio = asArray(data.audio).map(normalizeAudio).filter(function(a) { return a.name })
  out.usb = asArray(data.usb).map(normalizeUsb).filter(function(u) { return u.name || u.product })
  out.batteries = asArray(data.batteries).map(normalizeBattery).filter(function(b) { return b.name || b.capacity })
  out.thermals = asArray(data.thermals).map(normalizeThermal).filter(function(t) {
    return t.name && isFinite(t.temp)
  })

  var tpm = asObject(data.tpm)
  out.tpm.present = tpm.present === true
  out.tpm.version = cleanText(tpm.version, 12)

  var sb = asObject(data.secureBoot)
  out.secureBoot.available = sb.available === true
  out.secureBoot.enabled = sb.enabled === true

  var virt = asObject(data.virtualization)
  out.virtualization.hypervisor = cleanText(virt.hypervisor, 40)
  out.virtualization.guest = virt.guest === true || !!out.virtualization.hypervisor
  out.virtualization.kvm = virt.kvm === true

  out.pci = asArray(data.pci).map(normalizePci).filter(function(p) { return p.name || p.className })
  return out
}

function notableFlags(flags) {
  var want = {
    vmx: "VT-x", svm: "AMD-V", hypervisor: "hypervisor",
    aes: "AES", avx: "AVX", avx2: "AVX2", avx512f: "AVX-512",
    nx: "NX", lm: "64-bit", sse4_2: "SSE4.2"
  }
  var list = asArray(flags)
  var out = []
  var seen = {}
  for (var i = 0; i < list.length; i++) {
    var key = String(list[i] || "").toLowerCase()
    var label = want[key]
    if (!label || seen[label]) continue
    seen[label] = true
    out.push(label)
  }
  return out
}

function cpuSummary(cpu) {
  var row = asObject(cpu)
  var bits = []
  if (row.model) bits.push(row.model)
  var topo = []
  if (row.cores) topo.push(row.cores + (row.cores === 1 ? " core" : " cores"))
  if (row.threads && row.threads !== row.cores)
    topo.push(row.threads + " threads")
  if (row.sockets > 1) topo.push(row.sockets + " sockets")
  if (row.arch) topo.push(row.arch)
  if (topo.length) bits.push(topo.join(", "))
  var clock = ""
  if (row.maxMhz) clock = Math.round(row.maxMhz) + " MHz max"
  else if (row.mhz) clock = Math.round(row.mhz) + " MHz"
  if (clock) bits.push(clock)
  if (row.caches) bits.push(row.caches)
  var feats = notableFlags(row.flags)
  if (row.virtualization) feats.unshift(row.virtualization)
  if (feats.length) bits.push(feats.join(", "))
  return sentence(bits)
}

function machineSummary(machine) {
  var row = asObject(machine)
  return sentence([
    joinParts([row.vendor, row.name], " "),
    row.family && row.family !== row.name ? row.family : "",
    row.chassis,
    row.version ? "Version " + row.version : ""
  ])
}

function boardSummary(board) {
  var row = asObject(board)
  return sentence([
    joinParts([row.vendor, row.name], " "),
    row.version ? "Version " + row.version : ""
  ])
}

function chipsetSummary(chipset) {
  var row = asObject(chipset)
  return sentence([
    joinParts([row.vendor, row.name], " "),
    row.role,
    row.southbridge ? "Southbridge " + row.southbridge : "",
    row.pciId
  ])
}

function biosSummary(bios) {
  var row = asObject(bios)
  return sentence([
    joinParts([row.vendor, row.version], " "),
    row.date,
    row.uefi ? "UEFI" : ""
  ])
}

function formatModuleSize(bytes) {
  var n = Number(bytes)
  if (!isFinite(n) || n <= 0) return ""
  var gb = n / (1024 * 1024 * 1024)
  if (gb >= 1) {
    if (Math.abs(gb - Math.round(gb)) < 0.05) return Math.round(gb) + " GB"
    return gb.toFixed(1) + " GB"
  }
  return Math.round(n / (1024 * 1024)) + " MB"
}

function moduleSummary(item) {
  var row = normalizeModule(item)
  var bits = []
  if (row.size) bits.push(formatModuleSize(row.size))
  if (row.type) bits.push(row.type)
  if (row.form) bits.push(row.form)
  if (row.speed) bits.push(row.speed + " MT/s")
  if (row.manufacturer) bits.push(row.manufacturer)
  if (row.part) bits.push(row.part)
  if (row.rank) bits.push("rank " + row.rank)
  return bits.join(" · ")
}

function gpuSummary(item) {
  var row = normalizeGpu(item)
  return sentence([
    joinParts([row.vendor, row.name], " "),
    row.driver ? "Driver " + row.driver : "",
    row.pciId
  ])
}

function nicSummary(item) {
  var row = normalizeNic(item)
  var bits = []
  if (row.name && row.name !== row.iface) bits.push(row.name)
  if (row.wireless) bits.push("Wireless")
  if (row.driver) bits.push("Driver " + row.driver)
  if (row.speed > 0) bits.push(row.speed + " Mb/s")
  if (row.mac) bits.push(row.mac)
  return sentence(bits)
}

function batterySummary(item) {
  var row = normalizeBattery(item)
  var bits = []
  if (row.status) bits.push(row.status)
  if (row.capacity) bits.push(row.capacity + "%")
  if (row.technology) bits.push(row.technology)
  return sentence(bits)
}

function thermalSummary(item) {
  var row = normalizeThermal(item)
  if (!isFinite(row.temp)) return ""
  return (Math.round(row.temp * 10) / 10) + " °C"
}

function tpmSummary(tpm) {
  var row = asObject(tpm)
  if (!row.present) return ""
  return sentence([row.version ? "TPM " + row.version : "TPM is present"])
}

function virtSummary(virt) {
  var row = asObject(virt)
  var bits = []
  if (row.hypervisor) bits.push("Running on " + row.hypervisor)
  else if (row.guest) bits.push("This machine is a guest")
  if (row.kvm) bits.push("KVM is available for guests")
  return sentence(bits)
}

function npuSummary(item) {
  var row = asObject(item)
  if (row.npuIdentity) return cleanText(row.npuIdentity, 160)
  return sentence([joinParts([row.vendor, row.name], " "), row.pciId])
}

function parseDmiField(raw) {
  var source = String(raw || "")
  if (source.indexOf("\n") !== -1 || source.indexOf("\r") !== -1) return ""
  return cleanText(source, 160)
}

function parseHwIdentity(raw) {
  var source = String(raw || "")
  if (source.indexOf("\n") !== -1 || source.indexOf("\r") !== -1) return ""
  var s = cleanText(source, 160)
  if (!s) return ""
  if (s.indexOf("..") !== -1) return ""
  if (s.charAt(0) === "-" || s.charAt(0) === "/") return ""
  return s
}

function parseGpuIdentity(lspciText) {
  var gpus = pickGpus(parseLspci(lspciText))
  if (!gpus.length) return ""
  return parseHwIdentity(String(gpus[0].name || "").replace(/\s*\[[0-9a-fA-F]{4}:[0-9a-fA-F]{4}\].*$/, ""))
}

function parseNpuIdentity(lspciText) {
  var npus = pickNpus(parseLspci(lspciText))
  if (!npus.length) return ""
  return parseHwIdentity(String(npus[0].name || "").replace(/\s*\[[0-9a-fA-F]{4}:[0-9a-fA-F]{4}\].*$/, ""))
}

function parseCpuIdentity(cpuinfo) {
  var parsed = parseCpuinfo(cpuinfo)
  if (parsed.model) return parseHwIdentity(parsed.model)
  var map = parseKeyValues(cpuinfo)
  return parseHwIdentity(map.Hardware || map["cpu model"] || "")
}

function parseSystemStats(text) {
  var out = { cpu: "", memory: "" }
  var lines = String(text || "").split("\n")
  for (var i = 0; i < lines.length; i++) {
    var line = String(lines[i] || "").replace(/^\s+|\s+$/g, "")
    if (line.indexOf("cpu") === 0) out.cpu = line.slice(3).replace(/^\s+/, "")
    else if (line.indexOf("memory") === 0) out.memory = line.slice(6).replace(/^\s+/, "")
  }
  return out
}
