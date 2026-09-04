const { load, assert, assertEqual } = require("./harness");

const hw = load("services/Hardware.js");
assertEqual(hw.parseDmiField(" Framework "), "Framework", "parseDmiField trims");
assertEqual(hw.parseDmiField("To be filled by O.E.M."), "", "parseDmiField drops OEM filler");
assertEqual(hw.parseDmiField("none"), "", "parseDmiField drops none");
assertEqual(
  hw.parseDmiField("Desktop (AMD Ryzen AI Max 300 Series)"),
  "Desktop (AMD Ryzen AI Max 300 Series)",
  "parseDmiField keeps a product name",
);
assertEqual(hw.parseDmiField("evil\nname"), "", "parseDmiField rejects a newline");
assertEqual(hw.parseSystemStats("cpu1%\nmemory27.5GB / 125GB").cpu, "1%", "parseSystemStats cpu");
assertEqual(
  hw.parseSystemStats("cpu1%\nmemory27.5GB / 125GB").memory,
  "27.5GB / 125GB",
  "parseSystemStats memory",
);
assertEqual(
  hw.npuSummary({ npuIdentity: "AMD Strix Halo NPU" }),
  "AMD Strix Halo NPU",
  "npuSummary present",
);
assertEqual(hw.npuSummary({}), "", "npuSummary absent");
assertEqual(
  hw.machineSummary({ name: "Desktop", vendor: "Framework" }).indexOf("Framework") !== -1,
  true,
  "machineSummary names the vendor",
);
assert(
  hw.cpuSummary({ model: "Ryzen 7", cores: 8, threads: 16, maxMhz: 5000.4 }).indexOf("8 cores") !==
    -1,
  "cpuSummary names cores",
);
assert(
  hw.cpuSummary({ model: "Ryzen 7", cores: 8, threads: 16 }).indexOf("16 threads") !== -1,
  "cpuSummary names extra threads",
);
assert(
  hw.biosSummary({ vendor: "AMI", version: "1.2", uefi: true }).indexOf("UEFI") !== -1,
  "biosSummary names UEFI",
);
assertEqual(hw.thermalSummary({ temp: 42.26 }), "42.3 °C", "thermalSummary rounds tenths");
assertEqual(hw.thermalSummary({}), "", "thermalSummary empty without a temp");
assertEqual(
  hw.tpmSummary({ present: true, version: "2.0" }).indexOf("TPM 2.0") !== -1,
  true,
  "tpmSummary names the version",
);
assertEqual(hw.tpmSummary({ present: false }), "", "tpmSummary empty when absent");

const cpuPresent = "processor\t: 0\nmodel name\t: AMD RYZEN AI MAX+ 395 w/ Radeon 8060S\n";
assertEqual(
  hw.parseCpuIdentity(cpuPresent),
  "AMD RYZEN AI MAX+ 395 w/ Radeon 8060S",
  "parseCpuIdentity present",
);
assertEqual(
  hw.parseCpuIdentity("processor\t: 0\nvendor_id\t: AuthenticAMD\n"),
  "",
  "parseCpuIdentity absent",
);
assertEqual(hw.parseCpuIdentity("model name\t: --nproc\n"), "", "parseCpuIdentity rejects a flag");
assertEqual(
  hw.parseCpuIdentity("Hardware\t: Raspberry Pi 5\n"),
  "Raspberry Pi 5",
  "parseCpuIdentity ARM Hardware field",
);

const gpuLine =
  "c3:00.0 Display controller [0380]: Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics] [1002:1586] (rev c1)\n";
const npuLine =
  "c4:00.1 Signal processing controller [1180]: Advanced Micro Devices, Inc. [AMD] Strix/Krackan/Strix Halo Neural Processing Unit [1022:17f0] (rev 11)\n";
const ethLine =
  "c1:00.0 Ethernet controller [0200]: Intel Corporation Ethernet Controller [8086:125c]\n";
assertEqual(
  hw.parseGpuIdentity(gpuLine + npuLine),
  "Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics]",
  "parseGpuIdentity present",
);
assertEqual(hw.parseGpuIdentity(ethLine + npuLine), "", "parseGpuIdentity absent");
assertEqual(
  hw.parseGpuIdentity("00:00.0 VGA compatible controller: ../../etc/passwd [1002:0000]\n"),
  "",
  "parseGpuIdentity rejects a path",
);
assertEqual(
  hw.parseNpuIdentity(gpuLine + npuLine),
  "Advanced Micro Devices, Inc. [AMD] Strix/Krackan/Strix Halo Neural Processing Unit",
  "parseNpuIdentity present",
);
assertEqual(hw.parseNpuIdentity(gpuLine + ethLine), "", "parseNpuIdentity absent");
assertEqual(
  hw.parseNpuIdentity("00:00.0 Neural Processing Unit: -inject\n"),
  "",
  "parseNpuIdentity rejects a flag",
);
assertEqual(hw.parseHwIdentity("evil\nname"), "", "parseHwIdentity rejects a newline");

