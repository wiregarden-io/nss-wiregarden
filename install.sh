#!/bin/bash
set -eux
cd $(dirname $0)/target/release
cp libnss_wiregarden.so libnss_wiregarden.so.2
sudo install -m 0644 libnss_wiregarden.so.2 /lib
sudo /sbin/ldconfig -n /lib /usr/lib
