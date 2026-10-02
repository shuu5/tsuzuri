//! block「台帳」（見本の `#ledger` と index.html の ledgerBlock・ledger.js の描き方の関数・便 g-ledger）:
//! 見出しの横に判定の 1 語を出し、中身は台帳 open の一覧（module ledgerlist の epic ごとの組）だけを描く。
//! 前の指標の段（上段の 4 数・主な指標の行・burndown・memo の段）は外し、指標は一覧の見出しの小さな数と図に縮めた
//! （判断の記録 ADR-27 の決定 (8)・行 g-ledger-trim）。未反映の段（数と一覧）は帯の抜けの検査の窓の下へ移した（要件 FR13）。
//! 指標は口 /api/metrics（本文は契約の型の Reading で包んだ LedgerStats）から読む。数え方と判定は中核の crate が済ませていて、
//! ここは写すだけ（数え直しと判定の分岐を持たない）。一覧の見出しの図（burndown の座標と svg と card）と純減の字も
//! この file の関数で、sparkline と判定の表は account board も引く。
//! 一覧の口（/api/ledger）は問いの一覧（ask）と同じ口で、定数はこの module に 1 本だけ置く。
//! 前の一覧の組の純粋な関数（`body`・`staged_body`・`group_cards`・閉じた bead を出さない・行 g-ledger-home と c-ledger-stage）は
//! 件数と歯のために残し、DOM は描かない。
//! 未反映の種類の見出しは語の辞書の鍵 `unref:` と種類の名の label で、account board もここの関数で引く（行 g-kind-label）。
//! 器の局面と手番の語の平易な字も同じ辞書の鍵 `lc:` と `turn:` で引き、知らない語は「まだ分からない」に倒す（行 g-unref-lc）。
//! 未反映の数は 3 種とも分からなければ数えない字 ― にし、1 種でも分かれば数に測れていないの印を添える（行 g-unref-dash）。
//! 字と座標は純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use std::collections::BTreeMap;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::{LedgerJudge, Reading};
use tsuzuri_contract::ledger::LedgerRow;
use tsuzuri_contract::stats::{
    DayCount, LedgerStats, UnreflectedKind, UnreflectedList, UnreflectedRow,
};
use tsuzuri_contract::wire;

use super::seat::hm;
use super::{Body, Item, LEDGER_UNREAD, NO_CONTENT, NOT_READ, Staged, item, staged_item};
use crate::frame::Block;
use crate::view::{Fetched, Screen, clock};
use crate::vocab::{label, vocab};
use crate::widgets::hover::Card;
use crate::widgets::nodecard::card_of;

pub const BLOCK: Block = Block {
    id: "ledger",
    heading: "ledger_block",
    class: "panel",
};

/// 台帳の一覧の口（server の便 e-min・問いの一覧も読む）。
pub const PATH: &str = "/api/ledger";

/// 指標の段の口（便 g-parts）。
pub const METRICS_PATH: &str = "/api/metrics";

/// 未反映の一覧の口（本文は契約の型の UnreflectedList・便 e-view）。
pub const UNREF_PATH: &str = "/api/unreflected";

/// この file が字を持つ口の path（ほかの module の口を読む所は数えない・行 hs-derived）。
pub const PATHS: &[&str] = &[PATH, METRICS_PATH, UNREF_PATH];

/// この file の畳める段の開き閉じの鍵の形（行 hs-derived）。
pub const FOLDS: &[&str] = &["ledger:unref"];

/// 指標の段の口が読めないときの理由。
pub const METRICS_REASON: &str =
    "台帳の指標（積みと速度）を組む口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 口は読めたが、server が台帳を読めず指標が「まだ分からない」ときの理由。
pub const METRICS_UNKNOWN: &str = "server が台帳を読めないので、指標（積みと速度）はまだ分からない";

/// 測れて 0 件のときの 1 行。
pub const EMPTY: &str = "台帳に bead は無い";

/// 一覧に出さない状態の字（bd の語・行 g-ledger-home）。
pub const CLOSED: &str = "closed";

/// 台帳に行は在るが、閉じていない行が 1 つも無いときの 1 行（EMPTY と分ける）。
pub const NO_OPEN: &str = "閉じていない bead は無い";

