//! 入口の面の生成器（便 16・docs/design/delivery-16.md §1 (b)）。見本 `preview/index.html` の骨格（部品・要素の入れ子・
//! class の並び）に、入口の正本 `index.yaml` の案内文を escape して差し込み、数と日付（条の数・要件の数・語彙と rules の
//! 数・判断の記録の本数・各文書の版と生成日・文書が読めるか）は生成のたびに他の正本から数えて出す（ADR-5 決定 (2)）。
//! 生成物の文字列は 正本の値（α）・名札の表（β・`face.rs`）・正本から数えた数（γ）のどれかで、見本の layer-line・
//! 「次の一手」「まだ測っていない」の行は正本に無いので出さない。面に依らない口（head と site-bar・部品の名札・小窓）は
//! `face.rs` を呼ぶ。入口は番号付きの章を持たないので、帯は slim だけ・目次と prevnext は出さない。
//! 節「支度表」（便 20・delivery-20.md §1 (b)）は相談窓口の正本 `intake.yaml` の sheet の節と、在れば支度表
//! `<dir>/<sheet.file>`（`folio intake` の生成物）から出す。支度表が無ければ「まだ無い」の 1 行（導出できないではない）。
//! 支度表の documents の各行の id は `intake.yaml` の targets に解き、with（付録の id）は入口の正本 `index.yaml` の
//! shelf.annexes に解く（便 22・delivery-22.md §1 (a)）——付録は targets（行き先＝棚の文書 4 つと注入）には無く、
//! 棚の付録の行にだけ在るので、便 18 の床（targets の with を annexes に解く）と同じ規則で読む。
//! 棚の「判断の記録」の行は、記録が 1 本以上なら「読める」側（本数・状態ごとの数・各記録の面へのリンク）を出す
//! （便 26・delivery-26.md §1 (a)）——面は 1 本につき 1 枚（`adr-<数>.html`・便 25 の生成器）なので、記録が在れば
//! ページも在る。棚の「設計ノート」の行も同じ形で、設計ノートが 1 本以上なら「読める」側（本数・状態ごとの数・
//! 各設計ノートの面へのリンク）を出す（便 29・delivery-29.md §1 (a)）——面は 1 本につき 1 枚
//! （`note-<文書 id>.html`・便 28 の生成器）。入口の正本 `index.yaml` は改訂しない（ADR-7 帰結・棚の行の読み方だけが変わる）。

use std::fs;
use std::path::Path;

use crate::constitution_enums as ce;
use crate::face::{
    self, ANNEXES, Frame, INDEX_STATUS, R, SHELF_DOCS, SHELF_LEGEND, SHELF_RELATIONS, Shelf, X,
    anchor, esc, hint, hint_q, stop_anchor, tier_of,
};
use crate::face_note;
use crate::parts::catalog::Component;
use crate::yaml::Value;

/// 入口の面が使う部品（13 種・便 40 で ceiling-stamp を足した）。
pub const PARTS: [Component; 13] = [
    Component::FreshnessStamp,
    Component::CeilingStamp,
    Component::FontSizeControl,
    Component::HubCover,
    Component::FigurePanel,
    Component::DocShelf,
    Component::ShelfCard,
    Component::ShelfLink,
    Component::StatusLine,
    Component::IntakeLine,
    Component::ChapterDeckBand,
    Component::ReaderLane,
    Component::IntakeCallout,
];

/// 入口の面の骨格（head と部品の名札だけに使う）。
const FRAME: Frame = Frame {
    name: "入口",
    source: "index.yaml",
    favicon: "<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%231d3a54'/%3E%3Ctext x='16' y='22' font-size='16' font-weight='700' text-anchor='middle' fill='%23ffffff' font-family='sans-serif'%3E入%3C/text%3E%3C/svg%3E\">",
    current: 0,
    first: 1,
    bands: &[],
    prev: ("index.html", "入口"),
    next: ("constitution.html", "憲法"),
    parts: &PARTS,
};

/// 読んだ正本（foot の sources）。支度表は生成物なので載せない（節の中で file 名を出す）。
const SOURCES: [&str; 8] = [
    "index.yaml",
    "constitution.yaml",
    "srs.yaml",
    "vocabulary.yaml",
    "rules.yaml",
    "adr/",
    "design-note/",
    "intake.yaml",
];

/// 設計ノートの正本の置き場（この dir の直下の `.yaml` から欄の決まりを除いたもの）。
const NOTE_DIR: &str = "design-note";

/// 設計ノートではない（欄の決まりの）file 名。
const NOTE_SCHEMA: &str = "schema.yaml";

/// 相談窓口の正本（節「支度表」の見出しと説明・支度表の file 名・行き先の型の写像）。
const INTAKE: &str = "intake.yaml";

/// 判断の記録の状態（status）→ 棚の card に出す名札（β・便 26）。3 値は便 25 の面の表と同じで、
/// 名札は棚の要約に並べる短い字（要約はこの表の順に「0 でない種類」だけを並べる）。
const ADR_STATUS: &[(&str, &str)] = &[
    ("accepted", "発効"),
    ("proposed", "提案中"),
    ("retired", "廃止"),
];

/// 数が 0 のときの字（β・shelf-head の「これから増える文書」と status-line の「まだ無い」）。
const NONE: &str = "なし";

/// 節「支度表」の名札の表（β・本便）。数と日付は正本から数えたものだけ（ここには書かない）。
const SHEET_KICKER: &str = "支度表";
const SHEET_DOCUMENTS: &str = "持つ文書";
const SHEET_RECOMMENDED: &str = "推奨で進めた項目";
const SHEET_APPROVAL: &str = "承認";
const SHEET_SOURCE: &str = "正本";
const SHEET_ABSENT: &str = "まだ無い";
const SHEET_NO_DOCUMENTS: &str = "なし";
const SHEET_NO_RECOMMENDED: &str = "なし（全部に答えた）";
const SHEET_NO_APPROVAL: &str = "まだ（対話面で承認したら台帳と承認欄に記帳する）";
const SHEET_MADE_BY: &str = "（folio intake の生成物・手で直さない）";

fn dc(c: Component) -> String {
    FRAME.dc(c)
}

/// 読める面の数（入口 1 + 面が在る文書 + 判断の記録の本数 + 設計ノートの本数・どちらも 1 本につき面 1 枚）。
fn readable_faces(adr: usize, notes: usize) -> usize {
    1 + SHELF_DOCS.iter().filter(|(_, s)| s.face.is_some()).count() + adr + notes
}

