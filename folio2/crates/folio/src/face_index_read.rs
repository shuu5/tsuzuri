//! 入口の面の読み手（便 115・docs/design/delivery-115.md §1・ADR-15 決定 (2)(3)・責務の層 4 面に出す・面の中の割り）。`face_index.rs` から
//! 部品の一覧と枠・置き場の定数と判断の記録の状態の表・読んだ中身と文脈の型とその method・読みと数え・支度表の読みを字を変えずに降ろした。
//! 面の口 `derive` と HTML を書く側と名札の表は `face_index.rs` に残る。面の口と字面の逃がしを呼ぶので層 1 には置かない。見え方は
//! 書く側と使う側が名指すものだけを広げた。移した注の中の file 名（`render.rs` など）は移す前の置き場から見た字のまま。

use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use crate::catalog::Component;
use crate::constitution_enums as ce;
use crate::cursor::{self, R, X, esc};
use crate::face::{self, Frame, anchor, tier_of};
use crate::shelf::{self, ANNEXES, SHELF_DOCS, SHELF_RELATIONS, Shelf};
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
pub(crate) static FRAME: LazyLock<Frame> = LazyLock::new(|| Frame {
    name: "入口",
    source: "index.yaml",
    favicon: "<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%231d3a54'/%3E%3Ctext x='16' y='22' font-size='16' font-weight='700' text-anchor='middle' fill='%23ffffff' font-family='sans-serif'%3E入%3C/text%3E%3C/svg%3E\">",
    current: 0,
    first: 1,
    bands: &[],
    prev: face::link("index.html", "入口"),
    next: face::link("constitution.html", "憲法"),
    parts: &PARTS,
});

/// 設計ノートの正本の置き場（この dir の直下の `.yaml` から欄の決まりを除いたもの）。
pub(crate) const NOTE_DIR: &str = "design-note";

/// 設計ノートではない（欄の決まりの）file 名。
const NOTE_SCHEMA: &str = "schema.yaml";

/// 相談窓口の正本（節「支度表」の見出しと説明・支度表の file 名・行き先の型の写像）。
pub(crate) const INTAKE: &str = "intake.yaml";

/// 判断の記録の状態（status）→ 棚の card に出す名札（β・便 26）。3 値は便 25 の面の表と同じで、
/// 名札は棚の要約に並べる短い字（要約はこの表の順に「0 でない種類」だけを並べる）。
pub(crate) const ADR_STATUS: &[(&str, &str)] = &[
    ("accepted", "発効"),
    ("proposed", "提案中"),
    ("retired", "廃止"),
];

/// 判断の記録 1 本の読んだ中身（`records` が id の数の昇順に並べる）。
pub struct Record {
    /// id の数（`ADR-<数>` の数・並べ替えの鍵）
    num: u64,
    /// 正本の id（英数字と「-」「.」だけなので escape は要らない）
    pub(crate) id: String,
    /// 状態の名札（β・`ADR_STATUS`）
    pub(crate) status: &'static str,
    /// 日付（escape 済み）
    pub(crate) date: String,
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
    pub(crate) id: String,
    /// 状態の名札（β・`shelf::STATUS`）
    pub(crate) status: &'static str,
    /// 生成日（escape 済み）
    pub(crate) generated: String,
    /// 題（escape 済み）
    title: String,
}

impl Note {
    /// この設計ノートの面の file（`note-<文書 id>.html`・便 28 の生成器の出す 1 枚）。
    pub fn file(&self) -> String {
        format!("note-{}.html", self.id)
    }

    /// 隣の面の prevnext が指す（file・名 = 題・便 65）。
    pub fn link(&self) -> (String, String) {
        (self.file(), self.title.clone())
    }

