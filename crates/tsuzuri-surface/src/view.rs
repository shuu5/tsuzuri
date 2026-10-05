//! 画面の中身を決める純粋な関数: 並べ方・状態の印・測れていないの判定・時刻の字（日本時間）。
//! DOM と通信に触らないので host の cargo test で試す（描くのは project の下の block・読むのは net）。
//! 件数と見出しは block の module が持つ（見出しの語は vocab から引く）。
//! 読みの結果の 3 値と読み直しの合図の event の名は block に共通の部品。
//! 合図の種類と口の path の表で、合図ごとに読み直す口を決める。
//! 読みの後に signal へ新しい値を置くかの決め方もここに置き、net が口の読みごとに呼ぶ。

use std::cmp::Ordering;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::graph::NodeKind;
use tsuzuri_contract::ledger::{LEDGER_CHANGED_EVENT, LedgerList, LedgerRow};
use tsuzuri_contract::project::ProjectName;
use tsuzuri_contract::surface::{BOARD_CHANGED_EVENT, BoardChanged, ChangeKind};
use tsuzuri_contract::{account, case, host, project, runs, wire};

use crate::frame::BRAND;
use crate::ledgerlist::FACTS_PATH;
use crate::project::{ask, ledger, map, next, nodearound, notice, pipeline, seat};
use crate::vocab::{label, vocab};

/// 読み直しの合図の event の名（台帳の変化と、器の event の記録か設計文書か席か account の変化・便 g-parts）。
/// 字は契約の型の crate の定数から引き、面の code に直に書かない。合図は種類（`changed_kinds`）を読む口だけを読み直す。
pub const RELOAD_EVENTS: [&str; 2] = [LEDGER_CHANGED_EVENT, BOARD_CHANGED_EVENT];

/// 口の path と、その口の読みが読む変化の種類（行 c-ev-kind・種類は server の口の読みから決めた）。
/// 席の card は席の状態の file だけ、次の一手は台帳と event log と席の card、pipeline は台帳と event log と
/// 台帳の形の行（台帳の種類）、グラフと近傍は設計の索引と台帳と event log、account は account board の印と
/// 自分の repo の台帳を読む。project の名の口は起動の repo から決まるので種類を持たない（合図では読み直さない）。
/// 席からの知らせの口は知らせの記録の file だけを読む（行 i-11）。host の口は host の種類だけ（中身が動いた時だけ・
/// 持ち主の裁定 t3-hub.77.28）。
/// bead の事実の口は台帳の一覧の口と同じ台帳の読みから組むので台帳だけ（行 g-ledger-trim）。器の局面の出力
/// （state dir の fleet/lifecycle.json と lifecycle.stale）は局面の出力の種類の合図が持つ（中身が動いた時だけ・
/// 行 c-cases-watch）: 局面の出力の口はその種類だけ、それを台帳と一緒に読む指標と未反映の口は台帳とその種類、
/// pipeline の口は台帳と event log とその種類で読み直す。
pub const RELOAD_KINDS: [(&str, &[ChangeKind]); 16] = [
    (seat::PATH, &[ChangeKind::Seat]),
    (
        next::PATH,
        &[ChangeKind::Ledger, ChangeKind::Runs, ChangeKind::Seat],
    ),
    (ledger::PATH, &[ChangeKind::Ledger]),
    (
        ledger::METRICS_PATH,
        &[ChangeKind::Ledger, ChangeKind::Cases],
    ),
    (ledger::UNREF_PATH, &[ChangeKind::Ledger, ChangeKind::Cases]),
    (FACTS_PATH, &[ChangeKind::Ledger]),
    (case::PATH, &[ChangeKind::Cases]),
    (ask::PATH, &[ChangeKind::Ledger]),
    (
        pipeline::PATH,
        &[ChangeKind::Ledger, ChangeKind::Runs, ChangeKind::Cases],
    ),
    (
        map::PATH,
        &[ChangeKind::Ledger, ChangeKind::Runs, ChangeKind::Design],
    ),
    (
        nodearound::PATH,
        &[ChangeKind::Ledger, ChangeKind::Runs, ChangeKind::Design],
    ),
    (runs::PATH, &[ChangeKind::Runs]),
    (account::PATH, &[ChangeKind::Account, ChangeKind::Ledger]),
    (project::PATH, &[]),
    (notice::PATH, &[ChangeKind::Notice]),
    (host::PATH, &[ChangeKind::Host]),
];

/// 口の path の読みが読む変化の種類: 最初の `?` より前の字が、表の path と同じか表の path に `/` を続けた字で
/// 始まる最初の行の種類（グラフの行がグラフの眺めを、台帳の行が台帳の 1 本を持つ）。当たる行が無ければ全部の種類
/// （表に無い口は今までどおり全部の合図で読み直す）。
pub fn path_kinds(path: &str) -> &'static [ChangeKind] {
    let bare = path.split('?').next().unwrap_or_default();
    RELOAD_KINDS
        .iter()
        .find(|(p, _)| {
            bare.strip_prefix(p)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
        })
        .map_or(&ChangeKind::ALL, |(_, kinds)| *kinds)
}

