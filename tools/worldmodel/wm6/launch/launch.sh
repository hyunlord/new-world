#!/bin/bash
# WM-6 copy of wm5/launch/launch.sh.
# usage: launch.sh LOGDIR NAME CMD...  — runs CMD detached (setsid nohup); LOGDIR/NAME.{log,pid,start,exit,end}
L=$1; name=$2; shift 2
mkdir -p "$L"; rm -f "$L/$name.exit"
setsid nohup bash -c "date +%s > $L/$name.start; $*; echo \$? > $L/$name.exit; date +%s > $L/$name.end" > "$L/$name.log" 2>&1 < /dev/null &
echo $! > "$L/$name.pid"
echo "launched $name pid $(cat $L/$name.pid)"
