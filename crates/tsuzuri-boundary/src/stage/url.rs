//! board の URL（行 i-4・要件 FR16・判断の記録 ADR-15 の決定 (5)・持ち主の裁定 t3-hub.59.4 と t3-hub.53.1）。
//! ssh で届かない端末には tailnet の中の board の URL を渡すだけにする。host の名は tailnet の道具の status の
//! Self の DNSName から、port は anchor の追跡されない git config の tsuzuri.boardport から実行の時に読み、
//! どちらも code と版管理に載る file に書かない（行 D-4）。
//! 自分の board とみなすのは、port が自分の boardport で、host がこの host のどの形（tailnet の名・その短い名・
//! Self の TailscaleIPs の住所・127.0.0.1・localhost）でもよい頁（席の決め・i-4 と i-5 の起草の問い Q1）。

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::time::Duration;

use super::json;
use crate::acct;
use crate::server::proc;

/// 既定の tailnet の道具の名（PATH で引く）。
pub const TAILNET: &str = "tailscale";

/// tailnet の道具の status の引数。
pub const STATUS_ARGS: [&str; 2] = ["status", "--json"];

/// この host の loopback の形（自分の board の host の列の末）。
pub const LOOPBACK: [&str; 2] = ["127.0.0.1", "localhost"];

/// status の Self の DNSName（末の点を 1 つ除いて ASCII の小文字・点で分けた段がどれも空でなく英数字と - だけの時だけ）。
pub fn self_name(status: &str) -> Option<String> {
    let me = json::member(status, "Self")?;
    let name = json::member(me, "DNSName").and_then(json::unquote)?;
    let name = name.strip_suffix('.').unwrap_or(&name).to_ascii_lowercase();
    name.split('.')
        .all(|part| {
            !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
        .then_some(name)
}

/// この host の形の列（名・短い名・TailscaleIPs の住所・LOOPBACK の順・同じ字は 1 度）。
/// 名が読めないか、TailscaleIPs が字の配列でないか要素が住所の字でなければ None。
pub fn self_hosts(status: &str) -> Option<Vec<String>> {
    let name = self_name(status)?;
    let short = name.split('.').next().unwrap_or(&name).to_string();
    let mut hosts = vec![name, short];
    let me = json::member(status, "Self")?;
    if let Some(ips) = json::member(me, "TailscaleIPs") {
        for item in json::items(ips)? {
            let ip = json::unquote(item)?.to_ascii_lowercase();
            let shaped = !ip.is_empty()
                && ip
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || c == '.' || c == ':');
            if !shaped {
                return None;
            }
            hosts.push(ip);
        }
    }
    hosts.extend(LOOPBACK.iter().map(|h| h.to_string()));
    let mut out: Vec<String> = Vec::new();
    for host in hosts {
        if !out.contains(&host) {
            out.push(host);
        }
    }
    Some(out)
}

/// host と port の board の URL。
pub fn board_url(host: &str, port: u16) -> String {
    format!("http://{host}:{port}/")
}

/// 自分の board（渡す URL・この host の形の列・boardport）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub url: String,
    pub hosts: Vec<String>,
    pub port: u16,
}

impl Board {
    /// 頁の URL が自分の board か（port が自分の port で、host がこの host のどれかの形・scheme は http か https）。
    pub fn holds(&self, page: &str) -> bool {
        parts(page).is_some_and(|(_, host, port)| port == self.port && self.hosts.contains(&host))
    }
}

/// tailnet の道具の status と anchor の git config から自分の board を組む（どちらも cwd は repo）。
pub fn board(tailnet: &OsStr, git: &OsStr, repo: &Path, timeout: Duration) -> Result<Board, String> {
    let tool = tailnet.to_string_lossy();
    let status = proc::capture(tailnet, STATUS_ARGS, repo, timeout).ok_or_else(|| {
        format!("tailnet の道具 {tool} の status --json が {} 秒の内に rc 0 で返らない", timeout.as_secs_f32())
    })?;
    let hosts = self_hosts(&String::from_utf8_lossy(&status)).ok_or_else(|| {
        format!("tailnet の道具 {tool} の status の Self の DNSName か TailscaleIPs が読めない")
    })?;
    let mut args = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    args.extend(acct::BOARD_ARGS.iter().map(OsString::from));
    let port = proc::capture(git, &args, repo, timeout)
        .and_then(|out| acct::board_port(&String::from_utf8_lossy(&out)))
        .ok_or_else(|| {
            format!(
                "{} の git config の tsuzuri.boardport が読めない（無いか 1 から 65535 の数でない）",
                repo.display()
            )
        })?;
    Ok(Board {
        url: board_url(&hosts[0], port),
        hosts,
        port,
    })
}

/// 持ち主へ渡す出力の最後の 1 行。
pub fn line(url: &str) -> String {
    format!("board の URL {url}")
}

/// URL の scheme と host と port の字（http と https だけ・host は小文字で末の点と IPv6 の角括弧を除く）。
pub fn origin(url: &str) -> Option<String> {
    let (scheme, host, port) = parts(url)?;
    if host.contains(':') {
        Some(format!("{scheme}://[{host}]:{port}"))
    } else {
        Some(format!("{scheme}://{host}:{port}"))
    }
}

/// URL の scheme と host と port（port が無いか空なら scheme の既定の port）。
fn parts(url: &str) -> Option<(String, String, u16)> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    let default = match scheme.as_str() {
        "http" => 80,
        "https" => 443,
        _ => return None,
    };
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    let place = authority.rsplit_once('@').map_or(authority, |(_, place)| place);
    let (host, port) = match place.strip_prefix('[') {
        Some(inner) => {
            let (host, after) = inner.split_once(']')?;
            if after.is_empty() {
                (host, None)
            } else {
                (host, Some(after.strip_prefix(':')?))
            }
        }
        None => match place.rsplit_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (place, None),
        },
    };
    let host = host.strip_suffix('.').unwrap_or(host).to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }
    let port = match port {
        None | Some("") => default,
        Some(digits) if digits.bytes().all(|b| b.is_ascii_digit()) => digits.parse().ok()?,
        Some(_) => return None,
    };
    Some((scheme, host, port))
}