/// 口 `path` を種類 `kinds` の合図で読み直すか（`path_kinds` のどれかが `kinds` に在れば真）。
pub fn reloads(path: &str, kinds: &[ChangeKind]) -> bool {
    path_kinds(path).iter().any(|k| kinds.contains(k))
}

/// 合図の event の名と data の字から、動いた種類: 台帳の変化の名は台帳、板の変化の名は data の
/// `BoardChanged` の kinds（読めない・data が無い・kinds が空なら全部の種類）、ほかの名は空。
pub fn changed_kinds(event: &str, data: Option<&str>) -> Vec<ChangeKind> {
    if event == LEDGER_CHANGED_EVENT {
        return vec![ChangeKind::Ledger];
    }
    if event != BOARD_CHANGED_EVENT {
        return Vec::new();
    }
    match data.map(wire::decode::<BoardChanged>) {
        Some(Ok(b)) if !b.kinds.is_empty() => b.kinds,
        _ => ChangeKind::ALL.to_vec(),
    }
}

/// block の口から読んだ結果の 3 値（net が作る・便 g-parts）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fetched {
    /// まだ読んでいない（頁を開いた直後）。
    NotRead,
    /// 口が 200 で返した本文。
    Body(String),
    /// 口に届かない・200 でない・本文が読めない・知らせが切れた。
    Failed,
}

/// 読みの後に signal へ新しい値を置くか（閉じた 3 値・便 g-steady）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settle {
    /// 新しい値を置く。
    Set,
    /// 置かない（block を組み直さない）。
    Keep,
    /// 置かずに `RETRY_MS` の後にもう 1 回読む。
    Retry,
}

/// 一度の読めないの後に読み直すまでの間（ms）。
pub const RETRY_MS: u64 = 1000;

/// 1 つの口の読みの後の決め方（`attempt` は何回目の読みか・1 から）。上から順に当てる:
/// 新しい値が今の値と同じなら置かない・今の値が本文で新しい値が読めないで 1 回目なら読み直す・ほかは置く
/// （2 回目の読めないは置いて測れていないを出す・要件 NFR2）。
pub fn settle(now: &Fetched, new: &Fetched, attempt: u32) -> Settle {
    if now == new {
        return Settle::Keep;
    }
    match (now, new) {
        (Fetched::Body(_), Fetched::Failed) if attempt <= 1 => Settle::Retry,
        _ => Settle::Set,
    }
}

/// 頁の題の board の語（index.html の title の後ろの字と同じ）。
pub const BOARD_WORDS: &str = "project board";

/// project の名の口を読んだ後の名（行 g-brand）: 本文が ProjectName に読めて名が空でなければその名、
/// ほかは前の名（一度読めた名は、読めない間も次に読めるまで持ち続ける）。
pub fn kept_name(before: Option<String>, fetched: &Fetched) -> Option<String> {
    match fetched {
        Fetched::Body(body) => match wire::decode::<ProjectName>(body) {
            Ok(p) if !p.name.is_empty() => Some(p.name),
            _ => before,
        },
        Fetched::NotRead | Fetched::Failed => before,
    }
}

/// header の題の字（名が在ればそれ、無ければ frame の BRAND）。
pub fn brand(name: Option<&str>) -> &str {
    name.unwrap_or(BRAND)
}

/// 頁の題（題の字・空白・長い横棒・空白・board の語）。
pub fn board_title(name: Option<&str>) -> String {
    format!("{} \u{2014} {BOARD_WORDS}", brand(name))
}

/// 頁の題の語の字の数の上限（見本の ui.js の cut の 24・行 g-title）。
pub const SUBJECT_CHARS: usize = 24;

/// 頁の見出しの語に替える字（節点の頁の節点の題・行 g-title）。None なら見出しの語。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageSubject(pub Option<String>);

/// 頁ごとの題（題の字・空白・長い横棒・空白・語・行 g-title）。語は subject が在って空白だけでなければ
/// subject、ほかは見出しの語の鍵の語で、SUBJECT_CHARS の字を超えれば先頭の 1 字少ない字と「…」にする。
pub fn doc_title(name: Option<&str>, heading: &str, subject: Option<&str>) -> String {
    let word = match subject {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => label(heading),
    };
    let word = if word.chars().count() > SUBJECT_CHARS {
        let head: String = word.chars().take(SUBJECT_CHARS - 1).collect();
        format!("{head}\u{2026}")
    } else {
        word
    };
    format!("{} \u{2014} {word}", brand(name))
}