/// 判断の記録 1 本の読んだ中身（`records` が id の数の昇順に並べる）。
pub struct Record {
    /// id の数（`ADR-<数>` の数・並べ替えの鍵）
    num: u64,
    /// 正本の id（英数字と「-」「.」だけなので escape は要らない）
    id: String,
    /// 状態の名札（β・`ADR_STATUS`）
    status: &'static str,
    /// 日付（escape 済み）
    date: String,
}

impl Record {
    /// この記録の面の file（`adr-<数>.html`・便 25 の生成器の出す 1 枚）。
    pub fn file(&self) -> String {
        format!("{}.html", anchor(&self.id))
    }

    /// 正本の id（`folio face --face adr --id` に渡す字と同じ）。
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// 設計ノート 1 本の読んだ中身（`notes` が id の字の昇順に並べる）。
pub struct Note {
    /// 正本の id（英小文字・数字・ハイフンだけなので escape は要らない）
    id: String,
    /// 状態の名札（β・`face_note::STATUS`）
    status: &'static str,
    /// 生成日（escape 済み）
    generated: String,
}

impl Note {
    /// この設計ノートの面の file（`note-<文書 id>.html`・便 28 の生成器の出す 1 枚）。
    pub fn file(&self) -> String {
        format!("note-{}.html", self.id)
    }

    /// 正本の id（`folio face --face note --id` に渡す字と同じ）。
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// 読める文書の数えた中身（カードの数・更新）。
struct Readable {
    summary: String,
    updated: String,
}

/// 棚の文書 1 つ（棚の置き場の順）。
struct Doc {
    id: &'static str,
    shelf: Shelf,
    ty: String,
    use_: String,
    absent: Option<String>,
    readable: Option<Readable>,
}

struct Annex {
    ty: String,
    chapter: u8,
    unit: &'static str,
    count: usize,
}

struct Rel {
    class: &'static str,
    label: String,
    hint: String,
    from_ty: String,
}

/// 導出の文脈。
struct Ctx {
    docs: Vec<Doc>,
    annexes: Vec<Annex>,
    /// 棚の付録の id → 型（escape 済み・`index.yaml` の shelf.annexes の順）。
    annex_types: Vec<(&'static str, String)>,
    relations: Vec<Rel>,
    /// 判断の記録（id の数の昇順・空なら棚の行は「まだ無い」側）
    adr: Vec<Record>,
    /// 設計ノート（id の字の昇順・空なら棚の行は「まだ無い」側）
    notes: Vec<Note>,
}

impl Ctx {
    fn doc(&self, x: &X<'_>) -> R<&Doc> {
        let s = x.text()?;
        self.docs
            .iter()
            .find(|d| d.id == s)
            .ok_or_else(|| format!("{}: 文書の id「{s}」が棚に無い", x.at))
    }

    fn readable(&self) -> impl Iterator<Item = &Doc> {
        self.docs.iter().filter(|d| d.shelf.face.is_some())
    }

    /// まだ面が無い文書（記録・設計ノートが 1 本以上なら、その行は「これから増える」側から外す）。
    fn missing(&self) -> impl Iterator<Item = &Doc> {
        self.docs
            .iter()
            .filter(move |d| d.shelf.face.is_none() && self.pages(d.id) == 0)
    }

    /// 棚の行が持つ面の数（面が 1 枚の文書は数えない＝ここは記録と設計ノートの行だけ）。
    fn pages(&self, id: &str) -> usize {
        match id {
            "adr" => self.adr.len(),
            NOTE_DIR => self.notes.len(),
            _ => 0,
        }
    }

    /// いま読める文書の型の列挙（棚の順）。記録と設計ノートは 1 本以上のときだけ並べ、
    /// `counts` が真ならその型の直後に「 <数> 本」を置く。
    fn readable_types(&self, counts: bool) -> String {
        self.docs
            .iter()
            .filter_map(|d| {
                let n = self.pages(d.id);
                if d.shelf.face.is_some() {
                    return Some(d.ty.clone());
                }
                match (n > 0, counts) {
                    (false, _) => None,
                    (true, false) => Some(d.ty.clone()),
                    (true, true) => Some(format!("{} {n} 本", d.ty)),
                }
            })
            .collect::<Vec<_>>()
            .join("・")
    }
}

/// 正本 → 入口の面の HTML（決定的）。`ceiling` = 天井の束の置き場（解決済み・None = `--ceiling` なし・便 40）。
pub fn derive(dir: &Path, ceiling: Option<&Path>) -> R<String> {
    let i_doc = face::load(dir, "index.yaml")?;
    let c_doc = face::load(dir, "constitution.yaml")?;
    let s_doc = face::load(dir, "srs.yaml")?;
    let v_doc = face::load(dir, "vocabulary.yaml")?;
    let r_doc = face::load(dir, "rules.yaml")?;
    let n_doc = face::load(dir, INTAKE)?;
    let adr = records(dir)?;
    let notes = notes(dir)?;
    let i = X::root(&i_doc, "index.yaml");
    let c = X::root(&c_doc, "constitution.yaml");
    let s = X::root(&s_doc, "srs.yaml");
    let v = X::root(&v_doc, "vocabulary.yaml");
    let r = X::root(&r_doc, "rules.yaml");
    let n = X::root(&n_doc, INTAKE);

    let ctx = context(&i, &c, &s, &v, &r, adr, notes)?;
    let sheet = sheet_head(&n)?;
    let filled = sheet_body(dir, &n, &sheet, &ctx.annex_types)?;
    let m = i.f("meta")?;
    let stamp = face::ceiling_stamp(dir, ceiling)?;

    let mut o: Vec<String> = Vec::new();
    head(&mut o, &m, &stamp)?;
    cover(&mut o, &i, &c)?;
    shelf(&mut o, &ctx, &i, &m)?;
    status_line(&mut o, &ctx);
    intake_line(&mut o, &i)?;
    lanes(&mut o, &ctx, &i)?;
    intake(&mut o, &i)?;
    sheet_section(&mut o, &i, &sheet, filled.as_ref())?;
    foot(&mut o, &m)?;
    Ok(format!("{}\n", o.join("\n")))
}

// ── 読みと数え ──

/// 判断の記録の id（`ADR-<数>`）の数（「-」で割った 2 番目・ASCII の数字列だけ・便 11 の render.rs と同じ）。
fn adr_number(id: &str) -> Option<u64> {
    let second = id.split('-').nth(1)?;
    if second.is_empty() || !second.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    second.parse().ok()
}

/// `adr/` の下で名が ADR- で始まり .yaml で終わる正本を読み、id の数の昇順に並べる（便 11 の render.rs と同じ
/// 並べ方・数字列でない id や同じ数が 2 本は導出できない）。欄 id・title・status・date は必須。
pub fn records(dir: &Path) -> R<Vec<Record>> {
    let entries = fs::read_dir(dir.join("adr")).map_err(|e| format!("adr/: 読めない: {e}"))?;
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("adr/: 読めない: {e}"))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("ADR-") && name.ends_with(".yaml") {
            names.push(format!("adr/{name}"));
        }
    }
    // 読む順を file 系の返す順に依らせない（並べ替えの前に名で揃える）
    names.sort();

