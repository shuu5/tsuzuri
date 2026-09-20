//! `folio figure`（便 30・docs/design/delivery-30.md §1 (b)(c)）。設計ノートの正本（`design-note/<文書 id>.yaml`）の
//! 図の節（figures）の 1 枚を、型付き記述（spec）から図の道具（`vendor/archify`・rules 行 R-15）で検査と描画に掛け、
//! 図の本体（SVG）だけを 1 file に書く（--write）・検査する（--check）。
//!
//! 仕上がりの段は showcase 固定で、緩める旗を持たない（R-14）。検査を通らない図は生成せず、前の生成物も
//! 上書きしない（P-4.1・AC12）。往復（座標を直して撃ち直す）は folio の外＝planner が台帳に記帳する
//! （ADR-4 決定 (7)）。道具の他の命令（preview・brands capture・--open）は呼ばない。
//! 図の行 1 つから本体を描く口（`render`）は設計ノートの面（便 31）と共有する。
//!
//! 凍結 anchor（型付き記述 1 本と図の本体の写し・P-10.1）は compile 時に取り込み、命令と面の経路の導出の前に
//! 照合する。落ちたら判定を「まだ分からない」に落とし、出力も前の生成物も書かない（便 60・ADR-4 決定 (6)・P-10.3）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::face::{self, R, X};
use crate::face_note;
use crate::verdict::Verdict;
use crate::yaml::{self, Value};

/// 図の型の写像（β・正本の値 → 図の道具の型）。表に無い型は導出できない。
const FIGURE_TYPES: &[(&str, &str)] = &[
    ("archify-architecture", "architecture"),
    ("archify-workflow", "workflow"),
    ("archify-sequence", "sequence"),
    ("archify-dataflow", "dataflow"),
    ("archify-lifecycle", "lifecycle"),
];

/// 図の道具の置き場（正本の置き場の親 dir からの相対・便 28 の導出 file と同じ「親 dir」の規則）。
const TOOL: &str = "vendor/archify/bin/archify.mjs";

/// 仕上がりの段（R-14・旗を持たない）。
const QUALITY: &str = "showcase";

/// 道具の診断の欄が無いときに出す stdout の頭の字数。
const HEAD_CHARS: usize = 200;

/// 凍結 anchor の型付き記述（tests/fixtures/figure/anchor/spec.json の写し・compile 時に取り込む）。
const ANCHOR_SPEC: &str = include_str!("../../../tests/fixtures/figure/anchor/spec.json");

/// 凍結 anchor の図の本体（tests/fixtures/figure/anchor/body.svg の写し・compile 時に取り込む）。
const ANCHOR_BODY: &str = include_str!("../../../tests/fixtures/figure/anchor/body.svg");

/// 凍結 anchor の図の型（型付き記述の diagram_type と同じ）。
const ANCHOR_KIND: &str = "architecture";

/// 照合の結果（process の中で 1 回だけ計算する・道具の呼び出しを図ごとに増やさない）。
static ANCHOR: OnceLock<R<()>> = OnceLock::new();

pub enum Mode {
    Write,
    Check,
}