/// 口の本文を台帳の行に読む（まだ読んでいない・届かない・電文が読めない・台帳が読めないは Unknown）。
pub fn read_rows(fetched: &Fetched) -> Reading<Vec<LedgerRow>> {
    match fetched {
        Fetched::Body(body) => match wire::decode::<LedgerList>(body) {
            Ok(list) => list.rows,
            Err(_) => Reading::Unknown,
        },
        Fetched::NotRead | Fetched::Failed => Reading::Unknown,
    }
}

/// id の順（`.` で区切った段ごとに、数なら数の大小・字なら字の順: `a.2` は `a.10` より前）。
pub fn id_order(a: &str, b: &str) -> Ordering {
    let mut xs = a.split('.');
    let mut ys = b.split('.');
    loop {
        match (xs.next(), ys.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let o = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(m), Ok(n)) => m.cmp(&n).then_with(|| x.cmp(y)),
                    _ => x.cmp(y),
                };
                if o != Ordering::Equal {
                    return o;
                }
            }
        }
    }
}

/// 状態の印（字の印・日本語の 1 語・class の名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mark {
    pub glyph: &'static str,
    pub word: &'static str,
    pub class: &'static str,
}

/// bd の状態の語を印にする（知らない語は「?」の印で、語は title に残す）。
pub fn mark(status: &str) -> Mark {
    let (glyph, word, class) = match status {
        "open" => ("○", "未着手", "st-open"),
        "in_progress" => ("◐", "作業中", "st-in-progress"),
        "blocked" => ("⊘", "止まり", "st-blocked"),
        "deferred" => ("◌", "先送り", "st-deferred"),
        "closed" => ("●", "閉じた", "st-closed"),
        _ => ("?", "知らない状態", "st-other"),
    };
    Mark { glyph, word, class }
}

/// open の memo の状態の語の鍵（語の辞書の rephrase）。
pub const MEMO_OPEN_KEY: &str = "memo_unknown";

/// 台帳の行の状態の印（種類は地図の節点と同じ読み `LedgerRow::node_kind`）。
/// open の問いは持ち主の答えを待つので「◷ 答え待ち」、open の memo は語の辞書の `MEMO_OPEN_KEY` の語
/// （鍵が無ければ鍵の字）、ほかは bd の状態の `mark`。
/// memo の局面（処置の待ち・問いの待ち・形の崩れ）は器の局面の出力が決め、tsuzuri は判じない（要件 FR13）。
/// 後の行 c-ledger-lc がこの fn だけを局面の読みに替える。
pub fn row_mark(row: &LedgerRow) -> Mark {
    match (row.node_kind(), row.status.as_str()) {
        (NodeKind::Question, "open") => Mark {
            glyph: "◷",
            word: "答え待ち",
            class: "st-open",
        },
        (NodeKind::Memo, "open") => Mark {
            glyph: "?",
            word: vocab()
                .term(MEMO_OPEN_KEY)
                .map_or(MEMO_OPEN_KEY, |t| t.label.as_str()),
            class: "st-open",
        },
        _ => mark(&row.status),
    }
}

/// 日本時間の UTC からの差（秒・UTC に 9 時間を足す固定・行 g-jst）。browser の時間帯は読まない。
/// 面が人に見せる時刻の字だけに使い、記録の id の中の UTC の字と電文の epoch 秒は変えない。
pub const JST_OFFSET: EpochSecs = 32_400;

/// 日本時間の印（時刻の字の末に空白 1 つを挟んで付ける）。
pub const JST: &str = "JST";

/// 1 日の秒。
const DAY: EpochSecs = 86_400;

/// epoch 秒を日本時間の日の番号（1970-01-01 からの日数）とその日の 0 時からの秒の組にする。
/// 面の時刻の字と日の境の決めは全部この関数を通る。
pub fn jst(at: EpochSecs) -> (EpochSecs, EpochSecs) {
    let t = at + JST_OFFSET;
    (t / DAY, t % DAY)
}

/// epoch 秒を日本時間の字にする（`2026-09-27 16:39:00 JST`）。
pub fn clock(at: EpochSecs) -> String {
    let (days, secs) = jst(at);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} {JST}",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

/// 日本時間の時と分（`16:39`・印なし）。
pub fn hhmm(at: EpochSecs) -> String {
    clock(at)[11..16].to_string()
}

/// epoch 秒を短い日本時間の字にする（見本の hmd と同じ決め方）: 今の時刻との差が 20 時間以内なら
/// `21:53 JST`、超えれば月と日を前に足す（`09-26 21:53 JST`）。差は向きを問わない。
pub fn clock_short(at: EpochSecs, now: EpochSecs) -> String {
    let hm = format!("{} {JST}", hhmm(at));
    if at.abs_diff(now) <= 20 * 3600 {
        return hm;
    }
    format!("{} {hm}", &clock(at)[5..10])
}

/// 1970-01-01 からの日数を (年, 月, 日) にする（先発グレゴリオ暦）。
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + u64::from(m <= 2);
    (y, m, d)
}
