const fs = require('fs')
const path = require('path')
const vm = require('vm')

function load(rel) {
  const src = fs.readFileSync(path.join(__dirname, '..', rel), 'utf8')
  const ctx = {}
  vm.runInNewContext(src, ctx, { filename: rel })
  return ctx
}

function assert(condition, description, detail) {
  if (!condition) {
    if (detail) console.error(detail)
    console.error(`not ok - ${description}`)
    process.exit(1)
  }
  console.log(`ok - ${description}`)
}

function assertEqual(actual, expected, description) {
  assert(
    actual === expected,
    description,
    `expected: ${expected}\nactual:   ${actual}`
  )
}

const theme = load('services/Theme.js')
const shell = load('services/ShellConfig.js')

const colors = theme.parseColors(`
foreground = "#a9b1d6"
background = "#1a1b26"
accent = "#7aa2f7"
muted = "#414868"
red = "#f7768e"
`)
assertEqual(colors.foreground, '#a9b1d6', 'parseColors reads foreground')
assertEqual(colors.background, '#1a1b26', 'parseColors reads background')
assertEqual(colors.accent, '#7aa2f7', 'parseColors prefers accent over color4')
assertEqual(colors.muted, '#414868', 'parseColors reads muted')
assertEqual(colors.urgent, '#f7768e', 'parseColors maps red to urgent')

const legacy = theme.parseColors(`
color0 = "#111111"
color4 = "#0000ff"
color7 = "#eeeeee"
color8 = "#888888"
color1 = "#ff0000"
`)
assertEqual(legacy.background, '#111111', 'parseColors falls back to color0')
assertEqual(legacy.foreground, '#eeeeee', 'parseColors falls back to color7')
assertEqual(legacy.accent, '#0000ff', 'parseColors falls back to color4')
assertEqual(legacy.muted, '#888888', 'parseColors falls back to color8')
assertEqual(legacy.urgent, '#ff0000', 'parseColors maps color1 to urgent')

const shellValues = theme.parseShell(`
[font]
base-size = 14
[controls]
normal-fill-alpha = 0.04
hover-cursor-fill-alpha = 0.08
# comment
[bar]
position = top
`)
assertEqual(shellValues['font.base-size'], '14', 'parseShell reads numeric font.base-size')
assertEqual(shellValues['controls.normal-fill-alpha'], '0.04', 'parseShell reads control alphas')
assertEqual(shellValues['bar.position'], 'top', 'parseShell reads bare strings')
assertEqual(theme.numberToken(shellValues, 'font.base-size', 12), 14, 'numberToken coerces font size')

const merged = theme.mergeShell({ 'font.base-size': '12' }, { 'font.base-size': '16' })
assertEqual(merged['font.base-size'], '16', 'user shell.toml wins over theme')

assertEqual(theme.formatSeconds(45), '45s', 'formatSeconds under a minute')
assertEqual(theme.formatSeconds(150), '2m 30s', 'formatSeconds minutes and seconds')
assertEqual(theme.formatSeconds(300), '5m', 'formatSeconds whole minutes')

const parsed = shell.parseShellJson(
  '{"idle":{"screensaver":90,"lock":120},"bar":{"position":"left","transparent":true}}',
  '{}'
)
assertEqual(parsed.screensaver, 90, 'parseShellJson reads screensaver')
assertEqual(parsed.lock, 120, 'parseShellJson reads lock')
assertEqual(parsed.barPosition, 'left', 'parseShellJson reads bar position')
assert(parsed.barTransparent === true, 'parseShellJson reads bar transparency')

const fromDefaults = shell.parseShellJson('', '{"idle":{"screensaver":150,"lock":300},"bar":{"position":"top"}}')
assertEqual(fromDefaults.screensaver, 150, 'parseShellJson uses defaults when user file is empty')
assertEqual(fromDefaults.barPosition, 'top', 'parseShellJson default bar position')

assert(shell.rowMatches('', ['Theme']), 'empty query matches')
assert(shell.rowMatches('font', ['Theme', 'omarchy font set']), 'query matches hint')
assert(!shell.rowMatches('network', ['Theme', 'font']), 'query rejects unrelated rows')

