const { load, assertEqual } = require("./harness");

const net = load("services/NetworkPrefs.js");
assertEqual(net.sanitizeUuid("not-a-uuid"), "", "sanitizeUuid drops junk");
assertEqual(
  net.sanitizeUuid("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"),
  "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
  "sanitizeUuid keeps a uuid",
);
assertEqual(net.sanitizeMetered("yes"), "yes", "sanitizeMetered keeps yes");
assertEqual(net.sanitizeMetered("maybe"), "", "sanitizeMetered drops maybe");
assertEqual(net.clampPriority(99), 99, "clampPriority keeps 99");
assertEqual(net.clampPriority(9999), 999, "clampPriority caps at 999");
assertEqual(net.sanitizeMac("random"), "random", "sanitizeMac keeps random");
assertEqual(net.sanitizeMac("spoof"), "", "sanitizeMac drops spoof");
assertEqual(net.sanitizeIpv4("192.168.1.10"), "192.168.1.10", "sanitizeIpv4 keeps an address");
assertEqual(net.sanitizeIpv4("999.1.1.1"), "", "sanitizeIpv4 drops a bad address");
assertEqual(net.sanitizePrefix(24), 24, "sanitizePrefix keeps 24");
assertEqual(net.sanitizeHotspotSsid("Cafe"), "Cafe", "sanitizeHotspotSsid keeps Cafe");
assertEqual(net.sanitizeHotspotSsid("bad\nssid"), "", "sanitizeHotspotSsid drops a newline");
assertEqual(
  net.sanitizeWgPath("/tmp/wg0.conf"),
  "/tmp/wg0.conf",
  "sanitizeWgPath keeps an absolute conf",
);
assertEqual(net.sanitizeWgPath("wg0.conf"), "", "sanitizeWgPath requires an absolute path");

assertEqual(
  net.argvFor("metered", { uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", value: "yes" }).join(" "),
  "metered aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee yes",
  "argvFor metered",
);
assertEqual(
  net.argvFor("priority", { uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", value: 10 }).join(" "),
  "priority aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee 10",
  "argvFor priority",
);
assertEqual(
  net.argvFor("mac", { uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", value: "random" }).join(" "),
  "mac aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee random",
  "argvFor mac",
);
assertEqual(
  net
    .argvFor("ipv4", {
      uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
      method: "manual",
      address: "10.0.0.8",
      prefix: 24,
      gateway: "10.0.0.1",
      dns: "1.1.1.1",
    })
    .join(" "),
  "ipv4 aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee manual 10.0.0.8 24 10.0.0.1 1.1.1.1",
  "argvFor static ipv4",
);
assertEqual(
  net.argvFor("ipv4", { uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", method: "auto" }).join(" "),
  "ipv4 aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee auto",
  "argvFor dhcp ipv4",
);
assertEqual(
  net.argvFor("wireguard-import", { path: "/home/x/wg0.conf" }).join(" "),
  "wireguard-import /home/x/wg0.conf",
  "argvFor wireguard-import",
);
assertEqual(
  net.argvFor("hotspot", { on: true, ssid: "Desk", password: "abcdefgh" }).join(" "),
  "hotspot on Desk abcdefgh",
  "argvFor hotspot on",
);
assertEqual(net.argvFor("hotspot", { on: false }).join(" "), "hotspot off", "argvFor hotspot off");
assertEqual(
  net.argvFor("metered", { uuid: "bad", value: "yes" }),
  null,
  "argvFor refuses a bad uuid",
);
