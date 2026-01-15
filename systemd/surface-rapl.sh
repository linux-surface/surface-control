#!/bin/sh
target=/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0/constraint_2_power_limit_uw
[ -f "$target" ] && echo 0 > "$target" || :
