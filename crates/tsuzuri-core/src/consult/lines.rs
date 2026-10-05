//! 台帳の相談の行の字と置き場（判断の記録 ADR-29 決定 (4)(8)・器の注意 (a)(b)(c)）。
//! 行は「相談の」で始まる 6 つの頭（頼み・開き・所見・受け・処分・閉じ）と「 = 」と主の id の後に、
//! 「<名> = <値>」の欄を「・」で区切って並べる。時刻は UTC の分の字で、前に user を付けない。中身（所見の欄）は書かない。
//! 自由な字（題・理由・model・念入りさ）は「・」を「／」に、制御の字を空白に替えて切る（題と model と念入りさ 40 字・理由 80 字）。
//! 行の字に器の引用の 4 形（`:` と分の字と `-` と数字の裁定 id の形・`batch:`・`policy:`・`user ` と数字）と、
//! 導出グラフの定型行の頭（`TYPED_LINES`）と配達の印の頭（`MARK_PREFIX`）が在れば書かずに断る。
//! 裸の bead id（問いの id を含む）は断らない（器は `<問い id>:<分>-<n>` まで揃った字だけを問いの形の引用と読む）。
//! 置き場は memo（label `intake:memo`）か根の epic だけで、開いた問いと契約の bead には置かない。
//! notes の上限は memo だけに在る（器の rules 行 memo.notes_max_bytes）ので、memo が満ちる行は根に置く（根の epic に上限は無い）。

use std::collections::BTreeSet;

use tsuzuri_contract::consult::{
    ConsultUnreceived, FindingId, Form, RequestId, Starter, Verdict, Via, WindowId, Word, minute_ok,
};
use tsuzuri_contract::graph::natural_cmp;
use tsuzuri_contract::ledger::{BeadId, LedgerItem};

use crate::delivery::MARK_PREFIX;
use crate::graph::build::TYPED_LINES;

/// 6 つの行の頭（頼み・開き・所見・受け・処分・閉じの順）。
pub const HEADS: [&str; 6] = [
    "相談の頼み",
    "相談の開き",
    "相談の所見",
    "相談の受け",
    "相談の処分",
    "相談の閉じ",
];

/// 欄の区切り。
const SEP: char = '・';

/// 名と値の間。
const EQ: &str = " = ";

/// 列の値の区切り（保存と不保存の path の列）。
const LIST_SEP: char = '、';

/// 題の無い窓と頼みの題の字。
pub const NO_TOPIC: &str = "題なし";

/// 空の列の字。
pub const NONE: &str = "なし";

/// 題と model と念入りさの字の上限（字の数）。
pub const WORD_MAX: usize = 40;

/// 理由の字の上限（字の数）。
pub const REASON_MAX: usize = 80;

/// memo の notes の上限（byte・器の注意 (b)）。
pub const NOTES_MAX: usize = 8192;

/// 席への固定の 1 行の頭。
pub const NOTICE_HEAD: &str = "相談: ";

/// 窓の開きの起こし手（持ち主のチャットは器が記録した発話の時刻の分の字・持ち主の button は頼みの id を添える）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum By {
    Seat,
    Chat { uttered: String },
    Button { request: RequestId },
}

impl By {
    pub fn starter(&self) -> Starter {
        match self {
            By::Seat => Starter::Seat,
            By::Chat { .. } => Starter::Chat,
            By::Button { .. } => Starter::Button,
        }
    }
}

/// 受けた物（所見か頼み）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    Finding(FindingId),
    Request(RequestId),
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::Finding(id) => id.fmt(f),
            Subject::Request(id) => id.fmt(f),
        }
    }
}

/// 台帳の相談の行の 1 つ（題の None は「題なし」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Request {
        id: RequestId,
        topic: Option<String>,
        form: Form,
        model: String,
        at: String,
    },
    Open {
        window: WindowId,
        form: Form,
        topic: Option<String>,
        by: By,
        model: String,
        effort: String,
        opened: bool,
        again: bool,
        at: String,
    },
    Finding {
        id: FindingId,
        topic: Option<String>,
        at: String,
    },
    Receipt {
        subject: Subject,
        via: Via,
        at: String,
    },
    Disposal {
        id: FindingId,
        verdict: Verdict,
        reason: String,
        keep: Vec<String>,
        drop: Vec<String>,
        at: String,
    },
    Close {
        window: WindowId,
        by: Starter,
        findings: u32,
        at: String,
    },
}