    let mut docs = Vec::with_capacity(names.len());
    for name in names {
        let doc = face::load(dir, &name)?;
        docs.push((name, doc));
    }
    let mut out = Vec::with_capacity(docs.len());
    for (name, doc) in &docs {
        let a = X::root(doc, name);
        let id = a.f("id")?.id()?.to_string();
        let num = adr_number(&id)
            .ok_or_else(|| format!("{name}: id「{id}」の番号が ASCII の数字列でない"))?;
        a.ef("title")?;
        out.push(Record {
            num,
            status: a.f("status")?.lookup(ADR_STATUS, "判断の記録の状態")?,
            date: a.ef("date")?,
            id,
        });
    }
    out.sort_by_key(|r| r.num);
    if let Some(w) = out.windows(2).find(|w| w[0].num == w[1].num) {
        return Err(format!(
            "判断の記録の番号 {} が 2 本以上（{}・{}）",
            w[0].num, w[0].id, w[1].id
        ));
    }
    Ok(out)
}

/// `design-note/` の直下で名が `.yaml` で終わる正本（欄の決まり `schema.yaml` は除く）を読み、id の字の昇順に
/// 並べる（file 名が一意なので同じ id は無い）。欄 meta の id・title・status・generated は必須で、id は file 名の
/// stem と一致し、id の形（`face_note::is_doc_id`）で、status は設計ノートの面の表（`face_note::STATUS`）の中。
pub fn notes(dir: &Path) -> R<Vec<Note>> {
    let entries =
        fs::read_dir(dir.join(NOTE_DIR)).map_err(|e| format!("{NOTE_DIR}/: 読めない: {e}"))?;
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{NOTE_DIR}/: 読めない: {e}"))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".yaml") && name != NOTE_SCHEMA {
            names.push(name.into_owned());
        }
    }
    // 読む順を file 系の返す順に依らせない（並べ替えの前に名で揃える）
    names.sort();

    let mut docs = Vec::with_capacity(names.len());
    for name in names {
        let at = format!("{NOTE_DIR}/{name}");
        let doc = face::load(dir, &at)?;
        docs.push((name, at, doc));
    }
    let mut out = Vec::with_capacity(docs.len());
    for (name, at, doc) in &docs {
        let m = X::root(doc, at).f("meta")?;
        let id = m.f("id")?.text()?;
        let stem = name.trim_end_matches(".yaml");
        if id != stem {
            return Err(format!(
                "{at}: 欄 meta.id「{id}」が file 名「{stem}」と違う"
            ));
        }
        if !face_note::is_doc_id(&id) {
            return Err(format!(
                "{at}: 欄 meta.id「{id}」は id の形でない（英小文字で始まり 英小文字・数字・ハイフン）"
            ));
        }
        required(&m, "title")?;
        out.push(Note {
            status: m
                .f("status")?
                .lookup(face_note::STATUS, "設計ノートの状態")?,
            generated: esc(&required(&m, "generated")?),
            id,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// 一覧の各行の id が表の id と過不足なく一致するか。戻り値は表の順の（id・値・行）。
fn exact<'a, T: Copy>(
    list: &X<'a>,
    table: &[(&'static str, T)],
    what: &str,
) -> R<Vec<(&'static str, T, X<'a>)>> {
    let mut found: Vec<(&'static str, T, X<'a>)> = Vec::new();
    for row in list.seq()? {
        let id_x = row.f("id")?;
        let id = id_x.text()?;
        let Some((key, t)) = table.iter().find(|(k, _)| *k == id) else {
            return Err(format!("{}: {what}の表に無い id「{id}」", id_x.at));
        };
        if found.iter().any(|(k, _, _)| k == key) {
            return Err(format!("{}: {what}の id「{id}」が 2 度ある", id_x.at));
        }
        found.push((key, *t, row));
    }
    if found.len() != table.len() {
        return Err(format!(
            "{}: {what}の id が表の {} つと過不足なく一致しない",
            list.at,
            table.len()
        ));
    }
    found.sort_by_key(|(k, _, _)| table.iter().position(|(t, _)| t == k));
    Ok(found)
}

/// 憲法のカード（条の数と段ごとの数・counts と数えた数の一致）。
fn constitution_card(c: &X<'_>) -> R<Readable> {
    let m = c.f("meta")?;
    let mut tiers = Vec::new();
    for a in c.f("articles")?.seq()? {
        let t = a.f("tier")?;
        t.parse(ce::Tier::from_name, "段")?;
        tiers.push(t.text()?);
    }
    let counts = m.f("counts")?;
    let mut sum = 0;
    let mut parts = Vec::new();
    for (key, x) in counts.pairs()? {
        let tier = tier_of(key).map_err(|e| format!("{}: {e}", counts.at))?;
        let counted = tiers.iter().filter(|t| *t == key).count();
        let written = x.count()?;
        if written != counted as u64 {
            return Err(format!("{}: {written} だが数えた条は {counted}", x.at));
        }
        sum += counted;
        parts.push(format!("{} {counted}", tier.name));
    }
    if sum != tiers.len() {
        return Err(format!(
            "{}: 段ごとの数の和 {sum} が条の数 {} と違う",
            counts.at,
            tiers.len()
        ));
    }
    Ok(Readable {
        summary: format!("{} 条（{}）", tiers.len(), parts.join(" · ")),
        updated: updated(&m)?,
    })
}

/// 要件書のカード（機能・非機能・受入基準の数・counts と数えた数の一致）。
fn srs_card(s: &X<'_>) -> R<Readable> {
    let m = s.f("meta")?;
    let counts = m.f("counts")?;
    let mut parts = Vec::new();
    for (key, section, name) in [
        ("fr", "requirements", "機能"),
        ("nfr", "nonfunctional", "非機能"),
        ("ac", "acceptance", "受入基準"),
    ] {
        let counted = s.f(section)?.seq()?.len();
        let x = counts.f(key)?;
        let written = x.count()?;
        if written != counted as u64 {
            return Err(format!("{}: {written} だが数えた行は {counted}", x.at));
        }
        parts.push(format!("{name} {counted}"));
    }
    Ok(Readable {
        summary: parts.join(" · "),
        updated: updated(&m)?,
    })
}

fn updated(m: &X<'_>) -> R<String> {
    Ok(format!("{}・{}", m.ef("generated")?, m.ef("version")?))
}

fn context(
    i: &X<'_>,
    c: &X<'_>,
    s: &X<'_>,
    v: &X<'_>,
    r: &X<'_>,
    adr: Vec<Record>,
    notes: Vec<Note>,
) -> R<Ctx> {
    let sh = i.f("shelf")?;

    let mut docs = Vec::new();
    for (id, shelf, row) in exact(&sh.f("documents")?, SHELF_DOCS, "文書")? {
        let absent_x = row.f("absent")?;
        let absent = match absent_x.v {
            Value::Null => None,
            Value::Str(_) => Some(absent_x.e()?),
            _ => return Err(format!("{}: null か文字列でない", absent_x.at)),
        };
        let readable = match id {
            "constitution" => Some(constitution_card(c)?),
            "srs" => Some(srs_card(s)?),
            _ => None,
        };
        if shelf.face.is_none() && absent.is_none() {
            return Err(format!("{}: 面が無い文書に absent の文が無い", absent_x.at));
        }
        docs.push(Doc {
            id,
            shelf,
            ty: row.ef("type")?,
            use_: row.ef("use")?,
            absent,
            readable,
        });
    }
    let doc_ids: Vec<&str> = docs.iter().map(|d| d.id).collect();

    let terms = v.f("terms")?.seq()?.len();
    let rules = r.f("thresholds")?.seq()?.len() + r.f("discipline")?.seq()?.len();
    let mut annexes = Vec::new();
    let mut annex_types: Vec<(&str, String)> = Vec::new();
    for (id, (chapter, unit), row) in exact(&sh.f("annexes")?, ANNEXES, "付録")? {
        let inside = row.f("inside")?;
        if !doc_ids.contains(&inside.text()?.as_str()) {
            return Err(format!(
                "{}: 文書の id「{}」が棚に無い",
                inside.at,
                inside.text()?
            ));
        }
        let ty = row.ef("type")?;
        annex_types.push((id, ty.clone()));
        annexes.push(Annex {
            ty,
            chapter,
            unit,
            count: if id == "vocabulary" { terms } else { rules },
        });
    }

    let mut ctx = Ctx {
        docs,
        annexes,
        annex_types,
        relations: Vec::new(),
        adr,
        notes,
    };
    let type_of = |x: &X<'_>, ctx: &Ctx| -> R<String> {
        let id = x.text()?;
        if let Some((_, ty)) = ctx.annex_types.iter().find(|(k, _)| *k == id) {
            return Ok(ty.clone());
        }
        Ok(ctx.doc(x)?.ty.clone())
    };
    let mut relations = Vec::new();
    for (_, class, row) in exact(&sh.f("relations")?, SHELF_RELATIONS, "関係")? {
        type_of(&row.f("to")?, &ctx)?;
        relations.push(Rel {
            class,
            label: row.ef("label")?,
            hint: row.ef("hint")?,
            from_ty: type_of(&row.f("from")?, &ctx)?,
        });
    }
    ctx.relations = relations;
    Ok(ctx)
}

// ── 相談窓口の正本と支度表 ──

/// 相談窓口の正本の sheet の節（節の見出しと説明・支度表の file 名）。字面は escape する前。
struct SheetHead {
    file: String,
    title: String,
    explain: String,
}

/// 支度表 `<dir>/<sheet.file>` の読んだ中身（在るときだけ）。どの字面も escape 済み。
struct SheetBody {
    documents: Vec<String>,
    recommended: Vec<String>,
    approval: Option<String>,
}

/// 必須の欄の字面（無い・空は Err）。
fn required(x: &X<'_>, key: &str) -> R<String> {
    let f = x.f(key)?;
    let text = f.text()?;
    if text.is_empty() {
        return Err(format!("{}: 空", f.at));
    }
    Ok(text)
}

fn sheet_head(n: &X<'_>) -> R<SheetHead> {
    let sh = n.f("sheet")?;
    Ok(SheetHead {
        file: required(&sh, "file")?,
        title: required(&sh, "title")?,
        explain: required(&sh, "explain")?,
    })
}

/// 写像の行き先（intake.yaml の targets）の id → 型（型は escape 済み）。
fn targets(n: &X<'_>) -> R<Vec<(String, String)>> {
    n.f("targets")?
        .seq()?
        .iter()
        .map(|row| Ok((row.f("id")?.text()?, row.ef("type")?)))
        .collect()
}

/// 行き先の id を targets の型に写す（表に無い id は Err）。
fn target_type(targets: &[(String, String)], id: &str, at: &str) -> R<String> {
    targets
        .iter()
        .find(|(k, _)| k == id)
        .map(|(_, ty)| ty.clone())
        .ok_or_else(|| format!("{at}: 行き先の id「{id}」が {INTAKE} の targets に無い"))
}

/// 付録の id を棚の付録（`index.yaml` の shelf.annexes）の型に写す（棚に無い id は Err）。
fn annex_type(annexes: &[(&'static str, String)], id: &str, at: &str) -> R<String> {
    annexes
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, ty)| ty.clone())
        .ok_or_else(|| {
            format!(
                "{at}: 付録の id「{id}」が {} の annexes に無い",
                FRAME.source
            )
        })
}

/// 支度表が在れば読む（無ければ None＝「まだ無い」・在るのに読めないは Err）。
fn sheet_body(
    dir: &Path,
    n: &X<'_>,
    head: &SheetHead,
    annexes: &[(&'static str, String)],
) -> R<Option<SheetBody>> {
    if !dir.join(&head.file).exists() {
        return Ok(None);
    }
    let doc = face::load(dir, &head.file)?;
    let sheet = X::root(&doc, &head.file);
    let targets = targets(n)?;

    let mut documents = Vec::new();
    for row in sheet.f("documents")?.seq()? {
        let id_x = row.f("id")?;
        target_type(&targets, &id_x.text()?, &id_x.at)?;
        let ty = row.ef("type")?;
        let mut with = Vec::new();
        if let Some(list) = row.g("with")? {
            for w in list.seq()? {
                with.push(annex_type(annexes, &w.text()?, &w.at)?);
            }
        }
        documents.push(if with.is_empty() {
            ty
        } else {
            format!("{ty}（付録の{}）", with.join("・"))
        });
    }

    let mut recommended = Vec::new();
    for row in sheet.f("recommended")?.seq()? {
        recommended.push(format!(
            "{}（おすすめ: {}）",
            row.ef("ask")?,
            row.ef("recommend")?
        ));
    }

    let approval = match sheet.f("approval")?.seq()?.first() {
        Some(row) => Some(format!("{} {}", row.ef("when")?, row.ef("verbatim")?)),
        None => None,
    };

    Ok(Some(SheetBody {
        documents,
        recommended,
        approval,
    }))
}

// ── 骨格 ──

fn head(o: &mut Vec<String>, m: &X<'_>, stamp: &str) -> R<()> {
    let version = m.ef("version")?;
    let status = m.f("status")?.lookup(INDEX_STATUS, "入口の状態")?;
    m.f("id")?.text()?;
    FRAME.head(
        o,
        &format!("folio2 — 設計文書の入口（{version}）"),
        &m.ef("generated")?,
        &version,
        status,
        stamp,
    );
    // 入口は番号付きの章を持たないので、site-bar の here は「文書の一覧」
    if let Some(here) = o
        .iter_mut()
        .find(|l| l.starts_with("<span class=\"here\">"))
    {
        *here = format!(
            "<span class=\"here\"><b>{}</b> ▸ 文書の一覧</span>",
            FRAME.name
        );
    }
    Ok(())
}

/// hub-cover。meta の title と憲法の north_star の statement はここでだけ読む（検査もここ 1 か所）。
fn cover(o: &mut Vec<String>, i: &X<'_>, c: &X<'_>) -> R<()> {
    let title = i.f("meta")?.ef("title")?;
    let statement = c.f("north_star")?.ef("statement")?;
    let au = i.f("audience")?;
    let text = au.ef("text")?;
    o.push(format!("<header {}>", dc(Component::HubCover)));
    o.push("<p class=\"cover-eyebrow\"><span class=\"doc-type\">入口 (index)</span> <span>folio2 — 設計文書（design-intent）</span></p>".to_string());
    o.push(format!("<h1>folio2 — {title}</h1>"));
    o.push(format!(
        "<p class=\"north-star\">目指すこと: <q>{statement}</q> — <a href=\"constitution.html#s0\">憲法 §0 で読む</a></p>"
    ));
    o.push(format!(
        "<div class=\"summary-card\"><span class=\"ic\">誰</span><div><p class=\"lab\">{}</p><p class=\"txt\">{text}</p></div></div>",
        au.ef("label")?
    ));
    o.push(format!(
        "<p class=\"hub-who\">誰のため: {} {}</p>",
        au.ef("short")?,
        hint_q(&text)
    ));
    o.push("</header>".to_string());
    Ok(())
}

fn types<'d>(docs: impl Iterator<Item = &'d Doc>) -> String {
    docs.map(|d| d.ty.as_str()).collect::<Vec<_>>().join("・")
}

fn shelf(o: &mut Vec<String>, ctx: &Ctx, i: &X<'_>, m: &X<'_>) -> R<()> {
    let sh = i.f("shelf")?;
    let mut legend = String::from("<div class=\"fig-legend\">");
    for row in sh.f("legend")?.seq()? {
        let sw = row.f("id")?.lookup(SHELF_LEGEND, "凡例")?;
        legend.push_str(&format!(
            "<span class=\"lg\"><span class=\"{sw}\"></span>{}</span>",
            row.ef("text")?
        ));
    }
    legend.push_str("</div>");
    let explain = format!("<p>{}</p>", sh.ef("explain")?);

    o.push(format!(
        "<figure {} data-role=\"diagram\" id=\"docset\">",
        dc(Component::FigurePanel)
    ));
    o.push(format!(
        "<div class=\"fig-title\"><span class=\"fn\">棚</span>{} ── いま読めるのは <b>{} 面</b> ＝ このページ（入口）と、{} <span class=\"fig-tools\">{}{}<button class=\"zoom-btn\" type=\"button\">拡大</button></span><button class=\"zoom-close\" type=\"button\">✕ 閉じる</button></div>",
        sh.ef("title")?,
        readable_faces(ctx.adr.len(), ctx.notes.len()),
        ctx.readable_types(false),
        hint("凡例", &legend),
        hint("図の説明", &explain)
    ));
    // これから増える文書が 1 つも無ければ数の代わりに「なし」（括弧は出さない）
    let coming = if ctx.missing().next().is_none() {
        NONE.to_string()
    } else {
        format!("{}（{}）", ctx.missing().count(), types(ctx.missing()))
    };
    o.push(format!(
        "<div class=\"shelf-head\"><span class=\"shelf-here\">▣ 入口 = いま見ているページ</span><span class=\"sub\">これから増える文書 {coming} ／ 付録 {}（憲法の中）</span></div>",
        ctx.annexes.len()
    ));
    let mut minimap =
        String::from("<p class=\"shelf-minimap\"><span class=\"state ok\">入口（いまここ）</span>");
    for d in &ctx.docs {
        let class = if d.readable.is_some() || ctx.pages(d.id) > 0 {
            "state ok"
        } else {
            "state"
        };
        minimap.push_str(&format!(
            "<span>{}</span><a class=\"{class}\" href=\"#doc-{}\">{}</a>",
            d.shelf.sep, d.id, d.ty
        ));
    }
    minimap.push_str(&format!(
        "<span>｜</span><a class=\"state\" href=\"constitution.html#s7\">付録 {}</a></p>",
        ctx.annexes.len()
    ));
    o.push(minimap);

    let shelf_n = ctx.readable().count() + 1;
    o.push(format!(
        "<div {} class=\"shelf-grid\" style=\"--shelf-n:{shelf_n}\">",
        dc(Component::DocShelf)
    ));
    // 棚の置き場の順: shelf-c・l1・shelf-s・l2・shelf-d・branch・adr
    for (n, d) in ctx.docs.iter().enumerate() {
        if d.id == "adr" {
            o.push("<div class=\"shelf-adr\">".to_string());
            let up = &ctx.relations[3];
            o.push(format!(
                "<span {} class=\"{}\"><span class=\"arrow\"></span><span class=\"lbl\">{} → {} {}</span></span>",
                dc(Component::ShelfLink),
                up.class,
                up.from_ty,
                up.label,
                hint_q(&up.hint)
            ));
            // 棚の置き場（div の shelf-adr）は grid の外なので、card 自身の class だけが「読める」側で変わる
            let class = if ctx.adr.is_empty() {
                "is-absent"
            } else {
                d.shelf.place
            };
            shelf_card(o, ctx, d, class);
            o.push("</div>".to_string());
        } else {
            let class = if d.readable.is_some() || ctx.pages(d.id) > 0 {
                d.shelf.place.to_string()
            } else {
                format!("{} is-absent", d.shelf.place)
            };
            shelf_card(o, ctx, d, &class);
            shelf_link(o, ctx, n);
        }
    }
    o.push("</div>".to_string());
    o.push(format!(
        "<figcaption><span class=\"ver\">棚 · {} {} · index.yaml</span></figcaption>",
        m.ef("version")?,
        m.ef("generated")?
    ));
    o.push("</figure>".to_string());
    Ok(())
}

/// 判断の記録の card の 2 行（記録が 1 本以上のとき）。1 行目は本数と要約・2 行目は更新と各記録の面へのリンク。
fn adr_rows(o: &mut Vec<String>, adr: &[Record]) {
    let kinds = ADR_STATUS
        .iter()
        .filter_map(|(_, label)| {
            let n = adr.iter().filter(|r| r.status == *label).count();
            (n > 0).then(|| format!("{label} {n}"))
        })
        .collect::<Vec<_>>()
        .join("・");
    let first = &adr[0];
    let last = &adr[adr.len() - 1];
    o.push(format!(
        "<p class=\"sc-row\"><span class=\"state ok\">● {} 本</span><span>{}〜{}（{kinds}）</span></p>",
        adr.len(),
        first.id,
        last.id
    ));
    let links = adr
        .iter()
        .map(|r| format!("<a class=\"xref\" href=\"{}\">{}</a>", r.file(), r.id))
        .collect::<Vec<_>>()
        .join("・");
    // 「開く →」と当たり判定は最も新しい番号の記録の面へ
    let file = last.file();
    let updated = adr
        .iter()
        .map(|r| r.date.as_str())
        .max()
        .unwrap_or_default();
    o.push(format!(
        "<p class=\"sc-row\"><span class=\"up\">更新 {updated}</span>{links}<a class=\"sc-open\" href=\"{file}\">開く →</a><a class=\"sc-hit\" href=\"{file}\" aria-hidden=\"true\" tabindex=\"-1\"></a></p>"
    ));
}

/// 設計ノートの card の 2 行（設計ノートが 1 本以上のとき）。1 行目は本数と状態ごとの数・2 行目は更新と
/// 各設計ノートの面へのリンク（id は番号でないので範囲は出さない）。
fn note_rows(o: &mut Vec<String>, notes: &[Note]) {
    let kinds = face_note::STATUS
        .iter()
        .filter_map(|(_, label)| {
            let n = notes.iter().filter(|q| q.status == *label).count();
            (n > 0).then(|| format!("{label} {n}"))
        })
        .collect::<Vec<_>>()
        .join("・");
    o.push(format!(
        "<p class=\"sc-row\"><span class=\"state ok\">● {} 本</span><span>{kinds}</span></p>",
        notes.len()
    ));
    let links = notes
        .iter()
        .map(|q| format!("<a class=\"xref\" href=\"{}\">{}</a>", q.file(), q.id))
        .collect::<Vec<_>>()
        .join("・");
    // 「開く →」と当たり判定は生成日が最も新しい 1 本へ（同じ日付が 2 本以上なら id の順で後の 1 本）
    let updated = notes
        .iter()
        .map(|q| q.generated.as_str())
        .max()
        .unwrap_or_default();
    let last = notes
        .iter()
        .rfind(|q| q.generated == updated)
        .map(Note::file)
        .unwrap_or_default();
    o.push(format!(
        "<p class=\"sc-row\"><span class=\"up\">更新 {updated}</span>{links}<a class=\"sc-open\" href=\"{last}\">開く →</a><a class=\"sc-hit\" href=\"{last}\" aria-hidden=\"true\" tabindex=\"-1\"></a></p>"
    ));
}

fn shelf_card(o: &mut Vec<String>, ctx: &Ctx, d: &Doc, class: &str) {
    o.push(format!(
        "<article {} class=\"{class}\" id=\"doc-{}\" data-doc-type=\"{}\">",
        dc(Component::ShelfCard),
        d.id,
        d.id
    ));
    o.push(format!(
        "<p class=\"sc-type\">{}<span class=\"sub\">{}</span></p>",
        d.ty, d.shelf.en
    ));
    o.push(format!("<p class=\"sc-use\">{}</p>", d.use_));
    match (&d.readable, d.shelf.face) {
        (Some(rd), Some(file)) => {
            o.push(format!(
                "<p class=\"sc-row\"><span class=\"state ok\">● 揃っている</span><span>{}</span></p>",
                rd.summary
            ));
            o.push(format!(
                "<p class=\"sc-row\"><span class=\"up\">更新 {}</span><a class=\"sc-open\" href=\"{file}\">開く →</a><a class=\"sc-hit\" href=\"{file}\" aria-hidden=\"true\" tabindex=\"-1\"></a></p>",
                rd.updated
            ));
        }
        _ if d.id == "adr" && !ctx.adr.is_empty() => adr_rows(o, &ctx.adr),
        _ if d.id == NOTE_DIR && !ctx.notes.is_empty() => note_rows(o, &ctx.notes),
        _ => {
            let state = if d.id == "adr" {
                format!("○ {} 本", ctx.adr.len())
            } else {
                "○ まだ無い".to_string()
            };
            let text = d.absent.clone().unwrap_or_default();
            o.push(format!(
                "<p class=\"sc-row\"><span class=\"state\">{state}</span><span>{text}</span></p>"
            ));
            o.push("<p class=\"sc-none\">—（まだページはありません）</p>".to_string());
        }
    }
    o.push("</article>".to_string());
}

/// n 番目の文書の後の関係（shelf-c の後 = binds・shelf-s の後 = before-build・shelf-d の後 = inside）。
fn shelf_link(o: &mut Vec<String>, ctx: &Ctx, n: usize) {
    let rel = &ctx.relations[n];
    let chips = if n == 2 {
        let chips = ctx
            .annexes
            .iter()
            .map(|a| {
                format!(
                    "<a href=\"constitution.html#s{c}\">付録 {} <span class=\"cnt\">{} {} → 憲法 §{c}</span></a>",
                    a.ty,
                    a.count,
                    a.unit,
                    c = a.chapter
                )
            })
            .collect::<String>();
        format!("<span class=\"annex-chips\">{chips}</span>")
    } else {
        String::new()
    };
    o.push(format!(
        "<span {} class=\"{}\"><span class=\"arrow\"></span><span class=\"lbl\">{} {}</span>{chips}</span>",
        dc(Component::ShelfLink),
        rel.class,
        rel.label,
        hint_q(&rel.hint)
    ));
}

fn status_line(o: &mut Vec<String>, ctx: &Ctx) {
    let annex_types = ctx
        .annexes
        .iter()
        .map(|a| a.ty.as_str())
        .collect::<Vec<_>>()
        .join("・");
    let missing = ctx
        .missing()
        .map(|d| format!("{}（{}）", d.ty, d.absent.clone().unwrap_or_default()))
        .collect::<Vec<_>>()
        .join("／");
    // まだ無い文書が 1 つも無くても行は残す（部品の数を変えない）
    let missing = if missing.is_empty() {
        NONE.to_string()
    } else {
        missing
    };
    o.push(format!(
        "<section {} aria-label=\"いまの状態\">",
        dc(Component::StatusLine)
    ));
    st(
        o,
        " ok",
        "●",
        "揃っている",
        &format!(
            "{}が読める（付録の{annex_types}は憲法の中）",
            ctx.readable_types(true)
        ),
    );
    st(o, "", "○", "まだ無い", &missing);
    o.push("</section>".to_string());
}

/// status-line の 1 行（`class` は "" か " ok"・字面はすべて escape 済み）。
fn st(o: &mut Vec<String>, class: &str, mark: &str, k: &str, v: &str) {
    o.push(format!(
        "<p class=\"st{class}\"><span class=\"mark\">{mark}</span><span class=\"k\">{k}</span><span class=\"v\">{v}</span></p>"
    ));
}

fn steps(it: &X<'_>) -> R<Vec<String>> {
    it.f("steps")?
        .seq()?
        .iter()
        .map(|x| match x.v {
            Value::Str(_) => x.e(),
            _ => Err(format!("{}: 文字列でない", x.at)),
        })
        .collect()
}

fn intake_line(o: &mut Vec<String>, i: &X<'_>) -> R<()> {
    let it = i.f("intake")?;
    o.push(format!(
        "<p {}><span class=\"k\">相談を始める:</span><span class=\"cmd\">{}</span><span>（と AI に頼む）→ {}</span><a class=\"more\" href=\"#s2\">くわしく →</a></p>",
        dc(Component::IntakeLine),
        it.ef("command")?,
        steps(&it)?.join(" → ")
    ));
    Ok(())
}

/// slim の帯（読む順番・相談窓口・支度表）。字面はすべて escape 済み。
fn band(o: &mut Vec<String>, n: usize, class: &str, kicker: &str, title: &str, lead: &str) {
    o.push(format!(
        "<section id=\"s{n}\" {} class=\"{class} slim\">",
        dc(Component::ChapterDeckBand)
    ));
    o.push(format!("<span class=\"kicker\">{kicker}</span>"));
    o.push(format!("<h2>{title}</h2>"));
    o.push(format!("<p class=\"lead\">{lead}</p>"));
    o.push("</section>".to_string());
}

/// 欄 title と lead を持つ節の slim の帯。
fn slim_band(o: &mut Vec<String>, n: usize, class: &str, kicker: &str, x: &X<'_>) -> R<()> {
    band(o, n, class, kicker, &x.ef("title")?, &x.ef("lead")?);
    Ok(())
}

fn lanes(o: &mut Vec<String>, ctx: &Ctx, i: &X<'_>) -> R<()> {
    let la = i.f("lanes")?;
    slim_band(o, 1, "band-5", "読む順番", &la)?;
    let rows = la.f("rows")?.seq()?;
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<div class=\"lane-grid\" style=\"--band-n:{}\">",
        rows.len()
    ));
    for row in &rows {
        row.f("id")?.text()?;
        let mut stops = String::new();
        for st in row.f("stops")?.seq()? {
            let doc_x = st.f("doc")?;
            let doc = ctx.doc(&doc_x)?;
            let Some(file) = doc.shelf.face else {
                return Err(format!("{}: 文書「{}」は面が無い", doc_x.at, doc.id));
            };
            let at_x = st.f("at")?;
            let at = at_x.text()?;
            stop_anchor(doc.id, &at).map_err(|e| format!("{}: {e}", at_x.at))?;
            stops.push_str(&format!(
                "<li><a href=\"{file}#{at}\">{} {}</a></li>",
                doc.ty,
                st.ef("label")?
            ));
        }
        o.push(format!("<article {}>", dc(Component::ReaderLane)));
        o.push(format!(
            "<p class=\"who\"><span class=\"av\">{}</span>{}</p>",
            row.ef("mark")?,
            row.ef("who")?
        ));
        o.push(format!("<p class=\"why\">{}</p>", row.ef("why")?));
        o.push(format!("<ol>{stops}</ol>"));
        o.push(format!(
            "<span class=\"time\">約 {} 分</span>",
            row.f("minutes")?.count()?
        ));
        o.push("</article>".to_string());
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn intake(o: &mut Vec<String>, i: &X<'_>) -> R<()> {
    let it = i.f("intake")?;
    slim_band(o, 2, "band-3", "相談窓口", &it)?;
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", dc(Component::IntakeCallout)));
    o.push("<div>".to_string());
    o.push(format!("<h3>{}</h3>", it.ef("heading")?));
    o.push(format!("<p>{}</p>", it.ef("text")?));
    let steps = steps(&it)?
        .iter()
        .enumerate()
        .map(|(n, s)| format!("<span>{} · {s}</span>", n + 1))
        .collect::<String>();
    o.push(format!("<div class=\"steps\">{steps}</div>"));
    o.push("</div>".to_string());
    o.push(format!("<div class=\"cmd\">{}</div>", it.ef("command")?));
    o.push("</div>".to_string());
    if let Some(note) = it.g("note")? {
        o.push(format!(
            "<details class=\"note\"><summary>補足</summary><div><p>{}</p></div></details>",
            note.e()?
        ));
    }
    o.push("</div>".to_string());
    Ok(())
}

/// 節「支度表」（章 02 の直後・foot の前）。相談窓口の結果が在ればその行を、無ければ「まだ無い」の 1 行を出す。
fn sheet_section(
    o: &mut Vec<String>,
    i: &X<'_>,
    head: &SheetHead,
    body: Option<&SheetBody>,
) -> R<()> {
    band(
        o,
        3,
        "band-4",
        SHEET_KICKER,
        &esc(&head.title),
        &esc(&head.explain),
    );
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<section {} aria-label=\"{SHEET_KICKER}\">",
        dc(Component::StatusLine)
    ));
    match body {
        Some(b) => {
            let documents = if b.documents.is_empty() {
                SHEET_NO_DOCUMENTS.to_string()
            } else {
                b.documents.join("・")
            };
            let recommended = if b.recommended.is_empty() {
                SHEET_NO_RECOMMENDED.to_string()
            } else {
                b.recommended.join("／")
            };
            st(o, " ok", "●", SHEET_DOCUMENTS, &documents);
            st(o, "", "○", SHEET_RECOMMENDED, &recommended);
            st(
                o,
                "",
                "○",
                SHEET_APPROVAL,
                b.approval.as_deref().unwrap_or(SHEET_NO_APPROVAL),
            );
            st(
                o,
                "",
                "·",
                SHEET_SOURCE,
                &format!("{}{SHEET_MADE_BY}", esc(&head.file)),
            );
        }
        None => st(
            o,
            "",
            "○",
            SHEET_ABSENT,
            &format!(
                "支度表はまだ無い。「{}」と AI に頼むと作られる",
                i.f("intake")?.ef("command")?
            ),
        ),
    }
    o.push("</section>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn foot(o: &mut Vec<String>, m: &X<'_>) -> R<()> {
    let version = m.ef("version")?;
    o.push("<footer class=\"foot\">".to_string());
    o.push(format!(
        "<p class=\"ft-plain\">このページは正本 {} と他の正本から folio が生成した · {} {version}（{}）· 手で直さない</p>",
        FRAME.source,
        FRAME.name,
        m.ef("generated")?
    ));
    o.push(format!(
        "<details class=\"machine\" data-audience=\"machine\"><summary>機械のための面</summary><dl><dt>id</dt><dd>{}</dd><dt>version</dt><dd>{version}</dd><dt>status</dt><dd>{}</dd><dt>sources</dt><dd>{}</dd></dl></details>",
        m.ef("id")?,
        m.ef("status")?,
        SOURCES.join(" / ")
    ));
    o.push("</footer>".to_string());
    o.push("</main>".to_string());
    o.push(format!(
        "<p class=\"doc-locator\">この文書の所属: 設計文書（design-intent）/ {} — <a href=\"constitution.html\">憲法へ</a> · <a href=\"srs.html\">要件書へ</a></p>",
        FRAME.name
    ));
    o.push("</body>".to_string());
    o.push("</html>".to_string());
    Ok(())
}

#[cfg(test)]
mod face_index_tests {
    use super::*;
    use crate::yaml;

    #[test]
    fn face_index_document_table_has_four_ids_and_rejects_others() {
        let keys: Vec<&str> = SHELF_DOCS.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, ["constitution", "srs", "design-note", "adr"]);
        let ok = yaml::parse_typed(
            "- {id: adr}\n- {id: srs}\n- {id: design-note}\n- {id: constitution}\n",
        )
        .unwrap();
        let found = exact(&X::root(&ok, "d"), SHELF_DOCS, "文書").unwrap();
        let order: Vec<&str> = found.iter().map(|(k, _, _)| *k).collect();
        assert_eq!(order, keys, "表の順に並べ直す");
        for bad in [
            "- {id: constitution}\n- {id: srs}\n- {id: design-note}\n",
            "- {id: constitution}\n- {id: srs}\n- {id: design-note}\n- {id: adr}\n- {id: memo}\n",
            "- {id: constitution}\n- {id: srs}\n- {id: design-note}\n- {id: srs}\n",
        ] {
            let v = yaml::parse_typed(bad).unwrap();
            assert!(
                exact(&X::root(&v, "d"), SHELF_DOCS, "文書").is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn face_index_stop_anchor_resolves_only_the_face_anchors() {
        for at in ["s0", "s1", "s8"] {
            assert!(stop_anchor("constitution", at).is_ok(), "{at}");
        }
        for at in ["s1", "s8", "fig-context", "fig-rail", "fig-verdicts"] {
            assert!(stop_anchor("srs", at).is_ok(), "{at}");
        }
        for (doc, at) in [
            ("constitution", "s9"),
            ("constitution", "fig-rail"),
            ("srs", "s0"),
            ("srs", "s9"),
            ("srs", "fig-other"),
            ("srs", "s10"),
            ("design-note", "s1"),
            ("constitution", ""),
        ] {
            assert!(stop_anchor(doc, at).is_err(), "{doc}#{at}");
        }
    }

    #[test]
    fn face_index_counts_readable_faces() {
        assert_eq!(readable_faces(0, 0), 3);
        assert_eq!(readable_faces(7, 1), 11);
    }

    #[test]
    fn face_parts_are_all_allowed_on_the_index_face() {
        for part in PARTS {
            assert!(
                part.faces().contains(&"index"),
                "{} は入口の面に置けない",
                part.name()
            );
        }
        let mut names: Vec<&str> = PARTS.iter().map(|p| p.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 13);
    }
}
