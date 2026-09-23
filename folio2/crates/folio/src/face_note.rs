//! 設計ノートの面の生成器（便 28・docs/design/delivery-28.md §1 (a)〜(d)）。設計ノートの正本 1 本
//! （`design-note/<文書 id>.yaml`）から人が読むページ 1 枚を組む。生成物の文字列は 正本の値（α・escape して逐語）・
//! 名札の表（β・この file の表）・正本から数えた数（γ）のどれかで、案内の文は出さない。
//! 面に依らない口（head と site-bar・章の帯・card・toc・承認欄の帯・foot・部品の名札）は `face.rs` の `Frame` を呼ぶ。
//! 判断の記録の面（`face_adr.rs`）と違うのは 章が正本ごとに増える点で、章の数は節の数（+ 図の章）から数えて
//! `Frame` を関数の中で組む。節の型は閉じた一覧 6 つで、一覧に無い型が 1 つでもあれば面を導出しない（要件書 FR9）。
//! 契約表の節の欄は器（scribe2）の導出 file `contracts/schema.toml` の field の順に出し、欄の一覧を自前に持たない（FR10・P-6.4）。
//! 図の章（便 31）は図ごとに図の枠（figure-panel・便 34 からは `face.rs` の共有の口）を置き、図の本体（SVG）は
//! `figure.rs` の `render` が図の道具で描いたものを逐語で埋める。図が 1 枚でも導出できなければ面全体を導出しない
//! （全部か無しか・FR15）。

use std::fs;
use std::path::Path;

use crate::catalog::Component;
use crate::cursor::{self, R, X, esc};
use crate::face::{self, Frame, anchor, hint};
use crate::face_index;

/// 設計ノートの面が使う部品（8 種・判断の記録の面の section-lead-callout の代わりに figure-panel・便 40 で ceiling-stamp を足した）。
pub const PARTS: [Component; 8] = [
    Component::FreshnessStamp,
    Component::CeilingStamp,
    Component::FontSizeControl,
    Component::DocCoverBand,
    Component::ApprovalBlock,
    Component::ChapterDeckBand,
    Component::FigurePanel,
    Component::ItemRow,
];

/// 章の帯の class と kicker の絵記号。class は band-1〜6 を 2 回・絵記号は要件書の面の 6 つを同じ順で回す。
/// 章の数だけ先頭から切り出して `Frame` に渡す（`static` なので切り出しは 'static）。
static BANDS: [(&str, &str); 12] = [
    (
        "band-1",
        "<circle cx=\"12\" cy=\"12\" r=\"9\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>",
    ),
    (
        "band-2",
        "<path d=\"M3 7l9-4 9 4-9 4-9-4z\"/><path d=\"M3 7v10l9 4 9-4V7\"/>",
    ),
    ("band-3", "<path d=\"M20 6L9 17l-5-5\"/>"),
    ("band-4", "<path d=\"M4 18h16M6 14V8M12 14V4M18 14v-4\"/>"),
    (
        "band-5",
        "<path d=\"M9 11l3 3L22 4\"/><path d=\"M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11\"/>",
    ),
    (
        "band-6",
        "<path d=\"M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z\"/>",
    ),
    (
        "band-1",
        "<circle cx=\"12\" cy=\"12\" r=\"9\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>",
    ),
    (
        "band-2",
        "<path d=\"M3 7l9-4 9 4-9 4-9-4z\"/><path d=\"M3 7v10l9 4 9-4V7\"/>",
    ),
    ("band-3", "<path d=\"M20 6L9 17l-5-5\"/>"),
    ("band-4", "<path d=\"M4 18h16M6 14V8M12 14V4M18 14v-4\"/>"),
    (
        "band-5",
        "<path d=\"M9 11l3 3L22 4\"/><path d=\"M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11\"/>",
    ),
    (
        "band-6",
        "<path d=\"M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z\"/>",
    ),
];

/// 章の上限（帯の表の長さ・承認欄は数えない）。
const MAX_CHAPTERS: usize = 12;

/// 判断の記録の面の FRAME の favicon の字面。
const FAVICON: &str = "<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%235f45a6'/%3E%3Ctext x='16' y='22' font-size='16' font-weight='700' text-anchor='middle' fill='%23ffffff' font-family='sans-serif'%3E要%3C/text%3E%3C/svg%3E\">";

// ── 名札の表（β・表に無い値は導出できない）──

/// 節の型（閉じた一覧 6 つ）→ 章の名札。並びは欄の決まりの type_enum と同じ。
const TYPES: [(&str, &str); 6] = [
    ("prose", "説明"),
    ("parts-table", "部品"),
    ("ports-table", "口"),
    ("fields-table", "欄"),
    ("teeth-table", "検査"),
    ("contract-table", "契約表"),
];

const CONTRACT_TABLE: &str = "contract-table";

