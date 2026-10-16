#!/usr/bin/env bash
# usage: launch.sh NAME CMD...  — runs CMD detached (setsid nohup), log p14/logs/NAME.log, exit code NAME.exit, PID NAME.pid
L=/home/hyunlord/work/worldsim-wm-data/p14/logs
name=$1; shift
rm -f $L/$name.exit
setsid nohup bash -c "date +%s > $L/$name.start; $*; echo \$? > $L/$name.exit; date +%s > $L/$name.end" > $L/$name.log 2>&1 < /dev/null &
echo $! > $L/$name.pid
echo "launched $name pid $(cat $L/$name.pid)"
