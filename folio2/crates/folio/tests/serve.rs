//! `folio serve` の歯（便 17・docs/design/delivery-17.md §1 (d)）。binary 経由。
//! - 拒む: tailnet の外の bind 先・配信先が無い・入口が無い・版管理が在る
//! - `--host` 無しの起動（FR7 の確かめ方・環境に tailnet が在るか無いかで枝が分かれるが、どちらの枝も断定する）
//! - loopback での配信 11 要求（見せる file・閉じ込め・method・拡張子の表）
//!
//! 版管理へは この host の tailnet の住所・機器名・口座名 を書かない（D-8）。
//! ここに書く住所は loopback・範囲の境界の値・公開の例の住所だけ。

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> PathBuf {
    repo_root().join("tests/fixtures/face")
}

fn vendor() -> PathBuf {
    repo_root().join("vendor/archify")
}

fn temp_dir(case: &str) -> PathBuf {
    let td = std::env::temp_dir().join(format!("folio-serve-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&td);
    fs::create_dir_all(&td).unwrap();
    td
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn code(out: &Output, what: &str) -> i32 {
    out.status
        .code()
        .unwrap_or_else(|| panic!("{what} が signal で終わった: {}", stderr(out)))
}

/// 凍結 fixture の写しから配信先を組み立てる。戻り値 = (一時 dir, 配信先)。
fn built_site(case: &str) -> (PathBuf, PathBuf) {
    let td = temp_dir(case);
    let work = td.join("src");
    fs::create_dir_all(work.join("preview")).unwrap();
    fs::create_dir_all(work.join("adr")).unwrap();
    fs::create_dir_all(work.join("design-note")).unwrap();
    fs::create_dir_all(td.join("contracts")).unwrap();
    for name in [
        "constitution.yaml",
        "rules.yaml",
        "vocabulary.yaml",
        "srs.yaml",
        "index.yaml",
        "intake.yaml",
        "ceiling.yaml",
    ] {
        fs::copy(fixture().join(name), work.join(name)).unwrap();
    }
    // 組み立ては判断の記録の面も出す（便 26）ので、写す記録は欄の揃った便 25 の 1 本
    fs::copy(
        fixture().join("adr/ADR-2.yaml"),
        work.join("adr/ADR-2.yaml"),
    )
    .unwrap();
    // 組み立ては設計ノートの面も出す（便 29）ので、写す設計ノートは欄の揃った便 28 の 1 本。
    // その面は契約表の節を持つので、器の導出 file を写しの src/ の親 dir へも置く
    fs::copy(
        fixture().join("design-note/full.yaml"),
        work.join("design-note/full.yaml"),
    )
    .unwrap();
    fs::copy(
        repo_root().join("contracts/schema.toml"),
        td.join("contracts/schema.toml"),
    )
    .unwrap();
    for name in ["folio.css", "folio-ui.js"] {
        fs::copy(fixture().join(name), work.join("preview").join(name)).unwrap();
    }
    // 設計ノートの面は図ごとに図の道具を撃つ（便 31）ので、repo の vendor/archify/ も親 dir へ写す
    copy_dir(&vendor(), &td.join("vendor/archify"));
    let site = td.join("site");
    let build = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("build")
        .arg("--dir")
        .arg(&work)
        .arg("--out")
        .arg(&site)
        .arg("--write")
        .output()
        .unwrap();
    assert_eq!(code(&build, "folio build --write"), 0, "{}", stderr(&build));
    (td, site)
}

/// 起動して終わるはずの serve（拒むとき）。
fn folio_serve(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("serve")
        .args(args)
        .output()
        .expect("folio を起動できない")
}

/// 起動して配信に入るはずの serve。戻り値 = (子 process, 標準出力の 1 行)。
/// 5 秒のうちに 1 行が出なければ（＝起動を拒んだなら）1 行は空。
fn spawn_serve(args: &[&str]) -> (Child, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_folio"))
        .arg("serve")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("folio を起動できない");
    let stdout = child.stdout.take().expect("標準出力を取れない");
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(stdout).read_line(&mut line);
        let _ = tx.send(line);
    });
    let line = rx.recv_timeout(Duration::from_secs(5)).unwrap_or_default();
    (child, line)
}

/// 起動の 1 行から bind 先（<住所>:<port>）を取る。
fn address_of(line: &str) -> SocketAddr {
    let start = line
        .find("http://")
        .unwrap_or_else(|| panic!("起動の 1 行の形が違う: {line}"))
        + "http://".len();
    let rest = &line[start..];
    let end = rest
        .find("/index.html")
        .unwrap_or_else(|| panic!("起動の 1 行の形が違う: {line}"));
    rest[..end].parse().expect("住所として読めない")
}

/// 1 接続 = 1 要求。戻り値 = (状態の数, 頭, 本文)。
fn request(addr: &SocketAddr, raw: &str) -> (u16, String, Vec<u8>) {
    let mut s = TcpStream::connect(addr).expect("配信器へ繋がらない");
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    s.write_all(raw.as_bytes()).unwrap();
    s.flush().unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).expect("応答を読めない");
    let split = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap_or_else(|| panic!("応答に空行が無い: {}", String::from_utf8_lossy(&buf)));
    let head = String::from_utf8_lossy(&buf[..split]).into_owned();
    let body = buf[split + 4..].to_vec();
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split(' ').nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("状態行が読めない: {head}"));
    (status, head, body)
}