/// 文書の状態 → 名札（状態の行は関数 status）。
pub(crate) const STATUS: &[(&str, &str)] = &[
    ("draft", "下書き"),
    ("effective", "発効"),
    ("retired", "廃止"),
    ("example", "見本"),
];

/// 欄の要否 → pill の字。
const NEED: &[(&str, &str)] = &[("required", "必須"), ("optional", "任意")];

/// 欄の形 → pill の字。
const SHAPE: &[(&str, &str)] = &[
    ("text", "文字"),
    ("list", "一覧"),
    ("number", "数"),
    ("bool", "真偽"),
    ("table", "表"),
];

/// 検査の行の red_when の名札（欄の決まり row_note = 何を壊せば落ちるか・便 71）。
const RED_WHEN: &str = "赤くなる条件";

/// 契約表の章の先頭の凡例（部品 legend-line・値は契約表の size の値域 S / M の順・便 71）。
const SIZE_LEGEND: &str = "<div class=\"legend-line\"><span>大きさ: S = 小さい便（src の余地 100 行の見積）／M = 中くらいの便（300 行の見積）</span></div>";

/// 断らない口の印（欄の決まり refuses_none_marker）。
const REFUSES_NONE: &str = "なし";

/// 契約表の行の固定の置き場（この 7 つは hint に回さない）。
const CONTRACT_FIXED: [&str; 7] = ["id", "title", "req", "section", "verify", "size", "done"];

// ── 器（scribe2）の導出 file の読み方（note.rs と同じ字面を自前に持つ）──

const EXTERNAL_PATH: &str = "contracts/schema.toml";
const EXTERNAL_HEAD: &str = "schema = 1";
const EXTERNAL_ROWS_KEY: &str = "field";
const EXTERNAL_ROW_FIELDS: [&str; 3] = ["name", "need", "shape"];

// ── 面を組むのに要る材料 ──

/// 参照 id の行き先を解くための、他の正本の id の一覧。
struct Ctx {
    /// 条の id
    articles: Vec<String>,
    /// 規範文 id → その条の id
    statements: Vec<(String, String)>,
    /// rules 行の id
    rules: Vec<String>,
    /// 要件書の id（要件・非機能要件・受入基準・制約・ゴール）
    reqs: Vec<String>,
}

/// 契約 id（`<文書 id>#<行 id>`）→ 同じ面の行き先。
type Links = Vec<(String, String)>;

/// 正本の外の材料（行き先の解決に使う）。
struct Env<'a> {
    dir: &'a Path,
    ctx: Ctx,
    links: Links,
    /// 器の導出 file の field の name（宣言の順・契約表の節が無ければ空）
    fields: Vec<String>,
}

/// 節 1 つ（章の番号つき）。
struct Sec<'a> {
    /// 章の番号（1 から）
    idx: usize,
    /// 節番号（正本の字面・契約表の行の section の突き合わせに使う）
    raw_n: String,
    /// 節番号（escape 済み）
    n: String,
    /// 節の題（escape 済み）
    title: String,
    /// 節の型（閉じた一覧の key）
    key: &'static str,
    /// 節の型の名札
    label: &'static str,
    x: X<'a>,
}

/// 状態の名札（短い）と状態の行（組み立て済みの HTML）。
struct Status {
    label: &'static str,
    line: String,
    /// 見本か（承認欄の字面が変わる）
    example: bool,
}

/// 正本から数えた数（γ）。
struct Counts {
    sections: usize,
    figures: usize,
    /// 契約表の節の行の合計
    rows: usize,
    /// 型ごとの節の数（TYPES の順）
    by_type: [usize; 6],
}

impl Counts {
    /// 表紙の「節」の字（0 の型は出さない・型の一覧の順）。
    fn section_line(&self) -> String {
        let parts = TYPES
            .iter()
            .enumerate()
            .filter(|(i, _)| self.by_type[*i] > 0)
            .map(|(i, (_, label))| format!("{label} {}", self.by_type[i]))
            .collect::<Vec<_>>();
        format!("{} 節（{}）", self.sections, parts.join("・"))
    }
}