const ui = load('services/RichUi.js')
const qr = ui.parseQrOutput('meta\twlan0\tWPA\tCafe\n0110\n1001\n')
assert(qr.ok === true, 'parseQrOutput accepts a meta header and matrix')
assertEqual(qr.ssid, 'Cafe', 'parseQrOutput reads ssid')
assertEqual(qr.size, 4, 'parseQrOutput matrix width')
assertEqual(qr.rows[1][0], 1, 'parseQrOutput cell values')

assertEqual(ui.parseMbpsLine('12.3'), 12.3, 'parseMbpsLine reads a rate')
assert(isNaN(ui.parseMbpsLine('nope')), 'parseMbpsLine rejects junk')

const diskLine = ui.parseDiskSpeedLine('read 450')
assertEqual(diskLine.kind, 'read', 'parseDiskSpeedLine kind')
assertEqual(diskLine.value, 450, 'parseDiskSpeedLine value')
assertEqual(ui.parseDiskSpeedLine('disk WD Black').value, 'WD Black', 'parseDiskSpeedLine disk name')

const snaps = ui.parseSnapperList('{"root":[{"number":3,"date":"2026-01-01","description":"pre"}]}')
assertEqual(snaps[0].id, 3, 'parseSnapperList id')
assertEqual(snaps[0].config, 'root', 'parseSnapperList config')

const dns = ui.parseDnsServers('1.1.1.1 8.8.8.8')
assert(dns.ok === true, 'parseDnsServers accepts IPv4')
assertEqual(dns.servers.length, 2, 'parseDnsServers token count')
assert(ui.parseDnsServers('--bad').ok === false, 'parseDnsServers rejects flags')
assert(ui.parseDnsServers('nope').ok === false, 'parseDnsServers rejects hostnames')
assert(ui.parseDnsServers('2001:db8::1').ok === true, 'parseDnsServers accepts compressed IPv6')
assert(ui.parseDnsServers('::').ok === false, 'parseDnsServers rejects unspecified IPv6')
assertEqual(ui.formatDnsInput('1.1.1.1,8.8.8.8'), '1.1.1.1 8.8.8.8', 'formatDnsInput turns commas into spaces')
assertEqual(ui.formatDnsInput('1921680'), '192.168.0', 'formatDnsInput splits IPv4 octets as they overflow')
assertEqual(ui.formatDnsInput('1111'), '111.1', 'formatDnsInput starts a new octet after three digits')
assertEqual(ui.formatDnsInput('2001:DB8::1'), '2001:db8::1', 'formatDnsInput lowercases IPv6')
assert(ui.dnsInputStatus('1.1').error === '', 'dnsInputStatus allows a partial IPv4')
assert(ui.dnsInputStatus('1:2:3:4:5:6:7:8:9').error.indexOf('Not an IPv4') === 0, 'dnsInputStatus rejects an overlong IPv6')
assert(ui.dnsInputStatus('1..2.3').error.indexOf('Not an IPv4') === 0, 'dnsInputStatus rejects a double-dot IPv4')
assert(ui.dnsInputStatus('1.1.1.1 8.8.8.8').ok === true, 'dnsInputStatus accepts two IPv4 servers')
assert(ui.isPartialIpv6('2001:db8:') === true, 'isPartialIpv6 allows a trailing colon')
assert(ui.isIpv6('fe80::1') === true, 'isIpv6 accepts link-local')

assertEqual(ui.parseLauncherName('Notes'), 'Notes', 'parseLauncherName accepts a name')
assertEqual(ui.parseLauncherName('  Notes  '), 'Notes', 'parseLauncherName trims')
assertEqual(ui.parseLauncherName('foo/bar'), '', 'parseLauncherName rejects a slash')
assertEqual(ui.parseLauncherName('-secret'), '', 'parseLauncherName rejects a leading hyphen')
assertEqual(ui.parseWebAppUrl('example.com'), 'https://example.com', 'parseWebAppUrl adds https')
assertEqual(ui.parseWebAppUrl('https://hey.com'), 'https://hey.com', 'parseWebAppUrl keeps https')
assertEqual(ui.parseWebAppUrl('javascript:alert(1)'), '', 'parseWebAppUrl rejects a non-http scheme')
assert(ui.isTuiWindowStyle('float') === true, 'isTuiWindowStyle accepts float')
assert(ui.isTuiWindowStyle('stack') === false, 'isTuiWindowStyle rejects other values')

