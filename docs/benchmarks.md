# Benchmarks

`./tests/bench-ratmos` measures the backend, `ratmos`, and nothing else. It does not measure the Quickshell window, so these numbers say nothing about the app's total memory or CPU.

## Method

The script builds `ratmos` in release mode, then runs it with `--backend plain` on an empty fixture root under a throwaway `HOME`. It reads and writes nothing of yours.

- **Binary size:** `stat` on `backend/target/release/ratmos`.
- **One-shot request:** 200 sequential `ratmos request` calls with `{"op":"version"}`, wall time divided by 200. That includes process start, which is what a one-shot caller pays.
- **Serve idle:** start `ratmos serve`, send one `version` request and `watch.set` with `chrome` on, wait 10 seconds, then read `VmRSS` from `/proc/<pid>/status` and utime plus stime from `/proc/<pid>/stat`.

Run it again with `./tests/bench-ratmos [idle seconds]`. The idle step is the point of `ratmos serve`: one process per window that pushes changes instead of being polled, so idle should cost nothing.

## Results

ratmos 0.1.0 at commit `21dfb19`, measured 10 October 2026. AMD Ryzen AI Max+ 395, Linux 7.2.5, release build, plain backend.

| | |
| --- | --- |
| `ratmos` binary | 1.9 MB (1,934,752 bytes) |
| One-shot `request` (`version`) | 566 µs |
| `ratmos serve`, idle 10 s | 2.0 MB RSS, 0 CPU ticks |

Results describe one machine and one workload. The `plain` backend answers without running Omarchy commands, so ops that shell out to `omarchy` or `hyprctl` take longer on a real system, and `version` is the cheapest op there is.

## When to update this page

Re-run the script and replace the table here and the Footprint table in the README when `backend/Cargo.toml` changes version, or when `ratmos serve` changes how it watches. Keep the date, version and commit with the numbers.