/// item-row 1 つ（表の節の行）。
struct Row {
    /// 属性 id（`s<章>-<行 id>`）
    anchor: String,
    /// rid（escape 済み）
    rid: String,
    /// rt（escape 済み）
    rt: String,
    badges: Vec<String>,
    norm: Option<String>,
    plain: Option<(&'static str, String)>,
    chips: Vec<String>,
}

// ── 入口 ──

/// 正本 1 本 → 設計ノートの面の HTML（決定的）。天井の名札は印から読む（便 40・便 83）。
pub fn derive(dir: &Path, id: &str) -> R<String> {
    check_id_shape(id)?;
    let name = format!("design-note/{id}.yaml");
    let n_doc = cursor::load(dir, &name)?;
    let c_doc = cursor::load(dir, "constitution.yaml")?;
    let r_doc = cursor::load(dir, "rules.yaml")?;
    let s_doc = cursor::load(dir, "srs.yaml")?;
    let d = X::root(&n_doc, &name);
    let ctx = context(
        &X::root(&c_doc, "constitution.yaml"),
        &X::root(&r_doc, "rules.yaml"),
        &X::root(&s_doc, "srs.yaml"),
    )?;

    let meta = d.f("meta")?;
    let file_id = meta.f("id")?.id()?;
    if file_id != id {
        return Err(format!(
            "{name}: 欄 meta.id「{file_id}」が --id「{id}」と違う"
        ));
    }

    let secs = sections(&d)?;
    let figs = match d.g("figures")? {
        Some(x) => x.seq()?,
        None => Vec::new(),
    };
    let chapters = secs.len() + usize::from(!figs.is_empty());
    if chapters > MAX_CHAPTERS {
        return Err(format!(
            "{name}: 章が {chapters} 本ある＝章が多すぎる（上限 {MAX_CHAPTERS}）"
        ));
    }
    let fields = if secs.iter().any(|s| s.key == CONTRACT_TABLE) {
        load_external(dir)?
    } else {
        Vec::new()
    };
    let env = Env {
        dir,
        ctx,
        links: contract_links(&secs, id)?,
        fields,
    };
    let st = status(&meta)?;
    let counts = counts(&secs, figs.len())?;
    let stamp = face::ceiling_stamp(dir)?;
    // prevnext は入口の棚と同じ順（id の字の順）で隣の設計ノート・両端は入口（便 65）
    let notes = face_index::notes(dir)?;
    let links: Vec<(String, String)> = notes.iter().map(face_index::Note::link).collect();
    let at = notes
        .iter()
        .position(|q| q.id() == id)
        .ok_or_else(|| format!("design-note/: 設計ノートの列に {id} が無い"))?;
    let (prev, next) = face::neighbors(&links, at);

    let frame = Frame {
        name: "設計ノート",
        // 脚注の正本の file 名は実際の id（Frame の source は 'static なので 1 面に 1 本だけ leak する・便 71）
        source: Box::leak(name.clone().into_boxed_str()),
        favicon: FAVICON,
        // 読める面の nav にこの面は無い（どの nav にも aria-current を付けない）
        current: 3,
        first: 1,
        bands: &BANDS[..chapters],
        prev,
        next,
        parts: &PARTS,
    };

    let mut o: Vec<String> = Vec::new();
    head(&mut o, &frame, &meta, id, &st, &stamp)?;
    cover(&mut o, &frame, &meta, id, &st, &counts)?;
    toc(&frame, &mut o, &secs, figs.len());
    for s in &secs {
        section_chapter(&mut o, &frame, s, &secs, &env)?;
    }
    if !figs.is_empty() {
        figures_chapter(&mut o, &frame, secs.len() + 1, &figs, &env)?;
    }
    approval_chapter(&mut o, &frame, &meta, &st)?;
    let chip = face::glossary_chip(dir)?;
    foot(&mut o, &frame, &meta, id, &counts, &chip)?;
    Ok(format!("{}\n", o.join("\n")))
}

// ── 読みと解き ──

/// 文書 id の形（欄の決まり id_pattern = 英小文字で始まり 英小文字・数字・ハイフン）。
pub(crate) fn is_doc_id(s: &str) -> bool {
    let mut cs = s.chars();
    cs.next().is_some_and(|c| c.is_ascii_lowercase())
        && cs.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn check_id_shape(id: &str) -> R<()> {
    if is_doc_id(id) {
        Ok(())
    } else {
        Err(format!(
            "--id「{id}」は id の形でない（英小文字で始まり 英小文字・数字・ハイフン）"
        ))
    }
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn context(c: &X<'_>, r: &X<'_>, s: &X<'_>) -> R<Ctx> {
    let mut articles = Vec::new();
    let mut statements = Vec::new();
    for a in c.f("articles")?.seq()? {
        let aid = a.f("id")?.id()?.to_string();
        for st in a.f("statements")?.seq()? {
            statements.push((st.f("id")?.id()?.to_string(), aid.clone()));
        }
        articles.push(aid);
    }
    let mut rules = Vec::new();
    for section in ["thresholds", "discipline"] {
        for x in r.f(section)?.seq()? {
            rules.push(x.f("id")?.id()?.to_string());
        }
    }
    let mut reqs = Vec::new();
    for section in [
        "requirements",
        "nonfunctional",
        "acceptance",
        "constraints",
        "goals",
    ] {
        for x in s.f(section)?.seq()? {
            reqs.push(x.f("id")?.id()?.to_string());
        }
    }
    Ok(Ctx {
        articles,
        statements,
        rules,
        reqs,
    })
}

fn sections<'a>(d: &X<'a>) -> R<Vec<Sec<'a>>> {
    let mut out = Vec::new();
    for (i, x) in d.f("sections")?.seq()?.into_iter().enumerate() {
        let tx = x.f("type")?;
        let t = tx.text()?;
        let (key, label) = TYPES
            .iter()
            .find(|(k, _)| *k == t)
            .ok_or_else(|| format!("{}: 節の型「{t}」は閉じた一覧に無い", tx.at))?;
        let raw_n = x.f("n")?.text()?;
        let title = x.ef("title")?;
        out.push(Sec {
            idx: i + 1,
            n: esc(&raw_n),
            raw_n,
            title,
            key,
            label,
            x,
        });
    }
    Ok(out)
}

fn contract_links(secs: &[Sec<'_>], doc_id: &str) -> R<Links> {
    let mut out = Vec::new();
    for s in secs.iter().filter(|s| s.key == CONTRACT_TABLE) {
        for row in s.x.f("rows")?.seq()? {
            let rid = row.f("id")?.id()?;
            out.push((format!("{doc_id}#{rid}"), format!("#s{}-{rid}", s.idx)));
        }
    }
    Ok(out)
}

fn counts(secs: &[Sec<'_>], figures: usize) -> R<Counts> {
    let mut by_type = [0usize; 6];
    let mut rows = 0;
    for s in secs {
        let i = TYPES
            .iter()
            .position(|(k, _)| *k == s.key)
            .unwrap_or_default();
        by_type[i] += 1;
        if s.key == CONTRACT_TABLE {
            rows += s.x.f("rows")?.seq()?.len();
        }
    }
    Ok(Counts {
        sections: secs.len(),
        figures,
        rows,
        by_type,
    })
}

/// 状態の名札と状態の行。effective に承認欄が無い・retired に後継が無い、は導出できない。
/// 後継のリンクは実在を確かめない（後継の実在は床が数える）。
fn status(meta: &X<'_>) -> R<Status> {
    let sx = meta.f("status")?;
    let label = sx.lookup(STATUS, "状態")?;
    let key = sx.v.as_str().unwrap_or_default();
    let line = match key {
        "example" => "見本・拘束力なし".to_string(),
        "draft" => "未承認・拘束力なし → 持ち主の承認で発効".to_string(),
        "effective" => {
            let ap = meta
                .g("approval")?
                .ok_or_else(|| format!("{}: effective に承認欄が無い", meta.at))?;
            format!("発効・拘束力あり（承認 {}）", ap.ef("date")?)
        }
        _ => {
            let sb = meta
                .g("superseded_by")?
                .ok_or_else(|| format!("{}: retired に superseded_by が無い", meta.at))?;
            let sid = sb.id()?;
            format!(
                "廃止 → 後継 <a class=\"xref\" href=\"note-{sid}.html\">{}</a>",
                esc(sid)
            )
        }
    };
    Ok(Status {
        label,
        line,
        example: key == "example",
    })
}

/// 参照 id 1 つの行き先（無ければ None）。5 形のどれでもない id は Err。
fn resolve(env: &Env<'_>, id: &str) -> R<Option<String>> {
    let bad = || format!("参照 id「{id}」は id の形でない（条・rules 行・要件・判断の記録・契約）");
    if let Some((doc, row)) = id.split_once('#') {
        return if is_doc_id(doc) && is_doc_id(row) {
            Ok(env
                .links
                .iter()
                .find(|(k, _)| k == id)
                .map(|(_, href)| href.clone()))
        } else {
            Err(bad())
        };
    }
    if let Some(rest) = ["P-", "A-", "N-"].iter().find_map(|p| id.strip_prefix(p)) {
        return match rest.split_once('.') {
            // 枝番付きの規範文 id は条の anchor へ（字は枝番付きのまま）
            Some((n, sub)) if digits(n) && digits(sub) => Ok(env
                .ctx
                .statements
                .iter()
                .find(|(sid, _)| sid == id)
                .map(|(_, aid)| format!("constitution.html#{}", anchor(aid)))),
            None if digits(rest) => Ok(env
                .ctx
                .articles
                .iter()
                .any(|k| k == id)
                .then(|| format!("constitution.html#{}", anchor(id)))),
            _ => Err(bad()),
        };
    }
    if let Some(rest) = ["R-", "D-"].iter().find_map(|p| id.strip_prefix(p)) {
        return if digits(rest) {
            Ok(env
                .ctx
                .rules
                .iter()
                .any(|k| k == id)
                .then(|| format!("constitution.html#{}", anchor(id))))
        } else {
            Err(bad())
        };
    }
    if let Some(rest) = ["FR", "NFR", "AC", "CON", "GOAL"]
        .iter()
        .find_map(|p| id.strip_prefix(p))
    {
        return if digits(rest) {
            Ok(env
                .ctx
                .reqs
                .iter()
                .any(|k| k == id)
                .then(|| format!("srs.html#{}", anchor(id))))
        } else {
            Err(bad())
        };
    }
    if let Some(rest) = id.strip_prefix("ADR-") {
        return if digits(rest) {
            let file = env.dir.join("adr").join(format!("{id}.yaml"));
            Ok(file.is_file().then(|| format!("{}.html", anchor(id))))
        } else {
            Err(bad())
        };
    }
    Err(bad())
}

/// 参照 id 1 つ（在ればリンク・無ければ id の直後に「（まだ分からない）」）。
fn id_link(env: &Env<'_>, x: &X<'_>) -> R<String> {
    let id = x.text()?;
    Ok(match resolve(env, &id)? {
        Some(href) => format!("<a class=\"xref\" href=\"{href}\">{}</a>", esc(&id)),
        None => format!("{}（まだ分からない）", esc(&id)),
    })
}

/// 参照 id の一覧を「・」で繋ぐ。
fn id_links(env: &Env<'_>, x: &X<'_>) -> R<Vec<String>> {
    x.seq()?.iter().map(|q| id_link(env, q)).collect()
}

/// `<dir>` の親 dir の `contracts/schema.toml` を行走査で読み、field の name を宣言の順に返す。
/// 読めない・期待する形でない は「まだ分からない」（要件書 FR10・AC8）。
fn load_external(dir: &Path) -> R<Vec<String>> {
    let bad = |why: String| format!("{EXTERNAL_PATH}: 器の導出 file が読めない: {why}");
    let parent = dir
        .parent()
        .ok_or_else(|| bad("正本の置き場の親 dir が無い".to_string()))?;
    let path = parent.join(EXTERNAL_PATH);
    if path.is_symlink() {
        return Err(bad("symlink は認めない".to_string()));
    }
    let text = fs::read_to_string(&path).map_err(|e| bad(e.to_string()))?;
    let head = format!("[[{EXTERNAL_ROWS_KEY}]]");
    let mut rows: Vec<Vec<(String, String)>> = Vec::new();
    let mut seen_head = false;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if !seen_head {
            if t != EXTERNAL_HEAD {
                return Err(bad(format!("先頭が「{EXTERNAL_HEAD}」でない: {t}")));
            }
            seen_head = true;
            continue;
        }
        if t == head {
            rows.push(Vec::new());
            continue;
        }
        if let Some(pair) = quoted_pair(t)
            && let Some(row) = rows.last_mut()
        {
            row.push(pair);
        }
    }
    if !seen_head {
        return Err(bad(format!("「{EXTERNAL_HEAD}」の行が無い")));
    }
    if rows.is_empty() {
        return Err(bad(format!("{head} の行が無い")));
    }
    let mut names = Vec::with_capacity(rows.len());
    for row in &rows {
        let value = |key: &str| {
            row.iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .filter(|v| !v.trim().is_empty())
        };
        let (Some(name), Some(_), Some(_)) = (value("name"), value("need"), value("shape")) else {
            return Err(bad(format!(
                "{head} の行に {} が揃わない",
                EXTERNAL_ROW_FIELDS.join("・")
            )));
        };
        names.push(name);
    }
    Ok(names)
}

/// `名 = "値"` の行（引用符で囲んだ値だけ・正規表現は使わない）。
fn quoted_pair(line: &str) -> Option<(String, String)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let rest = rest.trim();
    let body = rest.strip_prefix('"')?.strip_suffix('"')?;
    if key.is_empty() || body.contains('"') {
        return None;
    }
    Some((key.to_string(), body.to_string()))
}

// ── 骨格 ──

fn head(o: &mut Vec<String>, f: &Frame, meta: &X<'_>, id: &str, st: &Status, stamp: &str) -> R<()> {
    f.head(
        o,
        &format!("folio2 — 設計ノート {id}（{}）", st.label),
        &meta.ef("generated")?,
        &format!("{id} {}", meta.ef("version")?),
        st.label,
        stamp,
    );
    Ok(())
}

fn meta_span(k: &str, v: &str) -> String {
    format!("<span class=\"m\"><span class=\"k\">{k}</span><span class=\"v\">{v}</span></span>")
}

fn cover(o: &mut Vec<String>, f: &Frame, meta: &X<'_>, id: &str, st: &Status, n: &Counts) -> R<()> {
    o.push(format!("<header {}>", f.dc(Component::DocCoverBand)));
    o.push(format!(
        "<p class=\"cover-eyebrow\"><span class=\"doc-type\">設計ノート (DESIGN NOTE)</span> <span>folio2 — {id}</span></p>"
    ));
    // h1 は短い名（title は文の長さなので副題へ・便 27 の判断の記録の面と同じ）
    o.push(format!("<h1>設計ノート {id}</h1>"));
    o.push(format!("<p class=\"sub-title\">{}</p>", meta.ef("title")?));
    // 読み手の名札は生成器の固定の 1 行（正本に欄を足さない・便 74）
    o.push(
        "<div class=\"summary-card\"><span class=\"ic\">読</span><div><p class=\"lab\">読み手</p><p class=\"txt\">作る人（実装する人）。中の組み立て方（口・検査・契約表）を書く面です。何を作るかは要件書へ、なぜそう決めたかは判断の記録へ。</p></div></div>"
            .to_string(),
    );
    if let Some(note) = meta.g("note")? {
        o.push(format!(
            "<div class=\"summary-card\"><span class=\"ic\">注</span><div><p class=\"lab\">注</p><p class=\"txt\">{}</p></div></div>",
            note.e()?
        ));
    }
    o.push("<div class=\"cover-meta\">".to_string());
    o.push(meta_span("状態", st.label));
    o.push(meta_span(
        "版",
        &format!("{} / {}", meta.ef("version")?, meta.ef("generated")?),
    ));
    o.push(meta_span("節", &n.section_line()));
    o.push(meta_span("図", &format!("{} 枚", n.figures)));
    o.push(meta_span("契約表", &format!("{} 行", n.rows)));
    o.push("</div>".to_string());
    o.push(format!(
        "<p class=\"cover-status\"><span class=\"k\">状態</span><span>{}（<a href=\"#approval\">承認欄へ</a>）</span></p>",
        st.line
    ));
    o.push("</header>".to_string());
    Ok(())
}

fn toc(f: &Frame, o: &mut Vec<String>, secs: &[Sec<'_>], figures: usize) {
    let mut heads = secs
        .iter()
        .map(|s| (format!("§{} {}", s.n, s.title), (*s.label).to_string()))
        .collect::<Vec<_>>();
    if figures > 0 {
        heads.push(("図".to_string(), format!("{figures} 枚")));
    }
    f.toc(o, &heads, "承認");
}

// ── 章 ──

fn section_chapter(
    o: &mut Vec<String>,
    f: &Frame,
    s: &Sec<'_>,
    secs: &[Sec<'_>],
    env: &Env<'_>,
) -> R<()> {
    f.band(o, s.idx, s.label, &format!("§{} {}", s.n, s.title), None);
    o.push("<div class=\"chapbody\">".to_string());
    if s.key == CONTRACT_TABLE {
        o.push(SIZE_LEGEND.to_string());
    }
    if let Some(note) = s.x.g("note")? {
        o.push(format!("<p class=\"intro\">{}</p>", note.e()?));
    }
    if s.key == "prose" {
        prose(o, &s.x.ef("body")?);
    } else {
        table_chapter(o, f, s, secs, env)?;
    }
    o.push("</div>".to_string());
    Ok(())
}

/// 散文の節。空行で段落に分け、段落の中の改行は空白 1 つに。各段落に便 27 の列挙の分割を掛ける。
fn prose(o: &mut Vec<String>, body: &str) {
    for para in paragraphs(body) {
        let marks = item_marks(&para);
        if marks.len() < 2 {
            o.push(format!("<p>{para}</p>"));
            continue;
        }
        let intro = para[..marks[0].0].trim();
        if !intro.is_empty() {
            o.push(format!("<p class=\"intro\">{intro}</p>"));
        }
        o.push("<ol class=\"items\">".to_string());
        for (k, (_, end)) in marks.iter().enumerate() {
            let stop = marks.get(k + 1).map_or(para.len(), |(start, _)| *start);
            o.push(format!("<li>{}</li>", para[*end..stop].trim()));
        }
        o.push("</ol>".to_string());
    }
}

/// 空行（空白だけの行を含む）で割り、段落の中の改行は空白 1 つに（Markdown の段落と同じ）。
fn paragraphs(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !cur.is_empty() {
                out.push(cur.join(" "));
                cur.clear();
            }
        } else {
            cur.push(line);
        }
    }
    if !cur.is_empty() {
        out.push(cur.join(" "));
    }
    out
}

/// 文の頭の列挙の印「(k) 」の位置（開きの括弧の byte・印の直後の byte）。印と数えるのは、印の直前
/// （末尾の空白を除く）が本文の先頭か句点で、かつ番号が 1 から 1 ずつ増えて続くときだけ（便 27 §1 (b)）。
fn item_marks(body: &str) -> Vec<(usize, usize)> {
    let b = body.as_bytes();
    let mut marks = Vec::new();
    let mut want = 1u32;
    let mut i = 0;
    while i < b.len() {
        // 半角の開き括弧 + ASCII の数字 1 つ以上 + 半角の閉じ括弧 + 半角空白 1 つ
        if b[i] == b'(' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && b.get(j) == Some(&b')') && b.get(j + 1) == Some(&b' ') {
                let head = body[..i].trim_end();
                let at_head = head.is_empty() || head.ends_with('。');
                if at_head && body[i + 1..j].parse::<u32>() == Ok(want) {
                    marks.push((i, j + 2));
                    want += 1;
                    i = j + 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    marks
}

fn pill(s: &str) -> String {
    format!("<span class=\"pill\">{s}</span>")
}

fn push_ref_chip(chips: &mut Vec<String>, row: &X<'_>, env: &Env<'_>) -> R<()> {
    if let Some(rs) = row.g("ref")? {
        let ids = id_links(env, &rs)?;
        if !ids.is_empty() {
            chips.push(hint("根拠", &ids.join("・")));
        }
    }
    Ok(())
}

fn push_note_chip(chips: &mut Vec<String>, row: &X<'_>) -> R<()> {
    if let Some(n) = row.g("note")? {
        chips.push(hint("注", &n.e()?));
    }
    Ok(())
}

/// 表の節（部品・口・欄・検査・契約表）。行ごとに item-row を 1 つ。
fn table_chapter(
    o: &mut Vec<String>,
    f: &Frame,
    s: &Sec<'_>,
    secs: &[Sec<'_>],
    env: &Env<'_>,
) -> R<()> {
    for row in s.x.f("rows")?.seq()? {
        let rid = row.f("id")?.id()?;
        let mut r = Row {
            anchor: format!("s{}-{rid}", s.idx),
            rid: esc(rid),
            rt: String::new(),
            badges: Vec::new(),
            norm: None,
            plain: None,
            chips: Vec::new(),
        };
        match s.key {
            "parts-table" => {
                r.rt = row.ef("name")?;
                r.norm = Some(row.ef("role")?);
                push_ref_chip(&mut r.chips, &row, env)?;
                push_note_chip(&mut r.chips, &row)?;
            }
            "ports-table" => {
                r.rt = row.ef("name")?;
                r.norm = Some(format!(
                    "<span class=\"ew\">入力</span> {} <span class=\"ew\">出力</span> {}",
                    row.ef("input")?,
                    row.ef("output")?
                ));
                let refuses = row.f("refuses")?;
                let body = if refuses.v.as_str() == Some(REFUSES_NONE) {
                    "断らない".to_string()
                } else {
                    refuses.e()?
                };
                r.plain = Some(("断る", body));
                push_ref_chip(&mut r.chips, &row, env)?;
                push_note_chip(&mut r.chips, &row)?;
            }
            "fields-table" => {
                r.rt = row.ef("name")?;
                r.badges.push(pill(row.f("need")?.lookup(NEED, "要否")?));
                r.badges.push(pill(row.f("shape")?.lookup(SHAPE, "形")?));
                if let Some(en) = row.g("enum")? {
                    let vals = en.seq()?.iter().map(X::e).collect::<R<Vec<_>>>()?;
                    r.norm = Some(format!("値域: {}", vals.join("・")));
                }
                push_note_chip(&mut r.chips, &row)?;
            }
            "teeth-table" => {
                r.rt = row.ef("name")?;
                r.norm = Some(format!(
                    "<span class=\"pk\">{RED_WHEN}</span>{}",
                    row.ef("red_when")?
                ));
                r.plain = Some(("固定の材料", format!("<code>{}</code>", row.ef("fixture")?)));
                push_ref_chip(&mut r.chips, &row, env)?;
                push_note_chip(&mut r.chips, &row)?;
            }
            _ => contract_row(&mut r, &row, secs, env)?,
        }
        push_row(o, f, &r);
    }
    Ok(())
}

/// 契約表の行。固定の置き場（id・題・要件・節・検証・大きさ・done）の外の欄は器の導出 file の field の順に hint で出す。
fn contract_row(r: &mut Row, row: &X<'_>, secs: &[Sec<'_>], env: &Env<'_>) -> R<()> {
    r.rt = row.ef("title")?;
    r.badges.push(pill(&row.ef("size")?));
    r.norm = Some(row.ef("done")?);

    // 節（同じ文書の該当の節の章へ・節が無ければ「（まだ分からない）」）
    let sec = row.f("section")?.text()?;
    let body = match secs.iter().find(|s| s.raw_n == sec) {
        Some(s) => format!("<a class=\"xref\" href=\"#s{}\">§{}</a>", s.idx, esc(&sec)),
        None => format!("§{}（まだ分からない）", esc(&sec)),
    };
    r.plain = Some(("節", body));

    let reqs = id_links(env, &row.f("req")?)?;
    if !reqs.is_empty() {
        r.chips.push(hint("要件", &reqs.join("・")));
    }
    let verify = row
        .f("verify")?
        .seq()?
        .iter()
        .map(|q| Ok(format!("<code>{}</code>", q.e()?)))
        .collect::<R<Vec<_>>>()?;
    if !verify.is_empty() {
        r.chips.push(hint("検証", &verify.join("<br>")));
    }

    // 器の導出 file に無い欄は導出できない
    for (k, _) in row.pairs()? {
        if !CONTRACT_FIXED.contains(&k) && !env.fields.iter().any(|f| f == k) {
            return Err(format!(
                "{}: 契約表の欄「{k}」は器の導出 file に無い",
                row.at
            ));
        }
    }
    // 残りの欄は field の順に（行に在る欄だけ・空の一覧は出さない）
    for name in &env.fields {
        if CONTRACT_FIXED.contains(&name.as_str()) {
            continue;
        }
        let Some(x) = row.g(name)? else { continue };
        let body = match x.v.as_seq() {
            Some([]) => continue,
            Some(_) => x
                .seq()?
                .iter()
                .map(|i| Ok(format!("<code>{}</code>", i.e()?)))
                .collect::<R<Vec<_>>>()?
                .join("・"),
            None => x.e()?,
        };
        r.chips.push(hint(&esc(name), &body));
    }
    Ok(())
}

fn push_row(o: &mut Vec<String>, f: &Frame, r: &Row) {
    o.push(format!(
        "<article {} id=\"{}\">",
        f.dc(Component::ItemRow),
        r.anchor
    ));
    let badges = if r.badges.is_empty() {
        String::new()
    } else {
        format!("<span class=\"badges\">{}</span>", r.badges.concat())
    };
    o.push(format!(
        "<div class=\"ir-head\"><span class=\"rid\">{}</span><h3 class=\"rt\">{}</h3>{badges}</div>",
        r.rid, r.rt
    ));
    if let Some(norm) = &r.norm {
        o.push(format!("<p class=\"norm\">{norm}</p>"));
    }
    if let Some((pk, body)) = &r.plain {
        o.push(format!(
            "<div class=\"plain\"><span class=\"pk\">{pk}</span>{body}</div>"
        ));
    }
    if !r.chips.is_empty() {
        o.push(format!("<p class=\"meta-chips\">{}</p>", r.chips.concat()));
    }
    o.push("</article>".to_string());
}

/// 図の章。図ごとに共有の図の枠（`face::figure_panel`・便 34・凡例は無し）を置き、図の本体は `face::figure_body` が
/// 図の道具で描いたものをそのまま埋める（escape しない・道具の出力は変えない）。
/// 図が 1 枚でも導出できなければ Err（面全体が「まだ分からない」・前の面は残る）。
fn figures_chapter(
    o: &mut Vec<String>,
    f: &Frame,
    idx: usize,
    figs: &[X<'_>],
    env: &Env<'_>,
) -> R<()> {
    f.band(o, idx, "図", &format!("図 {} 枚", figs.len()), None);
    o.push("<div class=\"chapbody\">".to_string());
    for (i, fig) in figs.iter().enumerate() {
        let drawn = face::figure_body(env.dir, fig)?;
        let caption = fig.ef("caption")?;
        let refs = match fig.g("refs")? {
            Some(rs) => id_links(env, &rs)?,
            None => Vec::new(),
        };
        face::figure_panel(o, f, i + 1, &drawn, &caption, &refs);
    }
    o.push("</div>".to_string());
    Ok(())
}

fn approval_chapter(o: &mut Vec<String>, f: &Frame, meta: &X<'_>, st: &Status) -> R<()> {
    f.approval_band(o, "承認", st.label);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", f.dc(Component::ApprovalBlock)));
    match meta.g("approval")? {
        Some(ap) => o.push(format!(
            "<div class=\"sign\"><span class=\"role\">承認</span><span class=\"who\">{}</span><span class=\"when\">{}</span><span class=\"when\">逐語「{}」</span><span class=\"stamp\">{}</span></div>",
            ap.ef("who")?,
            ap.ef("date")?,
            ap.ef("verbatim")?,
            ap.ef("ruling")?
        )),
        None if st.example => {
            o.push("<p>見本（拘束力なし）は承認欄を持たない。</p>".to_string());
        }
        None => o.push("<p>未（持ち主の逐語と日付が入ると発効）</p>".to_string()),
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

/// 脚（`chip` = 用語集への札・doc-locator の行の末尾・便 74）。
fn foot(o: &mut Vec<String>, f: &Frame, meta: &X<'_>, id: &str, n: &Counts, chip: &str) -> R<()> {
    let generated = meta.ef("generated")?;
    let version = meta.ef("version")?;
    let dl = format!(
        "<dt>id</dt><dd>{id}</dd><dt>status</dt><dd>{}</dd><dt>version</dt><dd>{version}</dd><dt>profile</dt><dd>{}</dd><dt>sections</dt><dd>{}</dd><dt>figures</dt><dd>{}</dd>",
        meta.ef("status")?,
        meta.ef("profile")?,
        n.sections,
        n.figures
    );
    f.foot_aside(o, &format!("{id} {version}"), &generated, &dl, chip);
    Ok(())
}
