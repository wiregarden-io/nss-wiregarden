#!/bin/bash
set -eux
lxc file push $(dirname $0)/target/${2:-release}/libnss_wiregarden.so $1/tmp/libnss_wiregarden.so.2
lxc exec $1 -- install -m 0644 /tmp/libnss_wiregarden.so.2 /lib
lxc exec $1 -- rm -f /tmp/libnss_wiregarden.so.2
lxc exec $1 -- /sbin/ldconfig -n /lib /usr/lib
