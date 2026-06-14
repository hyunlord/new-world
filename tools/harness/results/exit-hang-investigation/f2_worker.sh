#!/usr/bin/env bash
M="$1"
trap '' TERM
( trap '' TERM; exec -a "${M}_grandchild" sleep 40 ) &
echo "worker: grandchild pid=$! (marker ${M}_grandchild)"
exec -a "${M}_worker" sleep 40