/// 一覧に出す行か（状態が CLOSED の行は出さない・全件は地図の方で見る）。
pub fn listed(row: &LedgerRow) -> bool {
    row.status != CLOSED
}

/// epic の外の組の見出しの字（epic の項の代わり）。
pub const OUTSIDE: &str = "epic の外";

/// 値なしの字（年齢・lead）。
pub const NONE: &str = "―";

/// burndown と sparkline の日の数。
pub const DAYS: usize = 14;

/// burndown の図の幅と高さ（見本の burndown(LS, 320, 72)）。
pub const BURN_W: f64 = 320.0;
pub const BURN_H: f64 = 72.0;

/// sparkline の幅と高さ（見本の spark14 の既定）。
pub const SPARK_W: f64 = 120.0;
pub const SPARK_H: f64 = 24.0;

/// 一覧の 1 組（epic の項・その下の項）。epic の外の組は `head` が None。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub head: Option<Item>,
    pub children: Vec<Item>,
}

/// 判定の 1 語の写し（記号・class・語の鍵）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Judge {
    pub judge: LedgerJudge,
    pub symbol: &'static str,
    pub class: &'static str,
    pub key: &'static str,
}

/// 判定の 5 値の表（1 か所・悪い順）。
pub const JUDGES: [Judge; 5] = [
    Judge {
        judge: LedgerJudge::Clogged,
        symbol: "!",
        class: "jdg j-bad",
        key: "j_bad",
    },
    Judge {
        judge: LedgerJudge::PilingUp,
        symbol: "↑",
        class: "jdg j-up",
        key: "j_up",
    },
    Judge {
        judge: LedgerJudge::Stalled,
        symbol: "→",
        class: "jdg j-stall",
        key: "j_stall",
    },
    Judge {
        judge: LedgerJudge::OnTrack,
        symbol: "✓",
        class: "jdg j-ok",
        key: "j_ok",
    },
    Judge {
        judge: LedgerJudge::NoLedger,
        symbol: "―",
        class: "jdg j-none",
        key: "j_none",
    },
];

/// 判定の写し（表から引く）。
#[expect(clippy::expect_used, reason = "判定の表は 5 値の全部を持つ")]
pub fn judge(value: LedgerJudge) -> Judge {
    *JUDGES
        .iter()
        .find(|j| j.judge == value)
        .expect("判定の表は 5 値の全部を持つ")
}

/// 純減の矢印と数（見本の netHTML）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Net {
    pub arrow: &'static str,
    pub class: &'static str,
    /// 読み上げの語（純減・純増・横ばい）。
    pub word: &'static str,
    pub text: String,
}

/// 純減の値（今 − 過去・負が純減）を矢印と数にする。負は「−」と絶対値・正は「+」と値・0 は「0」。
pub fn net(value: i64) -> Net {
    let (arrow, class, word, text) = match value.signum() {
        -1 => (
            "↓",
            "net net-down",
            "純減",
            format!("−{}", value.unsigned_abs()),
        ),
        1 => ("↑", "net net-up", "純増", format!("+{value}")),
        _ => ("→", "net net-flat", "横ばい", "0".to_string()),
    };
    Net {
        arrow,
        class,
        word,
        text,
    }
}

/// 小数 1 桁の字（見本の toFixed(1)・ちょうど半分は大きい方へ）。
pub fn fixed1(value: f64) -> String {
    format!("{:.1}", (value * 10.0).round() / 10.0)
}

/// 日数の字（見本の days）: 値なしは「―」・1 日未満は時間の h・10 日未満は小数 1 桁の d・それ以上は整数の d。
pub fn age(days: Option<f64>) -> String {
    match days {
        None => NONE.to_string(),
        Some(d) if d < 1.0 => format!("{:.0}h", (d * 24.0).round()),
        Some(d) if d < 10.0 => format!("{}d", fixed1(d)),
        Some(d) => format!("{:.0}d", d.round()),
    }
}

/// 時刻 at から今 now までの日数（今より後の時刻は 0・memo と未反映の年齢は面の今から引く・行 c-abs-time）。
pub fn days_since(at: EpochSecs, now: EpochSecs) -> f64 {
    now.saturating_sub(at) as f64 / 86_400.0
}