/// 行を書かない理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineError {
    /// 時刻の字が UTC の分の形でない。
    Minute,
    /// 自由な字を整えたら空になった（model・念入りさ・理由）。
    Empty,
    /// 保存か不保存の path に区切りの字か制御の字が在る。
    Path,
    /// 行の字に器の引用の形か定型行の頭か配達の印の頭が在る。
    Cited,
}

/// 自由な字を行に置ける形に整える（「・」を「／」に・制御の字を空白に・前後の空白を除き max 字で切る）。
pub fn free(text: &str, max: usize) -> String {
    let swapped: String = text
        .chars()
        .map(|c| match c {
            '・' => '／',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    swapped
        .trim()
        .chars()
        .take(max)
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// 字に器の引用の形か定型行の頭か配達の印の頭が在るか。
pub fn cited(text: &str) -> bool {
    text.contains("batch:")
        || text.contains("policy:")
        || followed(text, "user ", |rest| {
            rest.bytes().next().is_some_and(|b| b.is_ascii_digit())
        })
        || followed(text, ":", ruling_tail)
        || TYPED_LINES.iter().any(|(head, _)| text.contains(head))
        || text.contains(MARK_PREFIX)
}

/// `mark` の後の字のどれかが `tail` に当たるか。
fn followed(text: &str, mark: &str, tail: impl Fn(&str) -> bool) -> bool {
    text.match_indices(mark)
        .any(|(at, m)| text.get(at + m.len()..).is_some_and(&tail))
}

/// 裁定の id の尾（UTC の分の字と `-` と数字）。
fn ruling_tail(rest: &str) -> bool {
    rest.get(..14).is_some_and(minute_ok)
        && rest.get(14..15) == Some("-")
        && rest
            .get(15..16)
            .is_some_and(|d| d.bytes().all(|b| b.is_ascii_digit()))
}

fn topic_word(topic: &Option<String>) -> String {
    match topic.as_deref().map(|t| free(t, WORD_MAX)) {
        Some(t) if !t.is_empty() => t,
        _ => NO_TOPIC.to_string(),
    }
}

fn word(text: &str, max: usize) -> Result<String, LineError> {
    let t = free(text, max);
    if t.is_empty() {
        Err(LineError::Empty)
    } else {
        Ok(t)
    }
}

fn minute(at: &str) -> Result<&str, LineError> {
    if minute_ok(at) {
        Ok(at)
    } else {
        Err(LineError::Minute)
    }
}

fn paths(list: &[String]) -> Result<String, LineError> {
    if list.iter().any(|p| {
        p.is_empty()
            || p.chars()
                .any(|c| c.is_control() || c == SEP || c == LIST_SEP)
    }) {
        return Err(LineError::Path);
    }
    Ok(if list.is_empty() {
        NONE.to_string()
    } else {
        list.join("、")
    })
}

/// 欄の列（名と値）。
type Fields = Vec<(&'static str, String)>;

/// 行の頭の番号と主の id と欄の列。
fn parts(line: &Line) -> Result<(usize, String, Fields), LineError> {
    Ok(match line {
        Line::Request { id, .. } => (0, id.to_string(), request_fields(line)?),
        Line::Open { window, .. } => (1, window.to_string(), open_fields(line)?),
        Line::Finding { id, topic, at } => (
            2,
            id.to_string(),
            vec![
                ("窓", id.window().to_string()),
                ("題", topic_word(topic)),
                ("時刻", minute(at)?.into()),
            ],
        ),
        Line::Receipt { subject, via, at } => (
            3,
            subject.to_string(),
            vec![("経路", via.word().into()), ("時刻", minute(at)?.into())],
        ),
        Line::Disposal { id, .. } => (4, id.to_string(), disposal_fields(line)?),
        Line::Close {
            window,
            by,
            findings,
            at,
        } => (
            5,
            window.to_string(),
            vec![
                ("起こし手", by.word().into()),
                ("所見", findings.to_string()),
                ("時刻", minute(at)?.into()),
            ],
        ),
    })
}

fn request_fields(line: &Line) -> Result<Fields, LineError> {
    let Line::Request {
        topic,
        form,
        model,
        at,
        ..
    } = line
    else {
        return Ok(Vec::new());
    };
    Ok(vec![
        ("題", topic_word(topic)),
        ("形", form.word().into()),
        ("model", word(model, WORD_MAX)?),
        ("起こし手", Starter::Button.word().into()),
        ("時刻", minute(at)?.into()),
    ])
}

fn disposal_fields(line: &Line) -> Result<Fields, LineError> {
    let Line::Disposal {
        verdict,
        reason,
        keep,
        drop,
        at,
        ..
    } = line
    else {
        return Ok(Vec::new());
    };
    Ok(vec![
        ("採否", verdict.word().into()),
        ("理由", word(reason, REASON_MAX)?),
        ("保存", paths(keep)?),
        ("不保存", paths(drop)?),
        ("時刻", minute(at)?.into()),
    ])
}

fn open_fields(line: &Line) -> Result<Fields, LineError> {
    let Line::Open {
        form,
        topic,
        by,
        model,
        effort,
        opened,
        again,
        at,
        ..
    } = line
    else {
        return Ok(Vec::new());
    };
    let mut fields = vec![
        ("形", form.word().to_string()),
        ("題", topic_word(topic)),
        ("起こし手", by.starter().word().to_string()),
    ];
    match by {
        By::Seat => {}
        By::Chat { uttered } => fields.push(("発話", minute(uttered)?.to_string())),
        By::Button { request } => fields.push(("頼み", request.to_string())),
    }
    fields.push(("model", word(model, WORD_MAX)?));
    fields.push(("念入りさ", word(effort, WORD_MAX)?));
    fields.push((
        "結果",
        if *opened { "開いた" } else { "落ちた" }.to_string(),
    ));
    if *again {
        fields.push(("撃ち直し", "はい".to_string()));
    }
    fields.push(("時刻", minute(at)?.to_string()));
    Ok(fields)
}

/// 行の字（末に改行を足さない）。自由な字は `free` で整え、引用の形が残れば断る。
pub fn render(line: &Line) -> Result<String, LineError> {
    let (head, id, fields) = parts(line)?;
    let mut text = format!("{}{EQ}{id}", HEADS.get(head).copied().unwrap_or_default());
    for (name, value) in fields {
        text.push(SEP);
        text.push_str(&format!("{name}{EQ}{value}"));
    }
    if cited(&text) {
        Err(LineError::Cited)
    } else {
        Ok(text)
    }
}

/// 行の字を読む（相談の行でないか欄が足りなければ None）。頼みの行は起こし手が持ち主の button の時だけ、
/// 所見の行は窓が所見の id の窓の時だけ読む（render が書く欄を全部見る）。
pub fn read(text: &str) -> Option<Line> {
    let mut items = text.trim_end_matches('\r').split(SEP);
    let (head, id) = items.next()?.split_once(EQ)?;
    let fields: Vec<(&str, &str)> = items.map(|p| p.split_once(EQ)).collect::<Option<_>>()?;
    let get = |name: &str| fields.iter().find(|(n, _)| *n == name).map(|(_, v)| *v);
    let topic = || get("題").map(|t| (t != NO_TOPIC).then(|| t.to_string()));
    let at = get("時刻").filter(|a| minute_ok(a))?.to_string();
    let index = HEADS.iter().position(|h| *h == head)?;
    match index {
        0 => {
            if Starter::from_word(get("起こし手")?)? != Starter::Button {
                return None;
            }
            Some(Line::Request {
                id: RequestId::parse(id).ok()?,
                topic: topic()?,
                form: Form::from_word(get("形")?)?,
                model: get("model")?.to_string(),
                at,
            })
        }
        1 => read_open(id, &get, topic()?, at),
        2 => {
            let id = FindingId::parse(id).ok()?;
            if get("窓")? != id.window().to_string() {
                return None;
            }
            Some(Line::Finding {
                id,
                topic: topic()?,
                at,
            })
        }
        3 => Some(Line::Receipt {
            subject: subject(id)?,
            via: Via::from_word(get("経路")?)?,
            at,
        }),
        4 => read_disposal(id, &get, at),
        _ => Some(Line::Close {
            window: WindowId::parse(id).ok()?,
            by: Starter::from_word(get("起こし手")?)?,
            findings: get("所見")?.parse().ok()?,
            at,
        }),
    }
}

fn read_open<'a>(
    id: &str,
    get: &impl Fn(&str) -> Option<&'a str>,
    topic: Option<String>,
    at: String,
) -> Option<Line> {
    let by = match Starter::from_word(get("起こし手")?)? {
        Starter::Seat => By::Seat,
        Starter::Chat => By::Chat {
            uttered: get("発話").filter(|h| minute_ok(h))?.to_string(),
        },
        Starter::Button => By::Button {
            request: RequestId::parse(get("頼み")?).ok()?,
        },
    };
    Some(Line::Open {
        window: WindowId::parse(id).ok()?,
        form: Form::from_word(get("形")?)?,
        topic,
        by,
        model: get("model")?.to_string(),
        effort: get("念入りさ")?.to_string(),
        opened: get("結果")? == "開いた",
        again: get("撃ち直し") == Some("はい"),
        at,
    })
}

fn read_disposal<'a>(id: &str, get: &impl Fn(&str) -> Option<&'a str>, at: String) -> Option<Line> {
    Some(Line::Disposal {
        id: FindingId::parse(id).ok()?,
        verdict: Verdict::from_word(get("採否")?)?,
        reason: get("理由")?.to_string(),
        keep: list(get("保存")?),
        drop: list(get("不保存")?),
        at,
    })
}

fn subject(id: &str) -> Option<Subject> {
    FindingId::parse(id)
        .map(Subject::Finding)
        .or_else(|_| RequestId::parse(id).map(Subject::Request))
        .ok()
}

fn list(value: &str) -> Vec<String> {
    if value == NONE {
        Vec::new()
    } else {
        value.split(LIST_SEP).map(str::to_string).collect()
    }
}

/// notes の相談の行の全部（台帳の順）。
pub fn scan(notes: &str) -> Vec<Line> {
    notes.lines().filter_map(read).collect()
}

/// 行の置き場（id）と、memo の notes が満ちて根に置いた時のその memo の id。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    pub id: BeadId,
    pub full: Option<BeadId>,
}

impl Home {
    /// 置き場に足す字（memo が満ちた時は末に `・memo = <id>（notes が満ちた）`）。
    pub fn line(&self, text: &str) -> String {
        match &self.full {
            Some(memo) => format!("{text}{SEP}memo{EQ}{memo}（notes が満ちた）"),
            None => text.to_string(),
        }
    }
}

/// 置き場が無い理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeError {
    /// 根の epic が無い。
    NoRoot,
}