/// 1 回の実行の結果。`stdout` / `stderr` は 1 行ずつ。
pub struct Outcome {
    pub verdict: Verdict,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl Outcome {
    fn unknown(reason: impl Into<String>) -> Self {
        Outcome {
            verdict: Verdict::Unknown,
            stdout: None,
            stderr: Some(format!("folio figure: まだ分からない: {}", reason.into())),
        }
    }
}

// ── 命令の口（face.rs の run と同じ 3 値）──

pub fn run(doc: &str, id: &str, dir: &Path, out: &Path, mode: Mode) -> Outcome {
    // --out が相対なら --dir からの相対・絶対ならそのまま
    let out_path = dir.join(out);
    if !out_path.parent().is_some_and(Path::is_dir) {
        return Outcome::unknown(format!("{}: 出力先の親 dir が無い", out_path.display()));
    }
    // 凍結 anchor が落ちていれば --write / --check とも導出せず「まだ分からない」（P-10.3）
    if let Err(e) = anchor_holds(dir) {
        return Outcome::unknown(e);
    }
    let body = match derive(dir, doc, id) {
        Ok(b) => b,
        Err(e) => return Outcome::unknown(e),
    };
    let size = body.len();
    match mode {
        Mode::Check => {
            if !out_path.exists() {
                return Outcome {
                    verdict: Verdict::Unknown,
                    stdout: None,
                    stderr: Some("folio figure: 図が無い（未生成）".to_string()),
                };
            }
            let cur = match fs::read(&out_path) {
                Ok(b) => b,
                Err(e) => {
                    return Outcome::unknown(format!("{}: 読めない: {e}", out_path.display()));
                }
            };
            if cur != body.as_bytes() {
                return Outcome {
                    verdict: Verdict::Fail,
                    stdout: None,
                    stderr: Some(format!(
                        "folio figure: DRIFT — 図 {} byte ≠ 導出 {size} byte（手で直したか正本が変わった）",
                        cur.len()
                    )),
                };
            }
            Outcome {
                verdict: Verdict::Pass,
                stdout: Some(format!("folio figure: OK — 図は正本と一致（{size} byte）")),
                stderr: None,
            }
        }
        Mode::Write => {
            if let Err(e) = fs::write(&out_path, &body) {
                return Outcome::unknown(format!("{}: 書けない: {e}", out_path.display()));
            }
            Outcome {
                verdict: Verdict::Pass,
                stdout: Some(format!("folio figure: 書いた（{size} byte）")),
                stderr: None,
            }
        }
    }
}

// ── 生成器 ──

/// 設計ノートの図 1 枚の本体（SVG）を導出する。読めない・型が表に無い・道具が通らないは Err（まだ分からない）。
pub fn derive(dir: &Path, doc: &str, id: &str) -> R<String> {
    if !face_note::is_doc_id(doc) {
        return Err(format!(
            "--doc「{doc}」は id の形でない（英小文字で始まり 英小文字・数字・ハイフン）"
        ));
    }
    let name = format!("design-note/{doc}.yaml");
    let value = face::load(dir, &name)?;
    let d = X::root(&value, &name);
    let file_id = d.f("meta")?.f("id")?.id()?;
    if file_id != doc {
        return Err(format!(
            "{name}: 欄 meta.id「{file_id}」が --doc「{doc}」と違う"
        ));
    }
    let figs = match d.g("figures")? {
        Some(x) => x.seq()?,
        None => Vec::new(),
    };
    let mut hit = Vec::new();
    for fig in &figs {
        if fig.f("id")?.id()? == id {
            hit.push(fig);
        }
    }
    let fig = match hit.as_slice() {
        [one] => *one,
        [] => return Err(format!("{name}: 図「{id}」が figures に無い")),
        many => {
            return Err(format!(
                "{name}: 図「{id}」が figures に {} 本ある（図 id は 1 本）",
                many.len()
            ));
        }
    };
    let tx = fig.f("type")?;
    let kind =
        tx.v.as_str()
            .ok_or_else(|| format!("{}: 図の型が文字列でない", tx.at))?;
    render(dir, id, kind, &fig.f("spec")?)
}

/// 図の行 1 つ（型の字面と型付き記述）→ 図の本体（SVG）。型が表に無い・spec が表でない・道具が無い・道具が
/// 通らないは Err（まだ分からない）。設計ノートの面（`face_note.rs`・便 31）も図ごとにここを呼ぶ。
pub fn render(dir: &Path, id: &str, kind: &str, spec: &X<'_>) -> R<String> {
    let kind = tool_type(kind)?;
    if spec.v.as_map().is_none() {
        return Err(format!("{}: 型付き記述（spec）が表でない", spec.at));
    }
    let json = to_json(spec.v)?;
    let tool = tool_path(dir)?;
    anchor_holds(dir)?;
    deliver(&tool, kind, &json, id)
}

// ── 凍結 anchor の照合（P-10.1・P-10.3）──

/// 凍結 anchor が保たれているか。型付き記述の写しを既存の導出の経路（to_json → 道具へ deliver）に掛け、
/// 図の本体の写しと byte で比べる。違えば Err（道具の版か写しが変わった）。道具が起動できない・検査を
/// 通らないは既存の Err がそのまま上がる。結果は process の中で 1 回だけ計算し、以後は再利用する。
pub fn anchor_holds(dir: &Path) -> R<()> {
    ANCHOR.get_or_init(|| check_anchor(dir)).clone()
}

fn check_anchor(dir: &Path) -> R<()> {
    let spec = yaml::parse_typed(ANCHOR_SPEC)
        .map_err(|e| format!("凍結 anchor の型付き記述を読めない: {e}"))?;
    let json = to_json(&spec)?;
    let tool = tool_path(dir)?;
    let body = deliver(&tool, ANCHOR_KIND, &json, "anchor")?;
    if body.as_bytes() != ANCHOR_BODY.as_bytes() {
        return Err(
            "凍結 anchor が落ちた（図の道具の出力が固定の写しと違う・道具の版か写しが変わった。P-10.3）"
                .to_string(),
        );
    }
    Ok(())
}

/// 図の型（閉じた表 β）→ 図の道具の型。既存 3 型（pipeline-rail・context-band・state-strip）も道具の型でない。
fn tool_type(name: &str) -> R<&'static str> {
    FIGURE_TYPES
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, t)| *t)
        .ok_or_else(|| format!("図の型「{name}」は図の道具の型でない"))
}

