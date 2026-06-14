#!/usr/bin/env bash
# grandchild: sleeps holding inherited fd 1/2 (NO redirection — mirrors a bg
# process launched without </dev/null >/dev/null 2>&1)
sleep 30 &
echo "WORKER: work done, result written, exiting now (grandchild pid=$!)"
# worker exits immediately; grandchild keeps running
exit 0