/// 根の epic（種類 epic・親が無い・closed でない・いくつも在れば id の自然な順で最初）。
pub fn root_of(items: &[LedgerItem]) -> Option<&LedgerItem> {
    items
        .iter()
        .filter(|i| i.row.kind == "epic" && i.row.parent.is_none() && i.row.status != "closed")
        .min_by(|a, b| natural_cmp(a.row.id.as_str(), b.row.id.as_str()))
}

fn fits(item: &LedgerItem, text: &str) -> bool {
    item.notes.len() + text.len() < NOTES_MAX
}

/// 行の置き場。題が bead なら、その bead から親を上へたどった最も近い memo（題が memo ならそれ）、
/// 無ければ根。題が無いか台帳に無い字なら根（開きと閉じの行は題を渡さない）。
/// memo の notes に足すと `NOTES_MAX` を越えるなら根に置く（根の notes の長さは見ない）。
pub fn home_of(items: &[LedgerItem], topic: Option<&str>, text: &str) -> Result<Home, HomeError> {
    let root = root_of(items).ok_or(HomeError::NoRoot)?;
    let find = |id: &str| items.iter().find(|i| i.row.id.as_str() == id);
    let mut at = topic.and_then(find);
    let mut steps = 0;
    while let Some(item) = at.filter(|_| steps <= items.len()) {
        if item.row.is_memo() {
            if fits(item, text) {
                return Ok(Home {
                    id: item.row.id.clone(),
                    full: None,
                });
            }
            return Ok(Home {
                id: root.row.id.clone(),
                full: Some(item.row.id.clone()),
            });
        }
        at = item.row.parent.as_ref().and_then(|p| find(p.as_str()));
        steps += 1;
    }
    Ok(Home {
        id: root.row.id.clone(),
        full: None,
    })
}