/// 未反映の種類の名（1 か所の表）。
pub const UNREF_KINDS: [(UnreflectedKind, &str); 3] = [
    (UnreflectedKind::Memo, "memo"),
    (UnreflectedKind::Ruling, "ruling"),
    (UnreflectedKind::Utterance, "utterance"),
];

/// 未反映の種類の名（表から引く）。
#[expect(clippy::expect_used, reason = "種類の表は 3 つの全部を持つ")]
pub fn kind_name(kind: UnreflectedKind) -> &'static str {
    UNREF_KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, n)| *n)
        .expect("種類の表は 3 つの全部を持つ")
}

/// 未反映の種類の見出しの語の鍵の接頭（鍵は接頭と kind_name の字）。
pub const UNREF_KIND_KEY: &str = "unref:";

/// 未反映の種類の見出し（語の辞書の label・見本の unrefBreak と unrefHTML の字）。
pub fn kind_label(kind: UnreflectedKind) -> String {
    name_label(kind_name(kind))
}

/// 未反映の種類の名（kind_name の字）の見出し（語の辞書の label）。
pub fn name_label(name: &str) -> String {
    label(&format!("{UNREF_KIND_KEY}{name}"))
}

/// 器の局面の語の語の鍵の接頭（鍵は接頭と局面の語・行 g-unref-lc）。
pub const PHASE_KEY: &str = "lc:";

/// 器の手番の語の語の鍵の接頭（鍵は接頭と手番の語）。
pub const TURN_KEY: &str = "turn:";

/// 語の辞書に無い局面と手番の語に出す字の鍵（まだ分からない）。
pub const UNKNOWN_WORD_KEY: &str = "gap_unknown";

/// 局面か手番の語の平易な字（接頭と語の鍵が語の辞書に在ればその見出し・無ければ「まだ分からない」）。
pub fn plain_word(prefix: &str, word: &str) -> String {
    match vocab().term(&format!("{prefix}{word}")) {
        Some(term) => term.label.clone(),
        None => label(UNKNOWN_WORD_KEY),
    }
}

/// 局面と手番の平易な字（局面の字・中黒の区切り・手番の字）。
pub fn phase_text(phase: &str, turn: &str) -> String {
    format!(
        "{} · {}",
        plain_word(PHASE_KEY, phase),
        plain_word(TURN_KEY, turn)
    )
}

/// 未反映の種類ごとの器の局面の語（行 c-unref-lc の 3 つ・手番はどれも席）。
pub const UNREF_PHASES: [(UnreflectedKind, &[&str]); 3] = [
    (UnreflectedKind::Memo, &["memo-actionable", "misfit"]),
    (UnreflectedKind::Ruling, &["ruling-unreflected"]),
    (UnreflectedKind::Utterance, &["utterance-open"]),
];

/// 未反映の種類の局面の平易な字（局面の字を・でつなぎ、手番の席の字を添える）。
#[expect(clippy::expect_used, reason = "局面の表は 3 つの全部を持つ")]
pub fn kind_phase_text(kind: UnreflectedKind) -> String {
    let phases = UNREF_PHASES
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, p)| *p)
        .expect("局面の表は 3 つの全部を持つ");
    let words: Vec<String> = phases.iter().map(|p| plain_word(PHASE_KEY, p)).collect();
    format!("{} · {}", words.join("・"), plain_word(TURN_KEY, "seat"))
}

/// 未反映の数（電文の数そのまま）と、分からない種類の名（測れていないの記号を添えて出す）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unref {
    pub count: u32,
    pub unknown: Vec<&'static str>,
}

impl Unref {
    /// 数の字（3 種とも分からなければ数えない字 NONE・ほかは数・行 g-unref-dash）。
    pub fn text(&self) -> String {
        if self.unknown.len() < UnreflectedKind::ALL.len() {
            self.count.to_string()
        } else {
            NONE.to_string()
        }
    }

