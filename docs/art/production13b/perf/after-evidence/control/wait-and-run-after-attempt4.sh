#!/usr/bin/env bash
set -euo pipefail
cd /Users/rexxa/work/new-world-art-prod
source /Users/rexxa/work/art-production13b-evidence/environment.sh
python3 - <<'WAIT'
import time,datetime
until=1792430329
print('Waiting for mandatory cooldown until',datetime.datetime.fromtimestamp(until,datetime.timezone.utc).isoformat(),flush=True)
while time.time()<until:
 time.sleep(min(30,until-time.time()))
print('Cooldown elapsed',datetime.datetime.now(datetime.timezone.utc).isoformat(),flush=True)
WAIT
python3 tools/art_pipeline/session_validity_p13.py --record-wait 1792428528.382771 1792430329
exec bash /Users/rexxa/work/art-production13b-evidence/run-after-attempt4.sh