fn get(addr: &SocketAddr, target: &str) -> (u16, String, Vec<u8>) {
    request(
        addr,
        &format!("GET {target} HTTP/1.1\r\nHost: folio\r\nConnection: close\r\n\r\n"),
    )
}

// ── 拒む ──

#[test]
fn serve_refuses_a_bind_host_outside_the_tailnet() {
    let (td, site) = built_site("outside");
    for host in ["0.0.0.0", "192.168.0.1", "8.8.8.8"] {
        let run = folio_serve(&["--dir", site.to_str().unwrap(), "--host", host]);
        assert_eq!(code(&run, "folio serve"), 1, "{host}: {}", stderr(&run));
        assert!(
            stderr(&run).contains("tailnet の外"),
            "{host}: {}",
            stderr(&run)
        );
    }
    let _ = fs::remove_dir_all(&td);
}

#[test]
fn serve_is_unknown_when_the_dir_is_missing() {
    let td = temp_dir("no-dir");
    let run = folio_serve(&[
        "--dir",
        td.join("no-such-dir").to_str().unwrap(),
        "--host",
        "127.0.0.1",
    ]);
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio serve"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("配信先が無い"), "{}", stderr(&run));
}

#[test]
fn serve_is_unknown_when_the_entrance_is_missing() {
    let td = temp_dir("no-entrance");
    let site = td.join("site");
    fs::create_dir(&site).unwrap();
    fs::write(site.join("srs.html"), "<!DOCTYPE html>\n").unwrap();
    let run = folio_serve(&["--dir", site.to_str().unwrap(), "--host", "127.0.0.1"]);
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio serve"), 2, "{}", stderr(&run));
    assert!(stderr(&run).contains("入口"), "{}", stderr(&run));
}

#[test]
fn serve_refuses_a_dir_that_holds_version_control() {
    let (td, site) = built_site("git");
    fs::create_dir(site.join(".git")).unwrap();
    let run = folio_serve(&["--dir", site.to_str().unwrap(), "--host", "127.0.0.1"]);
    let _ = fs::remove_dir_all(&td);
    assert_eq!(code(&run, "folio serve"), 1, "{}", stderr(&run));
    assert!(stderr(&run).contains("版管理か台帳"), "{}", stderr(&run));
}

#[test]
fn serve_without_a_host_binds_the_tailnet_address_or_refuses() {
    let (td, site) = built_site("auto");
    let (mut child, line) = spawn_serve(&["--dir", site.to_str().unwrap()]);
    if line.starts_with("folio serve: http://") {
        // tailnet が在る環境: bind 先は 100.64.0.0/10 の中
        let addr = address_of(&line);
        let ip = match addr.ip() {
            std::net::IpAddr::V4(ip) => ip,
            other => panic!("IPv4 でない住所: {other}"),
        };
        let [a, b, _, _] = ip.octets();
        let _ = child.kill();
        let _ = child.wait();
        let _ = fs::remove_dir_all(&td);
        assert!(
            a == 100 && (64..128).contains(&b),
            "bind 先が tailnet の範囲の外: {line}"
        );
    } else {
        // tailnet が無い環境: fail-closed で拒む
        let out = child.wait_with_output().unwrap();
        let _ = fs::remove_dir_all(&td);
        assert_eq!(
            code(&out, "folio serve（--host 無し）"),
            1,
            "{}",
            stderr(&out)
        );
        assert!(
            stderr(&out).contains("tailnet の住所が無い"),
            "{}",
            stderr(&out)
        );
    }
}