    /// 指標の電文の未反映の数と分からない種類の名（電文の数そのまま・抜けの検査の窓の未反映の段・行 g-ledger-trim）。
    pub fn of(s: &LedgerStats) -> Unref {
        Unref {
            count: s.unreflected,
            unknown: s
                .unreflected_unknown
                .iter()
                .map(|k| kind_name(*k))
                .collect(),
        }
    }

    /// 数に測れていないの印を添えるか（分からない種類が 1 つ以上で、3 種の全部ではない時だけ）。
    pub fn partial(&self) -> bool {
        !self.unknown.is_empty() && self.unknown.len() < UnreflectedKind::ALL.len()
    }
}

/// burndown の棒の 1 本。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// burndown の図（棒 = 閉じた数 / 日・線 = その日の終わりの open の task）。
#[derive(Debug, Clone, PartialEq)]
pub struct Burn {
    pub bars: Vec<Bar>,
    pub line: Vec<(f64, f64)>,
}

/// burndown の座標（見本の burndown と同じ式）。
pub fn burndown(days: &[DayCount], w: f64, h: f64) -> Burn {
    let most_open = days.iter().map(|d| d.open).fold(1, u32::max);
    let most_closed = days.iter().map(|d| d.closed).fold(1, u32::max);
    let bw = (w - 8.0) / DAYS as f64;
    let bars = days
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let height = f64::from(d.closed) / f64::from(most_closed) * (h * 0.45);
            Bar {
                x: 4.0 + i as f64 * bw + 1.0,
                y: h - height,
                width: (bw - 2.0).max(1.0),
                height,
            }
        })
        .collect();
    let line = days
        .iter()
        .enumerate()
        .map(|(i, d)| {
            (
                4.0 + i as f64 * bw + bw / 2.0,
                4.0 + (1.0 - f64::from(d.open) / f64::from(most_open)) * (h * 0.5),
            )
        })
        .collect();
    Burn { bars, line }
}

/// sparkline の 2 本（created の点線・closed の実線）。
#[derive(Debug, Clone, PartialEq)]
pub struct Spark {
    pub created: Vec<(f64, f64)>,
    pub closed: Vec<(f64, f64)>,
}

/// sparkline の座標（見本の spark14 と同じ式）。
pub fn spark(days: &[DayCount], w: f64, h: f64) -> Spark {
    let most = days
        .iter()
        .map(|d| d.created.max(d.closed))
        .fold(1, u32::max);
    let at = |i: usize, v: u32| {
        (
            i as f64 / (DAYS - 1) as f64 * (w - 2.0) + 1.0,
            h - 2.0 - f64::from(v) / f64::from(most) * (h - 4.0),
        )
    };
    Spark {
        created: days
            .iter()
            .enumerate()
            .map(|(i, d)| at(i, d.created))
            .collect(),
        closed: days
            .iter()
            .enumerate()
            .map(|(i, d)| at(i, d.closed))
            .collect(),
    }
}

