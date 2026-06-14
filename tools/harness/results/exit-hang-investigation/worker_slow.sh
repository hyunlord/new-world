#!/usr/bin/env bash
setsid sleep 40 >/dev/null 2>&1 &   # detached grandchild in its OWN session
echo "WORKER_SLOW: spawned detached grandchild pid=$! ; now sleeping 40s myself"
sleep 40