    /// 正本の id（`folio face --face note --id` に渡す字と同じ）。
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// 読める文書の数えた中身（カードの数・更新）。
pub(crate) struct Readable {
    pub(crate) summary: String,
    pub(crate) updated: String,
}

/// 棚の文書 1 つ（棚の置き場の順）。
pub(crate) struct Doc {
    pub(crate) id: &'static str,
    pub(crate) shelf: Shelf,
    pub(crate) ty: String,
    pub(crate) use_: String,
    pub(crate) absent: Option<String>,
    pub(crate) readable: Option<Readable>,
}

pub(crate) struct Annex {
    pub(crate) ty: String,
    pub(crate) chapter: u8,
    pub(crate) unit: &'static str,
    pub(crate) count: usize,
}

pub(crate) struct Rel {
    pub(crate) class: &'static str,
    pub(crate) label: String,
    pub(crate) hint: String,
    pub(crate) from_ty: String,
}

/// 導出の文脈。
pub(crate) struct Ctx {
    pub(crate) docs: Vec<Doc>,
    pub(crate) annexes: Vec<Annex>,
    /// 棚の付録の id → 型（escape 済み・`index.yaml` の shelf.annexes の順）。
    pub(crate) annex_types: Vec<(&'static str, String)>,
    pub(crate) relations: Vec<Rel>,
    /// 判断の記録（id の数の昇順・空なら棚の行は「まだ無い」側）
    pub(crate) adr: Vec<Record>,
    /// 設計ノート（id の字の昇順・空なら棚の行は「まだ無い」側）
    pub(crate) notes: Vec<Note>,
}

impl Ctx {
    pub(crate) fn doc(&self, x: &X<'_>) -> R<&Doc> {
        let s = x.text()?;
        self.docs
            .iter()
            .find(|d| d.id == s)
            .ok_or_else(|| format!("{}: 文書の id「{s}」が棚に無い", x.at))
    }

    pub(crate) fn readable(&self) -> impl Iterator<Item = &Doc> {
        self.docs.iter().filter(|d| d.shelf.face.is_some())
    }

    /// まだ面が無い文書（記録・設計ノートが 1 本以上なら、その行は「これから増える」側から外す）。
    pub(crate) fn missing(&self) -> impl Iterator<Item = &Doc> {
        self.docs
            .iter()
            .filter(move |d| d.shelf.face.is_none() && self.pages(d.id) == 0)
    }

    /// 棚の行が持つ面の数（面が 1 枚の文書は数えない＝ここは記録と設計ノートの行だけ）。
    pub(crate) fn pages(&self, id: &str) -> usize {
        match id {
            "adr" => self.adr.len(),
            NOTE_DIR => self.notes.len(),
            _ => 0,
        }
    }

    /// いま読める文書の型の列挙（棚の順）。記録と設計ノートは 1 本以上のときだけ並べ、
    /// `counts` が真ならその型の直後に「 <数> 本」を置く。
    pub(crate) fn readable_types(&self, counts: bool) -> String {
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

// ── 読みと数え ──

/// 判断の記録の id（`ADR-<数>`）の数（「-」で割った 2 番目・ASCII の数字列だけ・便 11 の render.rs と同じ）。
pub(crate) fn adr_number(id: &str) -> Option<u64> {
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
        let doc = cursor::load(dir, &name)?;
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
/// stem と一致し、id の形（`shelf::is_doc_id`）で、status は設計ノートの面の表（`shelf::STATUS`）の中。
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
        let doc = cursor::load(dir, &at)?;
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
        if !shelf::is_doc_id(&id) {
            return Err(format!(
                "{at}: 欄 meta.id「{id}」は id の形でない（英小文字で始まり 英小文字・数字・ハイフン）"
            ));
        }
        let title = esc(&required(&m, "title")?);
        out.push(Note {
            status: m
                .f("status")?
                .lookup(shelf::STATUS, "設計ノートの状態")?,
            generated: esc(&required(&m, "generated")?),
            id,
            title,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// 一覧の各行の id が表の id と過不足なく一致するか。戻り値は表の順の（id・値・行）。
pub(crate) fn exact<'a, T: Copy>(
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

pub(crate) fn context(
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
pub(crate) struct SheetHead {
    pub(crate) file: String,
    pub(crate) title: String,
    pub(crate) explain: String,
}

/// 支度表 `<dir>/<sheet.file>` の読んだ中身（在るときだけ）。どの字面も escape 済み。
pub(crate) struct SheetBody {
    pub(crate) documents: Vec<String>,
    pub(crate) recommended: Vec<String>,
    /// 承認の最初の 1 行（日付・逐語）。無ければ None＝「まだ」。
    pub(crate) approval: Option<(String, String)>,
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

pub(crate) fn sheet_head(n: &X<'_>) -> R<SheetHead> {
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
pub(crate) fn sheet_body(
    dir: &Path,
    n: &X<'_>,
    head: &SheetHead,
    annexes: &[(&'static str, String)],
) -> R<Option<SheetBody>> {
    if !dir.join(&head.file).exists() {
        return Ok(None);
    }
    let doc = cursor::load(dir, &head.file)?;
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
        Some(row) => Some((row.ef("when")?, row.ef("verbatim")?)),
        None => None,
    };

    Ok(Some(SheetBody {
        documents,
        recommended,
        approval,
    }))
}