/// 線の点の字（`x,y x,y …`・小数 1 桁）。
pub fn points(line: &[(f64, f64)]) -> String {
    line.iter()
        .map(|(x, y)| format!("{},{}", fixed1(*x), fixed1(*y)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// burndown の svg の字（見本の burndown と同じ形）。
pub fn burn_svg(burn: &Burn) -> String {
    let bars: String = burn
        .bars
        .iter()
        .map(|b| {
            format!(
                r#"<rect class="lb-closed" x="{}" y="{}" width="{}" height="{}"/>"#,
                fixed1(b.x),
                fixed1(b.y),
                fixed1(b.width),
                fixed1(b.height)
            )
        })
        .collect();
    format!(
        r#"<svg class="lburn" viewBox="0 0 {BURN_W} {BURN_H}" preserveAspectRatio="none" role="img" aria-label="burndown 14 日">{bars}<polyline class="lb-open" fill="none" points="{}"/></svg>"#,
        points(&burn.line)
    )
}

/// sparkline の svg の字（見本の spark14 と同じ形）。
pub fn spark_svg(spark: &Spark) -> String {
    format!(
        r#"<svg class="lspark" viewBox="0 0 {SPARK_W} {SPARK_H}" width="{SPARK_W}" height="{SPARK_H}" role="img" aria-label="14 日の created と closed"><polyline class="ls-created" fill="none" points="{}"/><polyline class="ls-closed" fill="none" points="{}"/></svg>"#,
        points(&spark.created),
        points(&spark.closed)
    )
}

/// 一覧の組の項の節点の card（組ごとに epic の項と下の項・電文に在る id だけ・行 g-card-adopt-c）。
pub fn group_cards(groups: &[Group], graph: &Fetched) -> BTreeMap<String, Card> {
    cards_of(
        groups
            .iter()
            .flat_map(|g| g.head.iter().chain(&g.children).map(|i| i.id.as_str())),
        graph,
    )
}

/// id の列のうち電文の節点に在るものの card（id の字の鍵）。
fn cards_of<'a>(ids: impl Iterator<Item = &'a str>, graph: &Fetched) -> BTreeMap<String, Card> {
    let Ok(doc) = super::map::doc(graph) else {
        return BTreeMap::new();
    };
    ids.filter_map(|id| card_of(&doc, id).map(|c| (id.to_string(), c)))
        .collect()
}

/// 指標の口の本文を電文に読む（本文は読めた指標か「まだ分からない」・まだ読んでいない・読めない・
/// 台帳が読めない・電文が読めないは理由）。
pub fn stats(fetched: &Fetched) -> Result<LedgerStats, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(METRICS_REASON),
        Fetched::Body(text) => match wire::decode::<Reading<LedgerStats>>(text) {
            Ok(Reading::Known(s)) => Ok(s),
            Ok(Reading::Unknown) => Err(METRICS_UNKNOWN),
            Err(_) => Err(NO_CONTENT),
        },
    }
}

/// burndown の数の出所（台帳の読み）。
pub const BURN_SRC: &str = "bd list --all の task";

/// burndown の図の card（見本の ledgerCard の burn の枝）: 題・純減 24h と 7d と closed/日・
/// 14 日の始めと終わりの open・出所と時点。詳しくは最後の日から 1 日おきに選んだ日を古い順に。
pub fn burn_card(s: &LedgerStats) -> Card {
    let n24 = net(s.net_drop_24h);
    let n7 = net(s.net_drop_7d);
    let kind = format!(
        "{}{} 24h · {}{} 7d · {} {}",
        n24.arrow,
        n24.text,
        n7.arrow,
        n7.text,
        label("l_rate"),
        fixed1(s.closed_per_day)
    );
    let value = match (s.days.first(), s.days.last()) {
        (Some(first), Some(last)) => {
            format!("open {} → {}（{} 日）", first.open, last.open, s.days.len())
        }
        _ => format!("open {NONE}"),
    };
    let n = s.days.len();
    let more = s
        .days
        .iter()
        .enumerate()
        .filter(|(i, _)| (n - 1 - i).is_multiple_of(2))
        .map(|(_, d)| {
            format!(
                "{} open {} · +{} / −{}",
                &clock(d.end)[5..10],
                d.open,
                d.created,
                d.closed
            )
        })
        .collect();
    Card {
        title: label("l_burn"),
        kind,
        value,
        src: format!("{BURN_SRC} · 時点 {}", hm(s.at)),
        more,
    }
}

/// 台帳の件数（epic も数える・測れていなければ Unknown）。
pub fn count(screen: &Screen) -> Reading<usize> {
    match &screen.board {
        Reading::Known(b) => Reading::Known(
            b.groups
                .iter()
                .map(|g| g.children.len() + usize::from(g.epic.is_some()))
                .sum(),
        ),
        Reading::Unknown => Reading::Unknown,
    }
}

/// 一覧の中身（閉じた行は出さない・閉じた epic の頭は、閉じていない下の項が在るときだけ残す・行 g-ledger-home）。
/// 板の段は持たない（`staged_body` に空の段を渡した値）。
pub fn body(screen: &Screen) -> Body<Vec<Group>> {
    staged_body(screen, &BTreeMap::new())
}

