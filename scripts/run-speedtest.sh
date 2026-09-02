#!/bin/bash
# Run both directions of omarchy network speedtest in a terminal.

set -euo pipefail

echo "Download"
omarchy network speedtest down
echo
echo "Upload"
omarchy network speedtest up