assertEqual(ui.usagePercent(50, 100), 50, 'usagePercent')
assertEqual(ui.pathFromUrl('file:///home/a/b.png'), '/home/a/b.png', 'pathFromUrl strips file://')
assertEqual(ui.fileBasename('/home/a/wall.png'), 'wall.png', 'fileBasename last path segment')
assertEqual(ui.fileBasename('plain'), 'plain', 'fileBasename no slash')
assertEqual(ui.fileBasename(''), '', 'fileBasename empty')

const wifi = ui.sortWifiRows([
  ui.wifiRow('b', 10, 'psk', false, false),
  ui.wifiRow('a', 80, 'psk', true, true)
])
assertEqual(wifi[0].ssid, 'a', 'sortWifiRows puts connected first')
assert(ui.requiresCredentials('psk') === true, 'psk needs a password')
assert(ui.requiresCredentials('open') === false, 'open has no password')
assert(ui.isEnterprise('enterprise') === true, 'enterprise kind')

assert(ui.isTimezoneId('America/New_York') === true, 'isTimezoneId accepts Area/City')
assert(ui.isTimezoneId('UTC') === true, 'isTimezoneId accepts UTC')
assert(ui.isTimezoneId('Etc/GMT+12') === true, 'isTimezoneId accepts Etc/GMT+12')
assert(ui.parseTimezoneId('  Europe/Paris  ') === 'Europe/Paris', 'parseTimezoneId trims')
assert(ui.isTimezoneId('../etc/passwd') === false, 'isTimezoneId rejects path traversal')
assert(ui.isTimezoneId('-America/New_York') === false, 'isTimezoneId rejects flags')
assert(ui.isTimezoneId('America/New York') === false, 'isTimezoneId rejects spaces')
assert(ui.parseTimezoneId('nope!') === '', 'parseTimezoneId rejects junk')

assert(ui.isHostname('hallas') === true, 'isHostname accepts a label')
assert(ui.isHostname('my-pc') === true, 'isHostname accepts a hyphen')
assert(ui.isHostname('a') === true, 'isHostname accepts a single character')
assert(ui.isHostname('foo.bar') === true, 'isHostname accepts an FQDN')
assert(ui.parseHostname('  omarchy  ') === 'omarchy', 'parseHostname trims')
assert(ui.isHostname('-bad') === false, 'isHostname rejects a leading hyphen')
assert(ui.isHostname('bad-') === false, 'isHostname rejects a trailing hyphen')
assert(ui.isHostname('has space') === false, 'isHostname rejects spaces')
assert(ui.parseHostname('--flag') === '', 'parseHostname rejects flags')

assert(ui.isKeyboardLayoutId('us') === true, 'isKeyboardLayoutId accepts us')
assert(ui.isKeyboardLayoutId('gb') === true, 'isKeyboardLayoutId accepts gb')
assert(ui.parseKeyboardLayoutId('us,ru') === 'us', 'parseKeyboardLayoutId takes the first layout')
assert(ui.isKeyboardLayoutId('US') === false, 'isKeyboardLayoutId rejects uppercase')
assert(ui.isKeyboardLayoutId('--us') === false, 'isKeyboardLayoutId rejects flags')
const xkbLayouts = ui.parseXkbLayoutList(`
! model
  pc105           Generic 105-key PC
! layout
  us              English (US)
  gb              English (UK)
  de              German
! variant
  intl            English (US, intl.)
`)
assertEqual(xkbLayouts.length, 3, 'parseXkbLayoutList reads the layout section')
assertEqual(xkbLayouts[0].value, 'us', 'parseXkbLayoutList value')
assertEqual(xkbLayouts[0].label, 'English (US)', 'parseXkbLayoutList label')
assertEqual(xkbLayouts[1].value, 'gb', 'parseXkbLayoutList gb')

assert(ui.parseTimedatectlYes('yes') === true, 'parseTimedatectlYes yes')
assert(ui.parseTimedatectlYes('YES') === true, 'parseTimedatectlYes is case-insensitive')
assert(ui.parseTimedatectlYes('no') === false, 'parseTimedatectlYes no')
assert(ui.parseTimedatectlYes('maybe') === false, 'parseTimedatectlYes rejects junk')

