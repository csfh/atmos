const { load, assertEqual } = require("./harness");

const fb = load("services/Feedback.js");
const t0 = 1_000_000;

let s = fb.empty();
assertEqual(s.phase, "idle", "feedback starts idle");

s = fb.touch(s, fb.rowKey("display", "Scale"), t0);
s = fb.begin(s, t0 + 200);
assertEqual(s.phase, "pending", "a write after a touch is pending");
assertEqual(fb.rowStatus(s, "display|Scale").phase, "pending", "the touched row shows pending");
assertEqual(fb.rowStatus(s, "display|Other").phase, "idle", "other rows stay idle");

s = fb.finish(s, true, "", t0 + 400);
assertEqual(s.phase, "saved", "a clean exit is saved");
assertEqual(fb.chipLabel(s), "Saved", "the chip reads Saved");
assertEqual(fb.expire(s, t0 + 400 + fb.SAVED_MS - 1).phase, "saved", "saved lingers");
assertEqual(fb.expire(s, t0 + 400 + fb.SAVED_MS).phase, "idle", "saved fades");
assertEqual(fb.expire(s, t0 + 400 + fb.SAVED_MS).rowKey, "", "a faded mark drops its row");

let f = fb.touch(fb.empty(), "a|b", t0);
f = fb.begin(f, t0 + 100);
f = fb.finish(f, false, "boom", t0 + 300);
assertEqual(f.phase, "failed", "a failed exit is failed");
assertEqual(fb.rowStatus(f, "a|b").message, "boom", "the row carries the message");
assertEqual(fb.expire(f, t0 + 999999).phase, "failed", "a failure does not fade");
assertEqual(fb.dismiss(f).phase, "idle", "dismiss clears a failure");

let stale = fb.touch(fb.empty(), "a|b", t0);
stale = fb.begin(stale, t0 + fb.TOUCH_MS + 1);
assertEqual(stale.rowKey, "", "a stale touch is not attributed");
assertEqual(stale.phase, "pending", "an unattributed write is still pending");

let bare = fb.finish(fb.empty(), false, "", t0);
assertEqual(bare.message, "Command failed", "a failure always has a message");
