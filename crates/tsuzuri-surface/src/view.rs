//! 画面の中身を決める純粋な関数（便 g-min）: 並べ方・状態の印・測れていないの判定・時刻の字。
//! DOM と通信に触らないので host の cargo test で試す（描くのは project の下の block・読むのは net）。
//! 件数と見出しは block の module が持つ（便 g-frame・見出しの語は vocab から引く）。
//! 読みの結果の 3 値と読み直しの合図の event の名は block に共通の部品（便 g-parts）。
//! 読みの後に signal へ新しい値を置くかの決め方（便 g-steady）もここに置き、net が口の読みごとに呼ぶ。

use std::cmp::Ordering;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LEDGER_CHANGED_EVENT, LedgerList, LedgerRow};
use tsuzuri_contract::surface::BOARD_CHANGED_EVENT;
use tsuzuri_contract::wire;

/// 問いの bead の種類。
pub const QUESTION_KIND: &str = "question";

/// epic の bead の種類。
pub const EPIC_KIND: &str = "epic";

/// 読み直しの合図の event の名（台帳の変化と、器の event の記録か設計文書の変化・便 g-parts）。
/// 字は契約の型の crate の定数から引き、面の code に直に書かない。
pub const RELOAD_EVENTS: [&str; 2] = [LEDGER_CHANGED_EVENT, BOARD_CHANGED_EVENT];

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

/// 1 つの epic とその下の bead（epic の外の bead は `epic` が None の組に入る）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpicGroup {
    pub epic: Option<LedgerRow>,
    pub children: Vec<LedgerRow>,
}

/// 読めた台帳から組んだ画面の中身（問いの一覧と台帳の一覧）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub questions: Vec<LedgerRow>,
    pub groups: Vec<EpicGroup>,
}

/// 画面の状態。台帳が読めなければ `Unknown`（0 件と区別する・要件 NFR2）。
#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    pub board: Reading<Board>,
    /// 最後に読めた時刻（まだ 1 度も読めていなければ None）。
    pub updated_at: Option<EpochSecs>,
}

impl Screen {
    /// 起動の直後（まだ読んでいない＝測れていない）。
    pub fn initial() -> Self {
        Self {
            board: Reading::Unknown,
            updated_at: None,
        }
    }

    /// 一覧の口を読んだ後の状態。読めなければ測れていないにし、最終更新は前の値を残す。
    pub fn after_read(&self, fetched: &Fetched, at: EpochSecs) -> Self {
        match read_rows(fetched) {
            Reading::Known(rows) => Self {
                board: Reading::Known(board(&rows)),
                updated_at: Some(at),
            },
            Reading::Unknown => self.after_lost(),
        }
    }

    /// 変化の知らせが切れた後の状態（届かない間は今の一覧が正しいと言えない）。
    pub fn after_lost(&self) -> Self {
        Self {
            board: Reading::Unknown,
            updated_at: self.updated_at,
        }
    }
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

/// 台帳の行から画面の中身を組む。
pub fn board(rows: &[LedgerRow]) -> Board {
    Board {
        questions: questions(rows),
        groups: ledger_groups(rows),
    }
}

/// 問いの一覧: open の question を古い順（更新時刻の昇順・同じ時刻は id の順）。
pub fn questions(rows: &[LedgerRow]) -> Vec<LedgerRow> {
    let mut out: Vec<LedgerRow> = rows
        .iter()
        .filter(|r| r.kind == QUESTION_KIND && r.status == "open")
        .cloned()
        .collect();
    out.sort_by(|a, b| {
        a.updated_at
            .cmp(&b.updated_at)
            .then_with(|| id_order(a.id.as_str(), b.id.as_str()))
    });
    out
}

/// 台帳の一覧: epic ごとの組（epic の id の順）に、その下の bead を task・memo・その他の順で並べる。
/// 親は id の階層（`a.1` の親は `a`）で決め、いちばん近い epic の祖先の下に置く。
/// epic の祖先の無い bead は最後の組（epic が None）に入る。question は問いの一覧が持つので入れない。
pub fn ledger_groups(rows: &[LedgerRow]) -> Vec<EpicGroup> {
    let mut epics: Vec<&LedgerRow> = rows.iter().filter(|r| r.kind == EPIC_KIND).collect();
    epics.sort_by(|a, b| id_order(a.id.as_str(), b.id.as_str()));
    let mut groups: Vec<EpicGroup> = epics
        .iter()
        .map(|e| EpicGroup {
            epic: Some((*e).clone()),
            children: Vec::new(),
        })
        .collect();
    let mut outside = Vec::new();
    for row in rows
        .iter()
        .filter(|r| r.kind != EPIC_KIND && r.kind != QUESTION_KIND)
    {
        let home =
            ancestors(row.id.as_str()).find_map(|a| epics.iter().position(|e| e.id.as_str() == a));
        match home {
            Some(i) => groups[i].children.push(row.clone()),
            None => outside.push(row.clone()),
        }
    }
    for group in &mut groups {
        sort_children(&mut group.children);
    }
    if !outside.is_empty() {
        sort_children(&mut outside);
        groups.push(EpicGroup {
            epic: None,
            children: outside,
        });
    }
    groups
}

/// id の祖先（近い順: `a.b.c` なら `a.b`・`a`）。
fn ancestors(id: &str) -> impl Iterator<Item = &str> {
    std::iter::successors(parent(id), |p| parent(p))
}

fn parent(id: &str) -> Option<&str> {
    id.rsplit_once('.').map(|(p, _)| p)
}

/// epic の下の順: task・memo・その他、同じ種類の中は id の順。
fn sort_children(children: &mut [LedgerRow]) {
    children.sort_by(|a, b| {
        kind_rank(&a.kind)
            .cmp(&kind_rank(&b.kind))
            .then_with(|| id_order(a.id.as_str(), b.id.as_str()))
    });
}

fn kind_rank(kind: &str) -> u8 {
    match kind {
        "task" => 0,
        "memo" => 1,
        _ => 2,
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

/// epoch 秒を UTC の字にする（`2026-09-27 07:39:00 UTC`）。
pub fn clock(at: EpochSecs) -> String {
    let days = at / 86_400;
    let secs = at % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

/// epoch 秒を短い UTC の字にする（見本の hmd と同じ決め方）: 今の時刻との差が 20 時間以内なら
/// `12:53Z`、超えれば月と日を前に足す（`09-26 12:53Z`）。差は向きを問わない。
pub fn clock_short(at: EpochSecs, now: EpochSecs) -> String {
    let secs = at % 86_400;
    let hm = format!("{:02}:{:02}Z", secs / 3600, secs % 3600 / 60);
    if at.abs_diff(now) <= 20 * 3600 {
        return hm;
    }
    let (_, m, d) = civil_from_days(at / 86_400);
    format!("{m:02}-{d:02} {hm}")
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