/// 板の段つきの一覧の中身（`body` と同じ組で、下の項は bead の id の段が在れば段の字と記号を出す・epic の頭は段を出さない）。
pub fn staged_body(screen: &Screen, stages: &BTreeMap<String, Staged>) -> Body<Vec<Group>> {
    match &screen.board {
        Reading::Unknown => Body::Unmeasured(LEDGER_UNREAD),
        Reading::Known(b) if b.groups.is_empty() => Body::Empty(EMPTY),
        Reading::Known(b) => {
            let groups: Vec<Group> = b
                .groups
                .iter()
                .filter_map(|g| {
                    let children: Vec<Item> = g
                        .children
                        .iter()
                        .filter(|r| listed(r))
                        .map(|r| staged_item(r, stages.get(r.id.as_str())))
                        .collect();
                    let keep = match &g.epic {
                        Some(epic) => listed(epic) || !children.is_empty(),
                        None => !children.is_empty(),
                    };
                    keep.then(|| Group {
                        head: g.epic.as_ref().map(item),
                        children,
                    })
                })
                .collect();
            if groups.is_empty() {
                Body::Empty(NO_OPEN)
            } else {
                Body::Filled(groups)
            }
        }
    }
}

/// 未反映の段は最初は開く（見本の id unref の details）。
pub const UNREF_OPEN: bool = true;

/// 未反映の一覧に出す行の数の上限（超える分は残りの数の 1 行）。
pub const UNREF_MAX: usize = 20;

/// 未反映の一覧の口が読めないときの理由。
pub const UNREF_REASON: &str =
    "未反映の一覧の口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 口は読めたが、3 種の全部が「まだ分からない」ときの理由。
pub const UNREF_UNKNOWN: &str =
    "未反映は器（scribe2）の局面の出力から読むが、その出力がまだ無いので、未反映の一覧はまだ分からない";

/// 口は読めたが 3 種の全部が分からず、局面の出力の file は在るのに版が読めない（電文 CaseDoc の unreadable が真）ときの理由。
pub const UNREF_UNREADABLE: &str =
    "未反映は器（scribe2）の局面の出力から読むが、その出力の版が読めないので、未反映の一覧は読めない";

/// 測れて 0 件のときの 1 行。
pub const UNREF_EMPTY: &str = "未反映のものは無い";

/// 未反映の種類ごとの次の 1 手の語の鍵（1 か所の表）。
pub const UNREF_NEXT: [(UnreflectedKind, &str); 3] = [
    (UnreflectedKind::Memo, "nx_promote"),
    (UnreflectedKind::Ruling, "nx_declare"),
    (UnreflectedKind::Utterance, "nx_sort"),
];

/// 未反映の一覧の 1 行（id・題・種類の名・年齢の字・次の 1 手の語の鍵）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrefRow {
    pub id: String,
    pub title: String,
    pub kind: &'static str,
    pub age: String,
    pub next: &'static str,
}

/// 未反映の一覧（先頭から UNREF_MAX までの行・超えた数・分からない種類の名）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrefList {
    pub rows: Vec<UnrefRow>,
    pub more: Option<usize>,
    pub unknown: Vec<&'static str>,
}

/// 次の 1 手の語の鍵（表から引く）。
#[expect(clippy::expect_used, reason = "次の 1 手の表は 3 つの全部を持つ")]
fn next_key(kind: UnreflectedKind) -> &'static str {
    UNREF_NEXT
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, n)| *n)
        .expect("次の 1 手の表は 3 つの全部を持つ")
}

/// 電文の 1 行を一覧の 1 行にする（作った時刻から今 now までの日数の字に）。
fn unref_row(kind: UnreflectedKind, row: &UnreflectedRow, now: EpochSecs) -> UnrefRow {
    UnrefRow {
        id: row.id.clone(),
        title: row.title.clone(),
        kind: kind_name(kind),
        age: age(row.created.map(|c| days_since(c, now))),
        next: next_key(kind),
    }
}