// ── 見せる（loopback）──

#[test]
fn serve_shows_the_pages_over_loopback() {
    let (td, site) = built_site("show");
    fs::write(site.join(".hidden.txt"), "隠し\n").unwrap();
    fs::create_dir(site.join("sub")).unwrap();
    fs::write(site.join("sub/a.txt"), "下位\n").unwrap();
    let outside = td.join("outside.txt");
    fs::write(&outside, "配信先の外\n").unwrap();
    std::os::unix::fs::symlink(&outside, site.join("out.txt")).unwrap();

    let index = fs::read(site.join("index.html")).unwrap();
    let srs_len = fs::metadata(site.join("srs.html")).unwrap().len();

    let loopback = Ipv4Addr::LOCALHOST.to_string();
    let (mut child, line) = spawn_serve(&[
        "--dir",
        site.to_str().unwrap(),
        "--host",
        &loopback,
        "--port",
        "0",
    ]);
    assert!(
        line.starts_with("folio serve: http://"),
        "起動していない: {line}"
    );
    let addr = address_of(&line);

    let entrance = get(&addr, "/index.html");
    let root = get(&addr, "/");
    let css = get(&addr, "/folio.css");
    let head_srs = request(
        &addr,
        "HEAD /srs.html HTTP/1.1\r\nHost: folio\r\nConnection: close\r\n\r\n",
    );
    let nothing = get(&addr, "/nothing.html");
    let hidden = get(&addr, "/.hidden.txt");
    let dir = get(&addr, "/sub");
    let out_link = get(&addr, "/out.txt");
    let up = get(&addr, "/../Cargo.toml");
    let post = request(
        &addr,
        "POST /index.html HTTP/1.1\r\nHost: folio\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
    );
    let bare = request(
        &addr,
        "GET index.html HTTP/1.1\r\nHost: folio\r\nConnection: close\r\n\r\n",
    );

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&td);

    assert_eq!(entrance.0, 200, "{}", entrance.1);
    assert!(
        entrance
            .1
            .contains("Content-Type: text/html; charset=utf-8"),
        "{}",
        entrance.1
    );
    assert!(
        entrance
            .1
            .contains(&format!("Content-Length: {}", index.len())),
        "{}",
        entrance.1
    );
    assert!(entrance.2 == index, "入口の本文が file と byte で違う");
    assert_eq!(root.0, 200, "{}", root.1);
    assert!(root.2 == index, "「/」の本文が入口と違う");
    assert_eq!(css.0, 200, "{}", css.1);
    assert!(
        css.1.contains("Content-Type: text/css; charset=utf-8"),
        "{}",
        css.1
    );
    assert_eq!(head_srs.0, 200, "{}", head_srs.1);
    assert!(
        head_srs.1.contains(&format!("Content-Length: {srs_len}")),
        "{}",
        head_srs.1
    );
    assert!(head_srs.2.is_empty(), "HEAD に本文が付いた");
    assert_eq!(nothing.0, 404, "{}", nothing.1);
    assert_eq!(hidden.0, 404, "{}", hidden.1);
    assert_eq!(dir.0, 404, "{}", dir.1);
    assert_eq!(out_link.0, 404, "{}", out_link.1);
    assert_eq!(up.0, 404, "{}", up.1);
    assert_eq!(post.0, 405, "{}", post.1);
    assert!(
        String::from_utf8_lossy(&post.2).contains("GET と HEAD だけ"),
        "{}",
        String::from_utf8_lossy(&post.2)
    );
    assert_eq!(bare.0, 400, "{}", bare.1);
}
