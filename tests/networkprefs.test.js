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

const U = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
const as = (op) => JSON.stringify(op);
assertEqual(
  as(net.opFor("metered", { uuid: U, value: "yes" })),
  as({ name: "wifi.metered", args: { uuid: U, value: "yes" } }),
  "opFor metered",
);
assertEqual(
  as(net.opFor("priority", { uuid: U, value: 10 })),
  as({ name: "wifi.priority", args: { uuid: U, value: 10 } }),
  "opFor priority",
);
assertEqual(
  as(net.opFor("mac", { uuid: U, value: "random" })),
  as({ name: "wifi.mac", args: { uuid: U, value: "random" } }),
  "opFor mac",
);
assertEqual(
  as(
    net.opFor("ipv4", {
      uuid: U,
      method: "manual",
      address: "10.0.0.8",
      prefix: 24,
      gateway: "10.0.0.1",
      dns: "1.1.1.1",
    }),
  ),
  as({
    name: "wifi.ipv4-manual",
    args: { uuid: U, address: "10.0.0.8", prefix: 24, gateway: "10.0.0.1", dns: "1.1.1.1" },
  }),
  "opFor static ipv4",
);
assertEqual(
  as(net.opFor("ipv4", { uuid: U, method: "auto" })),
  as({ name: "wifi.ipv4-auto", args: { uuid: U } }),
  "opFor dhcp ipv4",
);
assertEqual(
  as(net.opFor("wireguard-import", { path: "/home/x/wg0.conf" })),
  as({ name: "wifi.wireguard-import", args: { path: "/home/x/wg0.conf" } }),
  "opFor wireguard-import",
);
assertEqual(
  as(net.opFor("hotspot", { on: true, ssid: "Desk", password: "abcdefgh" })),
  as({ name: "wifi.hotspot-on", args: { ssid: "Desk", password: "abcdefgh" } }),
  "opFor hotspot on",
);
assertEqual(
  as(net.opFor("hotspot", { on: false })),
  as({ name: "wifi.hotspot-off", args: {} }),
  "opFor hotspot off",
);
assertEqual(net.opFor("metered", { uuid: "bad", value: "yes" }), null, "opFor refuses a bad uuid");