/// 未反映の一覧の口の本文を一覧にする（種類は UnreflectedKind の ALL の順・行は電文の順のまま・
/// 並べ直しと数え直しをしない・年齢は今 now から数える）。
pub fn unref_list(fetched: &Fetched, now: EpochSecs) -> Body<UnrefList> {
    let list = match fetched {
        Fetched::NotRead => return Body::Unmeasured(NOT_READ),
        Fetched::Failed => return Body::Unmeasured(UNREF_REASON),
        Fetched::Body(text) => match wire::decode::<UnreflectedList>(text) {
            Ok(list) => list,
            Err(_) => return Body::Unmeasured(NO_CONTENT),
        },
    };
    let mut rows = Vec::new();
    let mut unknown = Vec::new();
    for kind in UnreflectedKind::ALL {
        let reading = match kind {
            UnreflectedKind::Memo => &list.memos,
            UnreflectedKind::Ruling => &list.rulings,
            UnreflectedKind::Utterance => &list.utterances,
        };
        match reading {
            Reading::Known(items) => rows.extend(items.iter().map(|r| unref_row(kind, r, now))),
            Reading::Unknown => unknown.push(kind_name(kind)),
        }
    }
    if unknown.len() == UnreflectedKind::ALL.len() {
        return Body::Unmeasured(UNREF_UNKNOWN);
    }
    if rows.is_empty() && unknown.is_empty() {
        return Body::Empty(UNREF_EMPTY);
    }
    let more = rows.len().checked_sub(UNREF_MAX).filter(|n| *n > 0);
    rows.truncate(UNREF_MAX);
    Body::Filled(UnrefList {
        rows,
        more,
        unknown,
    })
}

/// 窓の未反映の一覧に局面の出力の読めない版の周を重ねる（要件 FR13）: 3 種とも分からない一覧（UNREF_UNKNOWN）は、
/// 局面の出力が読めない版なら理由を UNREF_UNREADABLE にし、出力がまだ無い周と分ける。ほかの周は一覧のまま。
pub fn unref_shown(list: Body<UnrefList>, cases: &crate::ledgerlist::Phases) -> Body<UnrefList> {
    match (list, cases) {
        (Body::Unmeasured(UNREF_UNKNOWN), crate::ledgerlist::Phases::Unreadable) => {
            Body::Unmeasured(UNREF_UNREADABLE)
        }
        (list, _) => list,
    }
}

/// 未反映の一覧の口の古さの印の 1 行（電文の stale が空でなければ「古い」と印の種類・口が読めないか印が無ければ None）。
pub fn unref_stale(fetched: &Fetched) -> Option<String> {
    let Fetched::Body(text) = fetched else {
        return None;
    };
    let list = wire::decode::<UnreflectedList>(text).ok()?;
    let old = label(crate::ledgerlist::STALE_KEY);
    (!list.stale.is_empty()).then(|| format!("{old}（{}）", list.stale.join("・")))
}

/// 上限を超えた残りの数の 1 行。
pub fn more_line(n: usize) -> String {
    format!("ほか {n} 件")
}

/// 未反映の数の chip の class（0 は class z を足す）。
pub fn unref_chip(count: u32) -> &'static str {
    if count == 0 {
        "chip num unref-n z"
    } else {
        "chip num unref-n"
    }
}

/// 台帳の block（見出しの横に判定の 1 語・中身は台帳 open の一覧）。
#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

#[cfg(target_arch = "wasm32")]
pub use dom::unref_panel;