const mem = hw.parseMeminfo(
  "MemTotal:       16398384 kB\nMemAvailable:    8000000 kB\nMemFree:         1000000 kB\nSwapTotal:             0 kB\nSwapFree:              0 kB\n",
);
assertEqual(mem.total, 16398384 * 1024, "parseMeminfo total bytes");
const cpuinfo = hw.parseCpuinfo(
  "processor: 0\nvendor_id: GenuineIntel\nmodel name: Intel(R) Xeon(R) Processor\ncpu cores: 4\nphysical id: 0\nflags: vmx avx2\n\nprocessor: 1\nvendor_id: GenuineIntel\nmodel name: Intel(R) Xeon(R) Processor\ncpu cores: 4\nphysical id: 0\n",
);
assertEqual(cpuinfo.model, "Intel(R) Xeon(R) Processor", "parseCpuinfo reads the model");
assertEqual(cpuinfo.cores, 4, "parseCpuinfo uses cpu cores per socket");
const lscpu = hw.parseLscpu(
  "Architecture: x86_64\nCPU(s): 16\nOn-line CPU(s) list: 0-15\nVendor ID: AuthenticAMD\nModel name: AMD Ryzen 7\nThread(s) per core: 2\nCore(s) per socket: 8\nSocket(s): 1\nCPU MHz: 3800.000\nCPU max MHz: 5000.000\nL3 cache: 32 MiB\nFlags: fpu vme sse\nVirtualization: AMD-V\nHypervisor vendor: KVM\n",
);
assertEqual(lscpu.model, "AMD Ryzen 7", "parseLscpu reads the model");
assertEqual(lscpu.cores, 8, "parseLscpu multiplies cores per socket");
assertEqual(lscpu.threads, 16, "parseLscpu reads CPU(s) as threads");
assertEqual(lscpu.mhz, 3800, "parseLscpu reads CPU MHz");
assertEqual(lscpu.maxMhz, 5000, "parseLscpu reads CPU max MHz");
assert(lscpu.caches.indexOf("L3 32 MiB") !== -1, "parseLscpu formats L3 cache");
assertEqual(lscpu.flags.join(" "), "fpu vme sse", "parseLscpu splits flags");
assertEqual(lscpu.virtualization, "AMD-V", "parseLscpu reads virtualization");
assertEqual(lscpu.hypervisor, "KVM", "parseLscpu reads hypervisor vendor");
assertEqual(mem.available, 8000000 * 1024, "parseMeminfo available bytes");
assertEqual(mem.used, (16398384 - 8000000) * 1024, "parseMeminfo used bytes");
const memFallback = hw.parseMeminfo(
  "MemTotal: 4096 kB\nMemFree: 1000 kB\nCached: 500 kB\nBuffers: 200 kB\nSwapTotal: 1024 kB\nSwapFree: 24 kB\n",
);
assertEqual(
  memFallback.available,
  (1000 + 500 + 200) * 1024,
  "parseMeminfo falls back to free+cached+buffers",
);
assertEqual(memFallback.swapUsed, 1000 * 1024, "parseMeminfo swap used");
const pci = hw.parseLspci(
  '00:00.0 "Host bridge [0600]" "Intel Corporation [8086]" "Raptor Lake-P Host Bridge [a74f]"\n00:02.0 "VGA compatible controller [0300]" "Intel Corporation [8086]" "Raptor Lake-P Integrated Graphics [a7a0]"\n',
);
assertEqual(
  hw.pickChipset(pci).name.indexOf("Raptor") !== -1,
  true,
  "pickChipset finds the host bridge",
);
assertEqual(hw.pickGpus(pci).length, 1, "pickGpus finds the VGA device");
assertEqual(hw.memoryTypeName(26), "DDR4", "memoryTypeName maps DDR4");
assertEqual(hw.chassisTypeName(9), "Laptop", "chassisTypeName maps laptop");
assertEqual(
  hw.formatModuleSize(16 * 1024 * 1024 * 1024),
  "16 GB",
  "formatModuleSize uses whole gigabytes",
);
assertEqual(
  hw.formatModuleSize(1536 * 1024 * 1024),
  "1.5 GB",
  "formatModuleSize fractional gigabytes",
);
assertEqual(hw.formatModuleSize(512 * 1024 * 1024), "512 MB", "formatModuleSize megabytes");
assertEqual(hw.formatModuleSize(0), "", "formatModuleSize empty for zero");
assert(
  hw.virtSummary({ hypervisor: "KVM", kvm: true }).indexOf("Running on KVM") !== -1,
  "virtSummary names the hypervisor",
);
assert(
  hw.virtSummary({ guest: true }).indexOf("guest") !== -1,
  "virtSummary guest without hypervisor",
);
assertEqual(hw.virtSummary({}), "", "virtSummary empty");
assertEqual(
  hw.notableFlags(["vmx", "avx2", "nope"]).join(","),
  "VT-x,AVX2",
  "notableFlags maps known CPU flags",
);
assertEqual(hw.notableFlags(["svm", "SVM"]).join(","), "AMD-V", "notableFlags dedupes labels");
assert(
  hw.batterySummary({ status: "Charging", capacity: 80, technology: "Li-ion" }).indexOf("80%") !==
    -1,
  "batterySummary names capacity",
);
assertEqual(
  hw
    .boardSummary({ vendor: "Framework", name: "Mainboard", version: "A7" })
    .indexOf("Version A7") !== -1,
  true,
  "boardSummary names the version",
);
assert(
  hw
    .gpuSummary({ vendor: "AMD", name: "Strix Halo", driver: "amdgpu", pciId: "1002:1586" })
    .indexOf("Driver amdgpu") !== -1,
  "gpuSummary names the driver",
);
assert(
  hw
    .nicSummary({ iface: "wlan0", name: "Wi-Fi", wireless: true, speed: 1200 })
    .indexOf("Wireless") !== -1,
  "nicSummary marks wireless",
);
assert(
  hw.moduleSummary({ size: 16 * 1024 * 1024 * 1024, typeCode: 26, speed: 5600 }).indexOf("DDR4") !==
    -1,
  "moduleSummary maps typeCode",
);
assertEqual(
  hw.pickNpus([
    { className: "Signal processing controller", name: "Strix Halo Neural Processing Unit" },
  ]).length,
  1,
  "pickNpus matches neural processing",
);
assertEqual(
  hw.pickNpus([{ className: "Ethernet controller", name: "I225" }]).length,
  0,
  "pickNpus skips NICs",
);
assert(
  hw
    .chipsetSummary({ vendor: "Intel", name: "Raptor", role: "Host bridge", southbridge: "LPC" })
    .indexOf("Southbridge LPC") !== -1,
  "chipsetSummary names the southbridge",
);
assertEqual(hw.parseKeyValues("A: 1\nB = 2", "=").B, "2", "parseKeyValues uses a custom delimiter");
const hwNorm = hw.normalize({
  machine: { chassis: 9, vendor: "Framework", name: "Laptop" },
  gpus: [{ name: "" }],
  nics: [{ iface: "wlan0" }],
});
assertEqual(hwNorm.machine.chassis, "Laptop", "normalize maps a chassis type code");
assertEqual(hwNorm.gpus.length, 0, "normalize drops a nameless GPU");
assertEqual(hwNorm.nics[0].iface, "wlan0", "normalize keeps a NIC iface");
assertEqual(hw.cleanText("To Be Filled By O.E.M."), "", "cleanText drops OEM filler");
assertEqual(hw.memoryFormName(8), "DIMM", "memoryFormName maps DIMM");
assertEqual(hw.memoryFormName(13), "SODIMM", "memoryFormName maps SODIMM");
assertEqual(hw.memoryFormName(99), "", "memoryFormName unknown code is empty");
assertEqual(hw.decodeMemorySize(0, 0), 0, "decodeMemorySize empty slot is zero");
assertEqual(hw.decodeMemorySize(0xffff, 0), 0, "decodeMemorySize unknown size is zero");
assertEqual(
  hw.decodeMemorySize(0x7fff, 8192),
  8192 * 1024 * 1024,
  "decodeMemorySize uses extended MiB",
);
assertEqual(hw.decodeMemorySize(0x7fff, 0), 0, "decodeMemorySize extended with no MiB is zero");
assertEqual(
  hw.decodeMemorySize(16, 0),
  16 * 1024 * 1024,
  "decodeMemorySize treats low words as MiB",
);
