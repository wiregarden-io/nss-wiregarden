extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate libnss;

extern crate debug_print;

use libnss::host::{AddressFamily, Addresses, Host, HostHooks};
use libnss::interop::Response;

use rusqlite::{params, Connection, Error, OpenFlags, Result};

use debug_print::debug_eprintln;

use std::net::{IpAddr, Ipv4Addr};

static DBPATH: &'static str = "/var/lib/wiregarden/db";

fn dbflags() -> OpenFlags {
    OpenFlags::SQLITE_OPEN_READ_ONLY
        | OpenFlags::SQLITE_OPEN_NOFOLLOW
        | OpenFlags::SQLITE_OPEN_PRIVATE_CACHE
        | OpenFlags::SQLITE_OPEN_FULL_MUTEX
}

struct WiregardenHost;
libnss_host_hooks!(wiregarden, WiregardenHost);

impl HostHooks for WiregardenHost {
    fn get_all_entries() -> Response<Vec<Host>> {
        match get_all_entries() {
            Ok(hosts) => {
                if hosts.is_empty() {
                    Response::NotFound
                } else {
                    Response::Success(hosts)
                }
            }
            Err(Error::QueryReturnedNoRows) => Response::NotFound,
            Err(_e) => {
                debug_eprintln!("get_all_entries failed: {}", _e);
                Response::Unavail
            }
        }
    }

    fn get_host_by_addr(addr: IpAddr) -> Response<Host> {
        match get_host_by_addr(addr) {
            Ok(Some(host)) => Response::Success(host),
            Ok(None) => Response::NotFound,
            Err(Error::QueryReturnedNoRows) => Response::NotFound,
            Err(_e) => {
                debug_eprintln!("get_host_by_addr {} failed: {}", addr, _e);
                Response::Unavail
            }
        }
    }

    fn get_host_by_name(name: &str, family: AddressFamily) -> Response<Host> {
        if family != AddressFamily::IPv4 {
            Response::NotFound
        } else {
            match get_host_by_name(name) {
                Ok(Some(host)) => Response::Success(host),
                Ok(None) => Response::NotFound,
                Err(Error::QueryReturnedNoRows) => Response::NotFound,
                Err(_e) => {
                    debug_eprintln!("get_host_by_name {} failed: {}", name, _e);
                    Response::Unavail
                }
            }
        }
    }
}

fn get_all_entries() -> Result<Vec<Host>> {
    let mut hosts = vec![];
    let db = Connection::open_with_flags(&DBPATH, dbflags())?;
    let mut stmt = db.prepare(
        "
select device_name, net_name, device_addr
from iface
union
select p.device_name, i.net_name, p.device_addr
from iface i join peer p on (i.id = p.iface_id",
    )?;
    stmt.query_map(params![], |row| {
        let device_name: String = row.get(0)?;
        let net_name: String = row.get(1)?;
        let device_addr_s: String = row.get(2)?;
        let device_addr: std::result::Result<Ipv4Addr, _> = trim_subnet(&device_addr_s).parse();
        match device_addr {
            Ok(v4addr) => {
                hosts.push(Host {
                    name: format!("{}.{}", device_name, net_name),
                    addresses: Addresses::V4(vec![v4addr]),
                    aliases: vec![],
                });
            }
            _ => {}
        };
        Ok(())
    })?;
    Ok(hosts)
}

fn get_host_by_addr(addr: IpAddr) -> Result<Option<Host>> {
    match addr {
        IpAddr::V4(v4addr) => {
            let db = Connection::open_with_flags(&DBPATH, dbflags())?;
            let addr_s = format!("{}", addr);
            let mut stmt = db.prepare(
                "
select device_name, net_name
from iface
where device_addr like ? || '/%'
union
select p.device_name, i.net_name
from iface i join peer p on (i.id = p.iface_id)
where p.device_addr like ? || '/%'",
            )?;
            stmt.query_row(params![addr_s, addr_s], |row| {
                let device_name: String = row.get(0)?;
                let net_name: String = row.get(1)?;
                Ok(Some(Host {
                    name: format!("{}.{}", device_name, net_name),
                    addresses: Addresses::V4(vec![v4addr]),
                    aliases: vec![],
                }))
            })
        }
        _ => Ok(None),
    }
}

fn get_host_by_name(name: &str) -> Result<Option<Host>> {
    let db = Connection::open_with_flags(&DBPATH, dbflags())?;
    let mut stmt = db.prepare(
        "
select device_name, net_name, device_addr
from iface
where device_name || '.' || net_name = ?
union
select p.device_name, i.net_name, p.device_addr
from iface i join peer p on (i.id = p.iface_id)
where p.device_name || '.' || i.net_name = ?",
    )?;
    stmt.query_row(params![name, name], |row| {
        let device_name: String = row.get(0)?;
        let net_name: String = row.get(1)?;
        let device_addr_s: String = row.get(2)?;
        let device_addr: std::result::Result<Ipv4Addr, _> = trim_subnet(&device_addr_s).parse();
        match device_addr {
            Ok(v4addr) => Ok(Some(Host {
                name: format!("{}.{}", device_name, net_name),
                addresses: Addresses::V4(vec![v4addr]),
                aliases: vec![],
            })),
            _ => Ok(None),
        }
    })
}

fn trim_subnet(s: &str) -> &str {
    match s.rfind("/") {
        Some(ri) => &s[0..ri],
        None => s,
    }
}