assert(ui.isLocaleId('en_US.UTF-8') === true, 'isLocaleId accepts en_US.UTF-8')
assert(ui.isLocaleId('C.UTF-8') === true, 'isLocaleId accepts C.UTF-8')
assert(ui.isLocaleId('ca_ES.UTF-8@valencia') === true, 'isLocaleId accepts a modifier')
assert(ui.parseLocaleId('  de_DE.UTF-8  ') === 'de_DE.UTF-8', 'parseLocaleId trims')
assert(ui.isLocaleId('EN_US.UTF-8') === false, 'isLocaleId rejects uppercase language')
assert(ui.isLocaleId('en') === false, 'isLocaleId rejects a language-only tag')
assert(ui.parseLocaleId('--bad') === '', 'parseLocaleId rejects flags')

assert(ui.isFullName('Ada Lovelace') === true, 'isFullName accepts a display name')
assert(ui.isFullName('') === true, 'isFullName allows empty')
assert(ui.parseFullName('  Jean-Luc  ') === 'Jean-Luc', 'parseFullName trims')
assert(ui.isFullName('bad:name') === false, 'isFullName rejects a colon')
assert(ui.isFullName('Last, First') === false, 'isFullName rejects a comma')
assert(ui.parseFullName('--flag') === '', 'parseFullName rejects flags')

assertEqual(ui.parseParallelDownloads('ParallelDownloads = 5\n'), 5, 'parseParallelDownloads reads a value')
assertEqual(ui.parseParallelDownloads('#ParallelDownloads = 9\nParallelDownloads = 3\n'), 3, 'parseParallelDownloads skips comments')
assertEqual(ui.parseParallelDownloads('Color\n'), 0, 'parseParallelDownloads missing')
assertEqual(ui.parseParallelDownloads('ParallelDownloads = 99\n'), 20, 'parseParallelDownloads clamps high values')

const layout = load('services/Layout.js')

function flags(items) {
  return layout.splitAfterVisible(items).map(function(v) { return v ? 1 : 0 }).join('')
}

assertEqual(
  flags([{ visible: true }, { visible: true }, { visible: true }]),
  '110',
  'splitAfterVisible splits between visible items, not after the last'
)
assertEqual(
  flags([{ visible: true }, { visible: false }, { visible: true }]),
  '100',
  'splitAfterVisible skips hidden items between two visible ones'
)
assertEqual(
  flags([{ visible: false }, { visible: true }, { visible: false }]),
  '000',
  'splitAfterVisible has no split for a single visible item'
)
assertEqual(
  flags([{ visible: false }, { visible: false }]),
  '00',
  'splitAfterVisible has no split when nothing is visible'
)
assertEqual(
  flags([]).length,
  0,
  'splitAfterVisible empty list'
)
assertEqual(
  flags(null).length,
  0,
  'splitAfterVisible ignores a non-array'
)

function beforeFlags(items) {
  return layout.splitBeforeVisible(items).map(function(v) { return v ? 1 : 0 }).join('')
}

assertEqual(
  beforeFlags([{ visible: true }, { visible: true }, { visible: true }]),
  '011',
  'splitBeforeVisible draws on later items, not after the last'
)
assertEqual(
  beforeFlags([{ visible: true }, { visible: false }, { visible: true }]),
  '001',
  'splitBeforeVisible skips hidden items'
)
assertEqual(
  beforeFlags([{ visible: false }, { visible: true }, { visible: false }]),
  '000',
  'splitBeforeVisible has no split on the first visible item'
)

const helpTopics = layout.sectionHelpTopics([
  { label: 'Theme', description: 'Palette.', detail: 'A named palette plus templates.', hint: 'omarchy theme set' },
  { label: 'Font', description: 'Monospace family.', hint: 'omarchy font set' },
  { label: '', description: '', detail: '', hint: '' },
  null
])
assertEqual(helpTopics.length, 2, 'sectionHelpTopics skips empty rows')
assertEqual(helpTopics[0].body, 'A named palette plus templates.', 'sectionHelpTopics prefers detail')
assertEqual(helpTopics[1].body, 'Monospace family.', 'sectionHelpTopics falls back to description')
assertEqual(helpTopics[1].command, 'omarchy font set', 'sectionHelpTopics keeps the command')
assertEqual(layout.sectionHelpTopics(null).length, 0, 'sectionHelpTopics ignores a non-array')