/// 台帳の block と未反映の段の DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use tsuzuri_contract::case;

    use super::{
        BLOCK, Judge, METRICS_PATH, UNREF_OPEN, UNREF_PATH, Unref, UnrefList, UnrefRow, judge,
        more_line, name_label, stats, unref_chip, unref_list, unref_shown, unref_stale,
    };
    use crate::frame::{self, Mode};
    use crate::ledgerlist::phases;
    use crate::project::{Body, UNKNOWN, fold, section, state_icon, unmeasured};
    use crate::vocab::label;
    use crate::widgets::help::{HelpCtx, hs};

    /// block（見出しの横に指標の口が読めた時だけ判定の 1 語・中身は module ledgerlist の一覧）。
    pub fn view() -> AnyView {
        let fetched = crate::net::read(METRICS_PATH);
        let extra = move || fetched.with(|f| stats(f).ok().map(|s| judge_view(judge(s.judge))));
        section(BLOCK, extra.into_any(), crate::ledgerlist::view())
    }

    /// 未反映の段（帯の抜けの検査の窓の下・見出しに数と分からない種類の chip・中に未反映の一覧・記録の鍵 ledger:unref）。
    /// 数は指標の口の電文の数で、1 種か 2 種が分からなければ数に測れていないの印を添え、指標の口が読めない間は chip を出さない。
    /// 一覧は局面の出力の 4 つの周（まだ無い・読めない版・古い・読めた）を分けて出す（要件 FR13）。
    pub fn unref_panel() -> AnyView {
        let metrics = crate::net::read(METRICS_PATH);
        let unref = crate::net::read(UNREF_PATH);
        let cases = crate::net::read(case::PATH);
        let chips = move || {
            metrics.with(|f| stats(f).ok().map(|s| Unref::of(&s))).map(|u| {
                let partial = u.partial().then(|| state_icon(UNKNOWN));
                let kinds = u
                    .unknown
                    .iter()
                    .map(|k| {
                        view! { <span class="chip num">{name_label(k)}" "{state_icon(UNKNOWN)}</span> }
                    })
                    .collect_view();
                view! {
                    <span class=unref_chip(u.count)>{u.text()}</span>
                    {partial}
                    <span class="lchips">{kinds}</span>
                }
            })
        };
        let ctx = use_context::<HelpCtx>();
        let mode = move || match ctx {
            Some(c) => c.mode.get(),
            None => Mode::from_query(&window().location().search().unwrap_or_default()),
        };
        let list = move || {
            let shown = unref_shown(
                unref_list(&unref.get(), crate::net::now()),
                &cases.with(phases),
            );
            match shown {
                Body::Unmeasured(reason) => unmeasured(reason),
                Body::Empty(line) => view! { <div class="small muted">{line}</div> }.into_any(),
                Body::Filled(l) => unref_rows(&l, mode()),
            }
        };
        let old = move || {
            unref
                .with(unref_stale)
                .map(|t| view! { <div class="small muted">{t}</div> })
        };
        let (open, toggle) = fold("ledger:unref".to_string(), || UNREF_OPEN);
        view! {
            <details class="fold lep" id="unref" prop:open=open on:toggle=toggle>
                <summary>{hs("unref")}{chips}</summary>
                {old}
                {list}
            </details>
        }
        .into_any()
    }

    /// 未反映の一覧の行（見本の unrefHTML・id・題・種類・年齢・次の 1 手の 5 列）と、残りの数と分からない種類の行。
    fn unref_rows(l: &UnrefList, mode: Mode) -> AnyView {
        let table = (!l.rows.is_empty()).then(|| {
            let rows = l.rows.iter().map(|r| unref_row_view(r, mode)).collect_view();
            view! {
                <div class="ulist">
                    <div class="urow hrow">
                        <span>"id"</span>
                        <span>{label("col_title")}</span>
                        {hs("u_kind")}
                        {hs("u_age")}
                        {hs("u_next")}
                    </div>
                    {rows}
                </div>
            }
        });
        let more = l
            .more
            .map(|n| view! { <div class="small muted">{more_line(n)}</div> });
        let unknown = l
            .unknown
            .iter()
            .map(|k| {
                view! {
                    <div class="small muted">{name_label(k)}" "{state_icon(UNKNOWN)}" "{label("gap_unknown")}</div>
                }
            })
            .collect_view();
        view! { {table}{more}{unknown} }.into_any()
    }

    /// 未反映の 1 行（id は節点の頁への link）。
    fn unref_row_view(r: &UnrefRow, mode: Mode) -> AnyView {
        view! {
            <div class="urow">
                <a class="u-id mono" href=frame::node_href(&r.id, mode)>{r.id.clone()}</a>
                <span class="u-t">{r.title.clone()}</span>
                <span>{name_label(r.kind)}</span>
                <span class="num">{r.age.clone()}</span>
                <span class="u-n">{label(r.next)}</span>
            </div>
        }
        .into_any()
    }

    /// 判定の 1 語（見本の judgeHTML）。
    fn judge_view(j: Judge) -> AnyView {
        view! {
            <span class=j.class data-term=j.key tabindex="0">
                <b aria-hidden="true">{j.symbol}</b>
                <span>{label(j.key)}</span>
            </span>
        }
        .into_any()
    }
}