/// 席に受けられていない所見（作業場に在って受けの行が無い）と頼み（頼みの行が在って受けの行が無い）。
pub fn unreceived(lines: &[Line], found: &[FindingId]) -> ConsultUnreceived {
    let received: Vec<&Subject> = lines
        .iter()
        .filter_map(|l| match l {
            Line::Receipt { subject, .. } => Some(subject),
            _ => None,
        })
        .collect();
    let got = |s: &Subject| received.contains(&s);
    let findings: BTreeSet<FindingId> = found
        .iter()
        .copied()
        .filter(|id| !got(&Subject::Finding(*id)))
        .collect();
    let mut requests: Vec<RequestId> = Vec::new();
    for line in lines {
        if let Line::Request { id, .. } = line
            && !got(&Subject::Request(id.clone()))
            && !requests.contains(id)
        {
            requests.push(id.clone());
        }
    }
    ConsultUnreceived {
        findings: findings.into_iter().collect(),
        requests,
    }
}

/// 見張りと起動の口が席へ出す事象。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// 所見が届いた（見張りは経路 見張り・問う窓の終わりは経路 完了）。
    Finding { id: FindingId, via: Via },
    /// 頼みが届いた。
    Request(RequestId),
    /// 問う窓が所見なしで止まった。
    Stalled(WindowId),
    /// 話す窓の最後の process が止まった（閉じの行は無い・閉じずに開き直しを待つ・判断の記録 ADR-55）。
    Gone(WindowId),
    /// 見張りが時間の上限で終わった。
    Timeout,
    /// 生きている話す窓が手すきで、席の今の口座と違う口座で動いている（行 cs-follow）。
    Drift(WindowId),
    /// 付いてくる口が待ちの上限までに手すきを見られず、起こし直さなかった（行 cs-follow）。
    Held(WindowId),
}