const wrap = load('services/TextWrap.js')
function ch(s) { return s.length }

assertEqual(
  wrap.prettyWrap('The quick brown fox jumps', ch, 20, 1),
  'The quick brown\nfox jumps',
  'prettyWrap balances a leftover last word'
)
assertEqual(
  wrap.balanceWrap('aaa bbb ccc ddd', ch, 7, 1),
  'aaa bbb\nccc ddd',
  'balanceWrap splits even groups'
)
assertEqual(
  wrap.balanceWrap('Word another last', ch, 12, 1),
  'Word another\nlast',
  'balanceWrap keeps the greedy line count'
)
assertEqual(
  wrap.prettyWrap('Word another last', ch, 12, 1),
  'Word\nanother last',
  'prettyWrap pulls a one-word last line up'
)
assertEqual(
  wrap.prettyWrap('Hello world', ch, 20, 1),
  'Hello world',
  'prettyWrap leaves a single line alone'
)
assertEqual(
  wrap.prettyWrap('/home/foo/very-long-path', ch, 10, 1),
  '/home/foo/very-long-path',
  'prettyWrap leaves a path without spaces alone'
)
assertEqual(wrap.prettyWrap('', ch, 20, 1), '', 'prettyWrap empty')
assertEqual(
  wrap.splitWords('  alpha   beta ').join(','),
  'alpha,beta',
  'splitWords collapses whitespace'
)

const hypr = load('services/HyprPrefs.js')
const look = hypr.clampLook({ gapsIn: 80, layout: 'niri', dimStrength: 2 })
assertEqual(look.gapsIn, 64, 'clampLook caps gaps')
assertEqual(look.layout, 'dwindle', 'clampLook rejects an unknown layout')
assertEqual(look.dimStrength, 1, 'clampLook caps dim strength')

const lookLua = hypr.serializeLook({
  gapsIn: 8,
  gapsOut: 12,
  borderSize: 3,
  rounding: 6,
  blur: true,
  shadow: false,
  layout: 'scrolling',
  columnWidth: 0.97,
  dimInactive: true,
  dimStrength: 0.15,
  animations: false,
  cursorHideOnKey: false,
  cursorWarp: true,
  allowTearing: true,
  resizeOnBorder: false
})
assert(lookLua.indexOf('-- omarchy-prefs:look begin') === 0, 'serializeLook starts with the look sentinel')
assert(lookLua.indexOf('gaps_in = 8') !== -1, 'serializeLook writes gaps_in')
assert(lookLua.indexOf('layout = "scrolling"') !== -1, 'serializeLook writes scrolling')
assert(lookLua.indexOf('column_width = 0.97') !== -1, 'serializeLook writes column width')
assert(lookLua.indexOf('warp_on_change_workspace = 1') !== -1, 'serializeLook writes cursor warp')
assert(lookLua.indexOf('hl.env("HYPRCURSOR_SIZE", "24")') !== -1, 'serializeLook writes default cursor size')
assert(lookLua.indexOf('active_opacity = 1') !== -1, 'serializeLook writes default active opacity')
assert(lookLua.indexOf('preserve_split = false') !== -1, 'serializeLook writes preserve_split')
assert(lookLua.indexOf('focus_on_activate = false') !== -1, 'serializeLook writes focus_on_activate')

const lookExtras = hypr.serializeLook({ cursorSize: 40, activeOpacity: 0.8, preserveSplit: true, focusOnActivate: true })
assert(lookExtras.indexOf('hl.env("HYPRCURSOR_SIZE", "40")') !== -1, 'serializeLook writes a custom cursor size')
assert(lookExtras.indexOf('hl.env("XCURSOR_SIZE", "40")') !== -1, 'serializeLook writes XCURSOR_SIZE')
assert(lookExtras.indexOf('active_opacity = 0.8') !== -1, 'serializeLook writes active opacity')
assert(lookExtras.indexOf('preserve_split = true') !== -1, 'serializeLook writes preserve_split on')
assertEqual(hypr.clampLook({ cursorSize: 90 }).cursorSize, 64, 'clampLook caps cursor size')
assertEqual(hypr.clampLook({ activeOpacity: 0.05 }).activeOpacity, 0.2, 'clampLook floors active opacity')