/// 図の道具の path（正本の置き場の親 dir の下・symlink は認めない）。
fn tool_path(dir: &Path) -> R<PathBuf> {
    let parent = dir
        .parent()
        .ok_or_else(|| "正本の置き場の親 dir が無い".to_string())?;
    let path = parent.join(TOOL);
    if path.is_symlink() {
        return Err(format!(
            "{}: 図の道具が symlink（認めない）",
            path.display()
        ));
    }
    if !path.is_file() {
        return Err(format!("{}: 図の道具が無い", path.display()));
    }
    Ok(path)
}

// ── 型付き記述の書き出し（Value → JSON の 1 行・決定的）──

/// 正本の値を JSON の字面にする（空白と改行を入れない・順は正本に書かれた順）。
fn to_json(v: &Value) -> R<String> {
    match v {
        Value::Null => Ok("null".to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Int(s) => Ok(s.clone()),
        Value::Float(f) => Ok(f.to_string()),
        Value::Date(s) => Ok(format!("\"{s}\"")),
        Value::Str(s) => Ok(quote(s)),
        Value::Seq(items) => {
            let parts = items.iter().map(to_json).collect::<R<Vec<_>>>()?;
            Ok(format!("[{}]", parts.join(",")))
        }
        Value::Map(entries) => {
            let mut parts = Vec::with_capacity(entries.len());
            for (k, val) in entries {
                let key = k
                    .as_str()
                    .ok_or_else(|| "型付き記述の鍵が文字列でない".to_string())?;
                parts.push(format!("{}:{}", quote(key), to_json(val)?));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
    }
}

/// JSON の文字列（二重引用符と逆斜線と U+0000〜U+001F だけ escape・他は逐語・非 ASCII はそのまま）。
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ── 道具の呼び出し ──

/// 一時 dir を作って道具を 1 回撃ち、図の本体を返す。一時 dir は結果に関わらず消す。
fn deliver(tool: &Path, kind: &str, json: &str, id: &str) -> R<String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let td = std::env::temp_dir().join(format!("folio-figure-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&td).map_err(|e| format!("{}: 一時 dir を作れない: {e}", td.display()))?;
    let result = deliver_in(&td, tool, kind, json, id);
    let _ = fs::remove_dir_all(&td);
    result
}

fn deliver_in(td: &Path, tool: &Path, kind: &str, json: &str, id: &str) -> R<String> {
    let spec = td.join("spec.json");
    let out = td.join("figure.html");
    fs::write(&spec, json).map_err(|e| format!("型付き記述を書けない: {e}"))?;
    let run = Command::new("node")
        .arg(tool)
        .arg("deliver")
        .arg(kind)
        .arg(&spec)
        .arg(&out)
        .arg("--quality")
        .arg(QUALITY)
        .arg("--json")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("図の道具を起動できない（node）: {e}"))?;
    if !run.status.success() {
        let stdout = String::from_utf8_lossy(&run.stdout);
        return Err(format!(
            "図「{id}」は図の道具の検査を通らない: {}",
            error_head(&stdout)
        ));
    }
    let html = fs::read(&out).map_err(|e| format!("図の出力を読めない: {e}"))?;
    let html = String::from_utf8(html).map_err(|_| "図の出力が UTF-8 でない".to_string())?;
    body(&html)
}

/// 出力 html から図の本体（`<svg` から最初の `</svg>` の直後まで）を抜く。1 つでなければ Err。
fn body(html: &str) -> R<String> {
    let n = html.matches("<svg").count();
    if n != 1 {
        return Err(format!("図の本体が 1 つでない（{n}）"));
    }
    let start = html
        .find("<svg")
        .ok_or_else(|| "図の本体が 1 つでない（0）".to_string())?;
    let tail = &html[start..];
    let end = tail
        .find("</svg>")
        .ok_or_else(|| "図の本体に閉じ（</svg>）が無い".to_string())?;
    Ok(tail[..end + "</svg>".len()].to_string())
}

/// 道具の診断の頭（JSON の鍵 error の値の先頭 2 行を空白 1 つで繋ぐ・欄が無ければ stdout の先頭 200 字）。
fn error_head(stdout: &str) -> String {
    let Some(raw) = error_field(stdout) else {
        return stdout.chars().take(HEAD_CHARS).collect();
    };
    let lines = unescape_lines(raw);
    let head: Vec<&str> = lines.iter().take(2).map(|l| l.trim()).collect();
    head.join(" ")
}

/// JSON の鍵 error の値（escape されたまま）。鍵の後に「:」と二重引用符が続く箇所だけを見る。
fn error_field(stdout: &str) -> Option<&str> {
    let key = "\"error\"";
    let mut rest = stdout;
    loop {
        let i = rest.find(key)?;
        let after = &rest[i + key.len()..];
        if let Some(tail) = after.trim_start().strip_prefix(':')
            && let Some(value) = tail.trim_start().strip_prefix('"')
            && let Some(end) = string_end(value)
        {
            return value.get(..end);
        }
        rest = after;
    }
}

/// escape を跨いで閉じの二重引用符までの byte 数。
fn string_end(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

/// escape された文字列を `\n` で行に分け、行ごとに escape を解く。
fn unescape_lines(raw: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cs = raw.chars();
    while let Some(c) = cs.next() {
        if c != '\\' {
            cur.push(c);
            continue;
        }
        match cs.next() {
            Some('n') => {
                lines.push(std::mem::take(&mut cur));
            }
            Some('t') => cur.push('\t'),
            Some('r') => cur.push('\r'),
            Some('b') => cur.push('\u{8}'),
            Some('f') => cur.push('\u{c}'),
            Some('u') => {
                let hex: String = cs.by_ref().take(4).collect();
                if let Ok(n) = u32::from_str_radix(&hex, 16)
                    && let Some(c) = char::from_u32(n)
                {
                    cur.push(c);
                }
            }
            // 二重引用符・逆斜線・斜線はその字
            Some(other) => cur.push(other),
            None => {}
        }
    }
    lines.push(cur);
    lines
}

#[cfg(test)]
mod figure_tests {
    use super::*;

    #[test]
    fn figure_to_json_writes_every_type_in_the_written_order() {
        assert_eq!(to_json(&Value::Null).unwrap(), "null");
        assert_eq!(to_json(&Value::Bool(true)).unwrap(), "true");
        assert_eq!(to_json(&Value::Bool(false)).unwrap(), "false");
        assert_eq!(to_json(&Value::Int("-70".to_string())).unwrap(), "-70");
        assert_eq!(to_json(&Value::Float(1.5)).unwrap(), "1.5");
        assert_eq!(
            to_json(&Value::Date("2026-09-19".to_string())).unwrap(),
            "\"2026-09-19\""
        );
        let v = yaml::parse_typed("a: [1, x]\nb: {c: 0}\n").unwrap();
        assert_eq!(to_json(&v).unwrap(), "{\"a\":[1,\"x\"],\"b\":{\"c\":0}}");
        // 順は正本に書かれた順（辞書順に直さない）
        let v = yaml::parse_typed("b: 1\na: 2\n").unwrap();
        assert_eq!(to_json(&v).unwrap(), "{\"b\":1,\"a\":2}");
    }

    #[test]
    fn figure_to_json_escapes_only_the_quote_the_backslash_and_the_controls() {
        assert_eq!(to_json(&Value::Str("持ち主".into())).unwrap(), "\"持ち主\"");
        assert_eq!(to_json(&Value::Str("a\"b".into())).unwrap(), "\"a\\\"b\"");
        assert_eq!(to_json(&Value::Str("a\\b".into())).unwrap(), "\"a\\\\b\"");
        assert_eq!(to_json(&Value::Str("a\nb".into())).unwrap(), "\"a\\nb\"");
        assert_eq!(
            to_json(&Value::Str("a\u{1}b\tc".into())).unwrap(),
            "\"a\\u0001b\\tc\""
        );
        assert_eq!(to_json(&Value::Str("<&>'".into())).unwrap(), "\"<&>'\"");
    }

    #[test]
    fn figure_to_json_refuses_a_map_key_that_is_not_a_string() {
        let v = yaml::parse_typed("1: x\n").unwrap();
        assert_eq!(
            to_json(&v).unwrap_err(),
            "型付き記述の鍵が文字列でない".to_string()
        );
    }

    #[test]
    fn figure_body_takes_exactly_one_svg_verbatim() {
        assert_eq!(
            body("<html><svg class=\"x\">あ</svg>\n</html>").unwrap(),
            "<svg class=\"x\">あ</svg>"
        );
        assert_eq!(
            body("<html></html>").unwrap_err(),
            "図の本体が 1 つでない（0）"
        );
        assert_eq!(
            body("<svg></svg><svg></svg>").unwrap_err(),
            "図の本体が 1 つでない（2）"
        );
        assert_eq!(
            body("<svg>閉じが無い").unwrap_err(),
            "図の本体に閉じ（</svg>）が無い"
        );
    }

    #[test]
    fn figure_error_head_joins_the_first_two_lines() {
        let out = "{\n \"ok\": false,\n \"error\": \"Architecture layout validation failed:\\n  label \\\"あ\\\" overlaps; adjust labelDx/labelDy or set labelAt\\n  3 more\",\n \"diagnostics\": []\n}";
        assert_eq!(
            error_head(out),
            "Architecture layout validation failed: label \"あ\" overlaps; adjust labelDx/labelDy or set labelAt"
        );
        // 1 行しか無ければその 1 行
        assert_eq!(error_head("{\"error\": \"1 行だけ\"}"), "1 行だけ");
        // 欄が無ければ stdout の先頭 200 字（severity の値の error は鍵でない）
        let no_field = "x".repeat(300) + "\"error\"";
        assert_eq!(error_head(&no_field), "x".repeat(HEAD_CHARS));
        assert_eq!(
            error_head("{\"severity\": \"error\"}"),
            "{\"severity\": \"error\"}"
        );
    }

    #[test]
    fn figure_anchor_constants_are_the_frozen_pair() {
        // 型付き記述は既存の読み手で表として読め、型は道具の型 architecture
        let spec = yaml::parse_typed(ANCHOR_SPEC).unwrap();
        let x = X::root(&spec, "anchor");
        assert_eq!(x.f("diagram_type").unwrap().v.as_str(), Some(ANCHOR_KIND));
        assert!(
            to_json(&spec)
                .unwrap()
                .starts_with("{\"schema_version\":1,")
        );
        // 図の本体の写しは svg 1 つ
        assert_eq!(body(ANCHOR_BODY).unwrap(), ANCHOR_BODY);
    }

    #[test]
    fn figure_type_table_has_the_five_tool_types() {
        assert_eq!(tool_type("archify-architecture").unwrap(), "architecture");
        assert_eq!(tool_type("archify-lifecycle").unwrap(), "lifecycle");
        assert_eq!(FIGURE_TYPES.len(), 5);
        assert_eq!(
            tool_type("pipeline-rail").unwrap_err(),
            "図の型「pipeline-rail」は図の道具の型でない"
        );
    }

    #[test]
    fn figure_render_refuses_a_spec_that_is_not_a_map_before_touching_the_tool() {
        // 型の表 → spec の形 の順に見る（道具の不在より前に断る）
        let v = yaml::parse_typed("spec: 表でない\n").unwrap();
        let spec = X::root(&v, "fig").f("spec").unwrap();
        assert_eq!(
            render(
                Path::new("/nonexistent/src"),
                "fig-1",
                "archify-architecture",
                &spec
            )
            .unwrap_err(),
            "fig.spec: 型付き記述（spec）が表でない"
        );
        assert_eq!(
            render(
                Path::new("/nonexistent/src"),
                "fig-1",
                "context-band",
                &spec
            )
            .unwrap_err(),
            "図の型「context-band」は図の道具の型でない"
        );
    }
}