/// 席への固定の 1 行（`tzw` は tz の解き方の命令の字）。見張りの出す行は末に置き直しの命令を足す。
pub fn notice(event: &Event, tzw: &str) -> String {
    let body = match event {
        Event::Finding { id, via } => format!(
            "窓 {} の所見 {id} が届いた（{tzw} consult show {id} --via {}）",
            id.window(),
            via.word()
        ),
        Event::Request(id) => {
            format!("頼み {id} が届いた（{tzw} consult open --request {id} --by button）")
        }
        Event::Stalled(w) => {
            format!("問う窓 {w} が所見なしで止まった（{tzw} consult launch {w} --again）")
        }
        Event::Gone(w) => format!(
            "話す窓 {w} が止まった（開き直しを待つ・持ち主が閉じると言えば {tzw} consult close {w} --by chat）"
        ),
        Event::Timeout => "見張りが上限で終わった".to_string(),
        Event::Drift(w) => {
            format!("話す窓 {w} が前の口座で動いている（{tzw} consult launch {w} --follow）")
        }
        Event::Held(w) => format!("話す窓 {w} が手すきにならず付いてこなかった（持ち主に問う）"),
    };
    match event {
        Event::Finding { via: Via::Done, .. } | Event::Held(_) => format!("{NOTICE_HEAD}{body}"),
        _ => format!("{NOTICE_HEAD}{body}{SEP}「{tzw} consult watch」を背景で置き直す"),
    }
}
