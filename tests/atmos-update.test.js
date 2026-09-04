const { load, assertEqual } = require("./harness");

const atmosUp = load("services/AtmosUpdate.js");
assertEqual(
  atmosUp.parseCheckOutput(
    "status behind\nchannel alpha\nlocal abcdef0\nremote abcdef1\nshort abcdef0\n",
  ).status,
  "behind",
  "parseCheckOutput behind",
);
assertEqual(
  atmosUp.parseCheckOutput("status behind\nchannel alpha\n").channel,
  "alpha",
  "parseCheckOutput channel",
);
assertEqual(
  atmosUp.parseCheckOutput("status current\nshort abcdef0\n").status,
  "current",
  "parseCheckOutput current",
);
assertEqual(atmosUp.parseChannel("alpha"), "alpha", "parseChannel accepts alpha");
assertEqual(atmosUp.parseChannel("main"), "", "parseChannel rejects main");
assertEqual(atmosUp.parseSha("--help"), "", "parseSha rejects a flag");
assertEqual(
  atmosUp.parseCheckOutput("status behind\nsummary rm -rf /\n").summary,
  "A newer Atmos is on alpha.",
  "parseCheckOutput drops a junk summary",
);
assertEqual(atmosUp.parseSha("AbCdEf01"), "abcdef01", "parseSha lowercases a hex sha");
assertEqual(atmosUp.parseSha("xyz"), "", "parseSha rejects non-hex");
const fetchFailed = atmosUp.parseCheckOutput(
  "status fetch-failed\nlocal deadbeef\nremote cafebabe\nshort dead\n",
);
assertEqual(fetchFailed.status, "fetch-failed", "parseCheckOutput fetch-failed");
assertEqual(fetchFailed.local, "deadbeef", "parseCheckOutput reads local sha");
assertEqual(fetchFailed.remote, "cafebabe", "parseCheckOutput reads remote sha");
assertEqual(fetchFailed.short, "dead", "parseCheckOutput reads short sha");
assertEqual(
  fetchFailed.summary,
  "Could not fetch the alpha branch.",
  "parseCheckOutput default fetch-failed summary",
);
assertEqual(
  atmosUp.parseCheckOutput("status current\n").summary,
  "Atmos is up to date.",
  "parseCheckOutput default current summary",
);
assertEqual(
  atmosUp.parseCheckOutput("status weird\n").status,
  "unknown",
  "parseCheckOutput rejects an unknown status",
);