const seed = '-- keep this comment\n\nhl.config({ general = { gaps_in = 1 } })\n'
const applied = hypr.applyLookFile(seed, { gapsIn: 4, gapsOut: 8 })
assert(applied.indexOf('-- keep this comment') !== -1, 'applyLookFile keeps user comments')
assert(applied.indexOf('hl.config({ general = { gaps_in = 1 } })') !== -1, 'applyLookFile keeps earlier hl.config')
assert(hypr.hasSentinel(applied, hypr.LOOK_BEGIN, hypr.LOOK_END), 'applyLookFile inserts the look sentinel')
const twice = hypr.applyLookFile(applied, { gapsIn: 9, gapsOut: 8 })
assertEqual(
  (twice.match(/-- omarchy-prefs:look begin/g) || []).length,
  1,
  'applyLookFile replaces an existing look block'
)
assert(twice.indexOf('gaps_in = 9') !== -1, 'applyLookFile updates gaps')
const resetLook = hypr.resetLookFile(twice)
assert(resetLook.indexOf('-- omarchy-prefs:look begin') === -1, 'resetLookFile strips the look block')
assert(resetLook.indexOf('-- keep this comment') !== -1, 'resetLookFile keeps user comments')

const inputLua = hypr.serializeInput({
  sensitivity: -0.64,
  accelProfile: 'flat',
  naturalScroll: true,
  kbLayoutOverride: 'us,dk',
  kbGroupToggle: true,
  workspaceGesture: true
})
assert(inputLua.indexOf('accel_profile = "flat"') !== -1, 'serializeInput writes a flat profile')
assert(inputLua.indexOf('kb_layout = "us,dk"') !== -1, 'serializeInput writes a layout override')
assert(inputLua.indexOf('grp:alts_toggle') !== -1, 'serializeInput adds the group toggle')
assert(inputLua.indexOf('hl.gesture({ fingers = 3') !== -1, 'serializeInput writes the workspace gesture')
assertEqual(hypr.clampInput({ kbLayoutOverride: 'US,dk' }).kbLayoutOverride, 'us,dk', 'clampInput lowercases layouts')
assertEqual(hypr.clampInput({ kbLayoutOverride: 'us/dk' }).kbLayoutOverride, '', 'clampInput rejects a slash in layouts')
assertEqual(hypr.parseCssFirst('5 5 5 5'), 5, 'parseCssFirst reads the first css number')
assertEqual(hypr.parseHyprOption({ int: 40 }), 40, 'parseHyprOption reads int')
assertEqual(hypr.parseHyprOption({ bool: false }), false, 'parseHyprOption reads bool')
assertEqual(hypr.parseHyprOption({ css: '10 10 10 10' }), 10, 'parseHyprOption reads css')
assertEqual(hypr.parseHyprOption({ str: '[[EMPTY]]' }), '', 'parseHyprOption treats empty str as blank')

const sunset = load('services/HyprSunset.js')
assertEqual(sunset.parseTime('7:00'), '07:00', 'parseTime pads an hour')
assertEqual(sunset.parseTime('20:15'), '20:15', 'parseTime keeps a valid time')
assertEqual(sunset.parseTime('25:00'), '', 'parseTime rejects a bad hour')
const parsedSunset = sunset.parseConf(`
profile {
    time = 06:30
    identity = true
}

profile {
    time = 21:00
    temperature = 3800
}
`)
assertEqual(parsedSunset.day, '06:30', 'parseConf reads the day profile')
assertEqual(parsedSunset.night, '21:00', 'parseConf reads the night profile')
assertEqual(parsedSunset.nightOn, true, 'parseConf sees a night profile')
assertEqual(parsedSunset.temperature, 3800, 'parseConf reads temperature')
const writtenSunset = sunset.serializeConf({ day: '07:00', night: '20:00', nightOn: true, temperature: 4000 })
assert(writtenSunset.indexOf('time = 07:00') !== -1, 'serializeConf writes day')
assert(writtenSunset.indexOf('temperature = 4000') !== -1, 'serializeConf writes temperature')
assertEqual(sunset.serializeConf({ nightOn: false }).indexOf('temperature') === -1, true, 'serializeConf omits night when off')

