const { load, assert, assertEqual } = require("./harness");

const sunset = load("services/HyprSunset.js");
assertEqual(sunset.parseTime("7:00"), "07:00", "parseTime pads an hour");
assertEqual(sunset.parseTime("20:15"), "20:15", "parseTime keeps a valid time");
assertEqual(sunset.parseTime("25:00"), "", "parseTime rejects a bad hour");
const parsedSunset = sunset.parseConf(`
profile {
    time = 06:30
    identity = true
}

profile {
    time = 21:00
    temperature = 3800
}
`);
assertEqual(parsedSunset.day, "06:30", "parseConf reads the day profile");
assertEqual(parsedSunset.night, "21:00", "parseConf reads the night profile");
assertEqual(parsedSunset.nightOn, true, "parseConf sees a night profile");
assertEqual(parsedSunset.temperature, 3800, "parseConf reads temperature");
const writtenSunset = sunset.serializeConf({
  day: "07:00",
  night: "20:00",
  nightOn: true,
  temperature: 4000,
});
assert(writtenSunset.indexOf("time = 07:00") !== -1, "serializeConf writes day");
assert(writtenSunset.indexOf("temperature = 4000") !== -1, "serializeConf writes temperature");
assertEqual(
  sunset.serializeConf({ nightOn: false }).indexOf("temperature") === -1,
  true,
  "serializeConf omits night when off",
);
assertEqual(sunset.parseTime("9:05"), "09:05", "parseTime pads a single-digit hour");
assertEqual(sunset.parseTime("7:5"), "", "parseTime rejects a single-digit minute");
assertEqual(sunset.parseTime("24:00"), "", "parseTime rejects 24:00");
assertEqual(sunset.clampTemp(2500, 4000), 3000, "clampTemp floors below 3000");
assertEqual(sunset.clampTemp(9000, 4000), 6500, "clampTemp caps above 6500");
assertEqual(sunset.clampTemp("nope", 4000), 4000, "clampTemp uses fallback on NaN");
const clampedSunset = sunset.clampSchedule({
  day: "nope",
  night: "21:30",
  nightOn: "yes",
  temperature: 2000,
});
assertEqual(clampedSunset.day, "07:00", "clampSchedule falls back to default day");
assertEqual(clampedSunset.night, "21:30", "clampSchedule keeps a valid night");
assertEqual(clampedSunset.nightOn, false, "clampSchedule requires nightOn === true");
assertEqual(clampedSunset.temperature, 3000, "clampSchedule floors temperature");
const quotedSunset = sunset.parseConf(`
profile {
    time = "8:00"
    identity = true
}
profile {
    time = "19:45"
    temperature = 5000
}
`);
assertEqual(quotedSunset.day, "08:00", "parseConf reads a quoted day time");
assertEqual(quotedSunset.night, "19:45", "parseConf reads a quoted night time");
assertEqual(
  sunset.parseConf("").nightOn,
  false,
  "parseConf empty file uses defaults with night off",
);
assertEqual(
  sunset.parseConf("profile {\n    time = 99:99\n    identity = true\n}\n").day,
  "07:00",
  "parseConf skips an invalid day time",
);
assertEqual(
  sunset.parseConf("profile {\n    time = 21:00\n}\n").nightOn,
  false,
  "parseConf skips a profile with neither identity nor temperature",
);
