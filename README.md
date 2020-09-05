# nss-wiregarden

nss-wiregarden is a [Name Service Switch](https://en.wikipedia.org/wiki/Name_Service_Switch) host plugin that resolves
[wiregarden](https://wiregarden.io) peers by device and network name.

# Build

```
cargo build --release
```

# Install

Package installation is coming soon for Ubuntu. Meanwhile, you can install from
a local build.

Run `./install.bash` to install on the current machine, or `./install-lxd.bash <container>`
to install into a LXD container on the current machine (useful for testing).
This will install the shared library so that libnss can load it.

nss-wiregarden requires libsqlite3 to be installed.

Add the `wiregarden` service to `/etc/nsswitch.conf`. For example:

`hosts:          files dns wiregarden`

# Operation

Given a wiregarden interface up, such as:

```
root@m3:~# wiregarden status
Interface       Network         Address         Port    Peers   Status      
wgn001          civil-manatee   10.160.91.3/24  42501   3       interface_up

Network         Peer            Address         Endpoint                Key                                         
civil-manatee   m3 (this host)  10.160.91.3/24                          3loqTaoObb1cwHMCeiVGiUVLd5S2g8/6HAQVmGQEc1g=
civil-manatee   m4              10.160.91.4/24                          KGhcMQZ/Z3NFHRuotV6JX66a/wHVr92aznKBLLAFJXw=
civil-manatee   m2              10.160.91.2/24                          7kWe4PNriOs3InKODoP4fTWWmCZlckhtPHX259JkNz8=
civil-manatee   m1              10.160.91.1/24  10.149.84.66:51281      sDLG2DwVmnDQik9YuZwqS007+AWfMr0fxPrL6JtlkXc=
```

With nss-wiregarden you can resolve peers by `<host>.<network>`:

```
root@m3:~# ping m1.civil-manatee
PING m1.civil-manatee (10.160.91.1) 56(84) bytes of data.
64 bytes from m1.civil-manatee (10.160.91.1): icmp_seq=1 ttl=64 time=0.949 ms
```

# Security

The NSS plugin reads `/var/lib/wiregarden/db` in a read-only, restricted mode,
and queries the wiregarden local database for interface and peer info. By
default, this database is installed world-readable, as it only contains
interface information and state, not secrets.

Because the plugin code may execute from any process that resolves host names
with libc, it must be secure against local privilege escalation type attacks.
Rust was chosen for its excellent runtime safety characteristics, but the crate
dependencies also have not yet been throughly reviewed by the author.

[Contact wiregarden.io](https://wiregarden.io/contact) to report a security
vulnerability.

# Troubleshooting

If for any reason the plugin can't read the database or encounters an
unexpected error, it will not be able to resolve names, but other host name
resolvers should continue to work with a properly configured NSS. To debug why
wiregarden names aren't resolving, install a debug build of the library and
error messages will be printed to stderr.

---

Copyright 2020 Cmars Technologies LLC.