const hooks = load('services/Hooks.js')
assertEqual(hooks.argFor('theme-set').indexOf('theme') !== -1, true, 'argFor describes theme-set')
assertEqual(hooks.isType('battery-low'), true, 'isType accepts battery-low')
assertEqual(hooks.isType('nope'), false, 'isType rejects an unknown hook')
assertEqual(hooks.parseListing([{ type: 'theme-set', name: 'mine.sh', path: '/tmp/../etc/passwd', sample: false }]).length, 0, 'parseListing rejects a path with ..')
assertEqual(hooks.itemsFor([{ type: 'theme-set', name: 'a.sh', path: '/home/u/.config/omarchy/hooks/theme-set.d/a.sh', sample: false }, { type: 'font-set', name: 'b.sh', path: '/home/u/.config/omarchy/hooks/font-set.d/b.sh', sample: false }], 'theme-set').length, 1, 'itemsFor filters by type')
assertEqual(hooks.labelFor('theme-set'), 'Theme set', 'labelFor names theme-set')
assertEqual(hooks.sanitizeName('notify'), 'notify.sh', 'sanitizeName adds .sh')
assertEqual(hooks.sanitizeName('bad/name'), '', 'sanitizeName rejects a slash')
assertEqual(hooks.sanitizeName('keep.sample'), '', 'sanitizeName rejects a sample suffix')
assertEqual(hooks.sanitizeLine('echo "$1"'), 'echo "$1"', 'sanitizeLine keeps $1')
assertEqual(hooks.sanitizeLine('bad\nline'), '', 'sanitizeLine rejects a newline')
assert(hooks.scriptBody('theme-set', 'echo "$1"').indexOf('echo "$1"') !== -1, 'scriptBody writes the command')
assertEqual(hooks.destHint('theme-set', 'notify'), '~/.config/omarchy/hooks/theme-set.d/notify.sh', 'destHint names the install path')
assertEqual(hooks.displayTypes([{ type: 'waki-webapp-install', name: 'waki-webapp-install', path: '/home/u/.config/omarchy/hooks/waki-webapp-install', sample: false, flat: true }]).length, 7, 'displayTypes appends an extra hook id')

const auto = load('services/Autostart.js')
const autoSeed = '-- Extra autostart processes.\no.launch_on_start("waybar")\n'
const autoApplied = auto.applyFile(autoSeed, ['hyprsunset', 'mako'])
assert(autoApplied.indexOf('-- Extra autostart processes.') !== -1, 'applyFile keeps user comments')
assert(autoApplied.indexOf('o.launch_on_start("waybar")') !== -1, 'applyFile keeps unmanaged launch lines')
assert(autoApplied.indexOf('o.launch_on_start("hyprsunset")') !== -1, 'applyFile writes a managed command')
const autoParsed = auto.parseFile(autoApplied)
assertEqual(autoParsed.filter(function(row) { return row.managed }).length, 2, 'parseFile marks managed commands')
assertEqual(autoParsed.filter(function(row) { return !row.managed && row.command === 'waybar' }).length, 1, 'parseFile keeps unmanaged commands')
assertEqual(auto.sanitizeCommand('bad\ncmd'), '', 'sanitizeCommand rejects a newline')

const software = load('services/Software.js')
assertEqual(software.lookup('steam').wipe, true, 'catalog marks Steam as a wipe remove')
assertEqual(software.presentIn(software.lookup('firefox'), { browsers: { firefox: true } }), true, 'presentIn reads a browser bag')
assertEqual(software.presentIn(software.lookup('vscode'), { editors: { code: true } }), true, 'presentIn maps vscode to editors.code')
assertEqual(software.isDevEnv('python'), true, 'isDevEnv accepts python')
assertEqual(software.isDockerDb('PostgreSQL'), true, 'isDockerDb accepts PostgreSQL')
assertEqual(software.groupItems('gaming').length > 3, true, 'groupItems lists gaming installers')

const binds = load('services/Bindings.js')
assertEqual(binds.sanitizeKeys('SUPER + SHIFT + R'), 'SUPER + SHIFT + R', 'sanitizeKeys keeps a chord')
assertEqual(binds.sanitizeKeys('SUPER + F\n'), '', 'sanitizeKeys rejects a newline')
assertEqual(binds.sanitizeCommand('bad\ncmd'), '', 'sanitizeCommand rejects a newline')
const bindSeed = '-- Keep only your personal keybinding overrides here.\no.bind("SUPER + D", "Desks", "omarchy-shell shell toggle com.mdtrr.omadesk")\n'
const bindApplied = binds.applyFile(bindSeed, [
  { keys: 'SUPER + F', label: 'Files', command: 'nautilus', unbind: true },
  { keys: 'SUPER + SHIFT + B', unbind: true }
])
assert(bindApplied.indexOf('-- Keep only your personal keybinding overrides here.') !== -1, 'applyFile keeps binding comments')
assert(bindApplied.indexOf('o.bind("SUPER + D", "Desks"') !== -1, 'applyFile keeps unmanaged binds')
assert(bindApplied.indexOf('hl.unbind("SUPER + F")') !== -1, 'applyFile writes unbind before a replacement')
assert(bindApplied.indexOf('o.bind("SUPER + F", "Files", "nautilus")') !== -1, 'applyFile writes a managed bind')
assert(bindApplied.indexOf('hl.unbind("SUPER + SHIFT + B")') !== -1, 'applyFile writes an unbind-only row')
const bindParsed = binds.parseFile(bindApplied)
assertEqual(bindParsed.filter(function(row) { return row.managed }).length, 2, 'parseFile marks managed bindings')
assertEqual(bindParsed.filter(function(row) { return !row.managed && row.keys === 'SUPER + D' }).length, 1, 'parseFile keeps unmanaged bindings')
const printed = binds.parsePrint('SUPER + Q                         \u2192 Close window\nSUPER + D                         \u2192 Desks\n')
assertEqual(printed.length, 2, 'parsePrint reads display lines')
assertEqual(printed[0].action, 'Close window', 'parsePrint reads the action')
assertEqual(binds.catalogConflict(printed, 'SUPER + Q'), 'Close window', 'catalogConflict finds a taken chord')

const rules = load('services/WindowRules.js')
assertEqual(rules.sanitizeMatch('firefox'), 'firefox', 'sanitizeMatch keeps a class')
assertEqual(rules.sanitizeMatch('bad]]class'), '', 'sanitizeMatch rejects ]]')
assertEqual(rules.sanitizeWorkspace('../etc'), '', 'sanitizeWorkspace rejects a path')
const ruleSeed = 'o.window("org.omarchy.prefs", { float = true })\no.window("org.omarchy.prefs", { center = true })\n'
const ruleApplied = rules.applyFile(ruleSeed, [
  { match: '^Emulator$', placement: 'float', center: true, width: 1280, height: 800 },
  { match: 'qemu', workspace: '5' }
])
assert(ruleApplied.indexOf('o.window("org.omarchy.prefs", { float = true })') !== -1, 'applyFile keeps prefs window rules')
assert(ruleApplied.indexOf('o.window("^Emulator$", { float = true, center = true, size = { 1280, 800 } })') !== -1, 'applyFile writes a managed window rule')
assert(ruleApplied.indexOf('o.window("qemu", { workspace = "5" })') !== -1, 'applyFile writes a workspace rule')
const ruleParsed = rules.parseFile(ruleApplied)
assertEqual(ruleParsed.filter(function(row) { return row.managed }).length, 2, 'parseFile marks managed window rules')
assertEqual(ruleParsed.filter(function(row) { return !row.managed && row.match === 'org.omarchy.prefs' }).length, 2, 'parseFile keeps the prefs window rules')
const required = rules.ensureRequire('require("hypr.autostart")\nrequire("default.hypr.toggles")\n')
assert(required.indexOf('require("hypr.omarchy_prefs")\nrequire("default.hypr.toggles")') !== -1, 'ensureRequire inserts before toggles')
assertEqual(rules.ensureRequire(''), '', 'ensureRequire leaves an empty hyprland.lua alone')
assertEqual(rules.describe({ placement: 'float', center: true }).indexOf('float') !== -1, true, 'describe names float')

