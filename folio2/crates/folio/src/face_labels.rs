//! 面の共有の名札（便 87・docs/design/delivery-87.md §1 (a)）。憲法の値域の型（`constitution_enums`）への網羅の
//! 場合分けで持つ名札と、値域に依らない名札の表を持つ。便 87 で `face.rs` から移したもので、字は 1 字も変えていない。
//! 公開の名は `face.rs` から丸ごと再輸出されるので、呼び出し側は `crate::face::…` のまま名指せる。
//! 入口の棚の閉じた一覧（`Shelf` と表 4 つ）は便 109 で `shelf.rs` へ降ろした（ADR-15・層 1 読む）。

use crate::constitution_enums as ce;
use crate::cursor::{R, X};
use crate::rules;

// ── 名札（β・憲法の値域の名札は導出した型への網羅の場合分け・便 50）──
// 憲法の値域（`constitution_enums`・組み立て時に憲法の正本から導出）の名札は、型の値の全部を並べた場合分けで持ち、
// その他を受ける枝を置かない = 憲法の側で値が足されても消えても組み立てが通らない（ADR-11 決定 (4)②）。
// 値域の値の字面（must-not・ask-first など）を鍵にした表は持たない。

/// 段の名札。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tier {
    pub name: &'static str,
    pub en: &'static str,
    pub class: &'static str,
    pub color: &'static str,
    pub meaning: &'static str,
    /// 外すのに要るもの（字面）
    pub remove: &'static str,
    /// 外すのに要るもの（§6 への xref を含む HTML）
    pub remove_html: &'static str,
}

/// 段 → 名札。
pub fn tier_label(t: ce::Tier) -> Tier {
    match t {
        ce::Tier::Always => Tier {
            name: "いつも守る",
            en: "Always",
            class: "tier-always",
            color: "ok",
            meaning: "道具も AI も、毎回これに従う",
            remove: "憲法の改訂（§6: 判断の記録 + 持ち主の承認）",
            remove_html: "憲法の改訂（<a class=\"xref\" href=\"#s6\">§6</a>: 判断の記録 + 持ち主の承認）",
        },
        ce::Tier::AskFirst => Tier {
            name: "確認してから変える",
            en: "Ask-first",
            class: "tier-askfirst",
            color: "warn",
            meaning: "やってよいが、実行前に持ち主へ確認する",
            remove: "その場の持ち主の確認",
            remove_html: "その場の持ち主の確認",
        },
        ce::Tier::Never => Tier {
            name: "絶対にやらない",
            en: "Never",
            class: "tier-never",
            color: "bad",
            meaning: "確認があってもやらない",
            remove: "憲法の改訂（確認では解けない）",
            remove_html: "<a class=\"xref\" href=\"#s6\">憲法の改訂</a>（確認では解けない）",
        },
    }
}

/// 段の表引き（値域に無い値は Err）。
pub fn tier_of(key: &str) -> R<Tier> {
    ce::Tier::from_name(key)
        .map(tier_label)
        .ok_or_else(|| format!("段 の表に無い値「{key}」"))
}

/// 強度 → 規範の語。
pub fn strength_label(s: ce::Strength) -> &'static str {
    match s {
        ce::Strength::Must => "MUST",
        ce::Strength::MustNot => "MUST NOT",
        ce::Strength::Should => "SHOULD",
    }
}

/// 強度 → 意味（要件書の凡例）。
pub fn strength_meaning(s: ce::Strength) -> &'static str {
    match s {
        ce::Strength::Must => "必ず守る",
        ce::Strength::MustNot => "決してしない",
        ce::Strength::Should => "強い推奨（外すなら理由が要る）",
    }
}

/// 強度 → 色の class（prio）。
pub fn strength_prio(s: ce::Strength) -> &'static str {
    match s {
        ce::Strength::Must | ce::Strength::MustNot => "must",
        ce::Strength::Should => "should",
    }
}

/// 型（EARS の pattern）→ 名札。
pub fn pattern_label(p: ce::Pattern) -> &'static str {
    match p {
        ce::Pattern::Ubiquitous => "つねに",
        ce::Pattern::Event => "〜のとき",
        ce::Pattern::State => "〜のあいだ",
        ce::Pattern::Unwanted => "〜になったら",
        ce::Pattern::Optional => "〜ならば",
    }
}

/// 縛る相手 → 名札。
pub fn binds_label(b: ce::Binds) -> &'static str {
    match b {
        ce::Binds::Tool => "道具",
        ce::Binds::Practice => "作法",
        ce::Binds::Both => "両方",
    }
}

/// 機構の種別 → 名札。
pub fn mechanism_kind_label(k: ce::MechanismKind) -> &'static str {
    match k {
        ce::MechanismKind::Reject => "機械が拒む",
        ce::MechanismKind::BuildCheck => "生成時の検査",
        ce::MechanismKind::HumanReview => "人が目で確かめる",
        ce::MechanismKind::None => "なし",
    }
}

/// 機構の live → 名札（now でない 4 値は 1 つの名札・「まだ分からない」の字は使わない・ADR-23 決定 (4)・便 132）。
pub fn mechanism_live_label(l: ce::MechanismLive) -> &'static str {
    match l {
        ce::MechanismLive::Now => "いま動く",
        ce::MechanismLive::M0
        | ce::MechanismLive::Delivery0
        | ce::MechanismLive::M1
        | ce::MechanismLive::Adr => "機構がまだ無い",
    }
}

/// 機構の live → 意味（now でない 4 値は床が判定しないことと憲法が書く実在の予定を正本の字のまま残す・便 132・141）。
pub fn mechanism_live_meaning(l: ce::MechanismLive) -> &'static str {
    match l {
        ce::MechanismLive::Now => "今の folio に在る",
        ce::MechanismLive::M0 => "床は判定しない・憲法が書く実在の予定は M0",
        ce::MechanismLive::Delivery0 => "床は判定しない・憲法が書く実在の予定は delivery-0",
        ce::MechanismLive::M1 => "床は判定しない・憲法が書く実在の予定は M1",
        ce::MechanismLive::Adr => "床は判定しない・憲法が書く実在の予定は adr",
    }
}

/// stage → 名札。
pub fn stage_label(s: ce::Stage) -> &'static str {
    match s {
        ce::Stage::InLoop => "編集時",
        ce::Stage::Post => "事後",
    }
}

/// polarity → 名札。
pub fn polarity_label(p: ce::Polarity) -> &'static str {
    match p {
        ce::Polarity::FailOpen => "開く",
        ce::Polarity::FailClosed => "閉じる",
    }
}

/// 根拠の種別 → 名札。
pub fn rationale_kind_label(k: ce::RationaleKind) -> &'static str {
    match k {
        ce::RationaleKind::V1Incident => "v1 の実害",
        ce::RationaleKind::Scribe2Article => "scribe2 の条",
        ce::RationaleKind::Folio2Ruling => "持ち主の裁定",
    }
}

/// 撤退条件の種別 → 名札（憲法の面・判断の記録の面の名札は `face_adr.rs`）。
pub fn retreat_kind_label(k: ce::RetreatKind) -> &'static str {
    match k {
        ce::RetreatKind::Spike => "試して測る",
        ce::RetreatKind::Measure => "測る",
        ce::RetreatKind::Ruling => "持ち主に問う",
    }
}

/// rules 行の種別 → 名札（便 54・値域は `rules::RuleKind`・網羅の場合分けで値が足されても消えても組み立てが通らない）。
pub fn rule_kind_label(k: rules::RuleKind) -> &'static str {
    match k {
        rules::RuleKind::Deny => "測って落とす",
        rules::RuleKind::BuildCheck => "生成時の検査",
        rules::RuleKind::Detect => "記録のみ",
        rules::RuleKind::HumanReview => "人が守る作法",
    }
}

/// rules 行の状態 → state の chip の class（便 54・値域は `rules::RuleStatus`）。
pub fn rule_status_class(s: rules::RuleStatus) -> &'static str {
    match s {
        rules::RuleStatus::Provisional => "state warn",
        rules::RuleStatus::Frozen => "state ok",
        rules::RuleStatus::Undecided => "state",
    }
}

// ── 名札の表（β・値域に依らない表・表に無い値は導出できない）──

pub const DOC_STATUS: &[(&str, &str)] = &[
    ("effective", "発効・拘束力あり"),
    ("draft", "未承認・拘束力なし"),
];
/// 確かめ方の 1 語の名札（test+inspection は 2 つを「 + 」で繋ぐ・関数 method_label）。
pub const METHOD: &[(&str, &str)] = &[
    ("test", "実際に動かして確かめる（Test）"),
    ("inspection", "目で見て確かめる（Inspection）"),
];
/// 図の色（class は tone-<値>・凡例の sw は sw <値>）。
pub const TONE: &[(&str, &str)] = &[
    ("ok", "tone-ok"),
    ("bad", "tone-bad"),
    ("neutral", "tone-neutral"),
    ("warn", "tone-warn"),
];

/// 入口の状態の名札。
pub const INDEX_STATUS: &[(&str, &str)] = &[("draft", "下書き・拘束力なし"), ("effective", "発効")];

// ── 版の立場（便 138・delivery-138.md §1 (b) の 1）──
// 要件書の欄 meta の version・status・effective_version から、効いている版と版の欄の立場を 1 つの口で決める。
// 新しいか古いかは比べない（字が違えば Pending）。

/// 版の欄が効いている版より先に進んでいるときの札。
pub const PENDING: &str = "起草・承認待ち";
/// 発効なのに効いている版の欄が無いときの札（P-4.2）。
pub const UNKNOWN_EFFECTIVE: &str = "効く版はまだ分からない";

/// 版の立場。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// 状態が draft（効いている版は読まない）。
    Draft,
    /// 効いている版 = 版の欄。
    Effective,
    /// 効いている版（escape 済み）≠ 版の欄。
    Pending(String),
    /// 発効だが効いている版の欄が無い。
    Unknown,
}

/// 欄 meta の版の立場。状態が文書の状態の表に無い・effective_version が scalar でないなら Err。
pub fn standing(m: &X<'_>) -> R<Standing> {
    let status = m.f("status")?;
    status.lookup(DOC_STATUS, "文書の状態")?;
    if status.v.as_str() != Some("effective") {
        return Ok(Standing::Draft);
    }
    Ok(match m.g("effective_version")? {
        None => Standing::Unknown,
        Some(ev) => {
            let ev = ev.text()?;
            if ev == m.f("version")?.text()? {
                Standing::Effective
            } else {
                Standing::Pending(crate::cursor::esc(&ev))
            }
        }
    })
}

impl Standing {
    /// 鮮度の札の版と名札（`version`・`label` は escape 済み）。
    pub fn stamp(&self, version: &str, label: &str) -> (String, String) {
        match self {
            Standing::Pending(ev) => (ev.clone(), format!("{label}・{version} は{PENDING}")),
            Standing::Unknown => (version.to_string(), UNKNOWN_EFFECTIVE.to_string()),
            Standing::Draft | Standing::Effective => (version.to_string(), label.to_string()),
        }
    }

    /// 表紙の状態（`state` は今の字）。
    pub fn cover(&self, version: &str, state: String) -> String {
        match self {
            Standing::Pending(ev) => format!("{ev} が{state}・{version} は{PENDING}"),
            Standing::Unknown => format!("{state}・{UNKNOWN_EFFECTIVE}"),
            Standing::Draft | Standing::Effective => state,
        }
    }

    /// 承認欄のリードの名札。
    pub fn lead(&self, version: &str, label: &str) -> String {
        match self {
            Standing::Pending(_) => format!("{label}（{version} は{PENDING}）"),
            Standing::Unknown => format!("{label}（{UNKNOWN_EFFECTIVE}）"),
            Standing::Draft | Standing::Effective => label.to_string(),
        }
    }

    /// 入口の棚のカードの更新の行に添える字。
    pub fn card(&self) -> String {
        match self {
            Standing::Pending(ev) => format!("（{PENDING}・発効は {ev}）"),
            Standing::Unknown => format!("（{UNKNOWN_EFFECTIVE}）"),
            Standing::Draft | Standing::Effective => String::new(),
        }
    }
}

/// 読む順番の行き先（stops の at）→ その面の anchor か。憲法は s0〜s8・要件書は s1〜s8 と 3 つの図・
/// 判断の記録は `ADR-<1 以上の数>`（記録の面は 1 本 1 枚なので行き先は記録の id そのもの・便 66）。
pub fn stop_anchor(doc: &str, at: &str) -> R<()> {
    let chapter = |from: u8| matches!(at.as_bytes(), [b's', d] if (b'0' + from..=b'8').contains(d));
    let record = || match at.strip_prefix("ADR-") {
        Some(n) => n.bytes().all(|b| b.is_ascii_digit()) && n.parse::<u64>().is_ok_and(|n| n >= 1),
        None => false,
    };
    let ok = match doc {
        "constitution" => chapter(0),
        "srs" => chapter(1) || matches!(at, "fig-context" | "fig-rail" | "fig-verdicts"),
        "adr" => record(),
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(format!(
            "行き先「{doc}#{at}」はその面の節の id（判断の記録は記録の id）に無い"
        ))
    }
}

/// 確かめ方（verify の method）の名札。表に無い値は Err。
pub fn method_label(x: &X<'_>) -> R<String> {
    if x.v.as_str() == Some("test+inspection") {
        return Ok(METHOD
            .iter()
            .map(|(_, l)| *l)
            .collect::<Vec<_>>()
            .join(" + "));
    }
    Ok(x.lookup(METHOD, "確かめ方")?.to_string())
}

#[cfg(test)]
mod face_labels_tests {
    use super::*;
    use crate::yaml;

    fn standing_of(meta: &str) -> R<Standing> {
        let v = yaml::parse_typed(meta).unwrap();
        standing(&X::root(&v, "meta"))
    }

    #[test]
    fn f138_standing_reads_status_version_and_effective_version() {
        // 凍結の針（delivery-138.md §1 (b) の 1 の字を手で写した）
        assert_eq!(PENDING, "起草・承認待ち");
        assert_eq!(UNKNOWN_EFFECTIVE, "効く版はまだ分からない");
        let s = |m: &str| standing_of(m).unwrap();
        assert_eq!(
            s("{version: v0.3, status: effective, effective_version: v0.3}"),
            Standing::Effective
        );
        assert_eq!(
            s("{version: v0.4, status: effective, effective_version: v0.3}"),
            Standing::Pending("v0.3".to_string())
        );
        assert_eq!(s("{version: v0.3, status: effective}"), Standing::Unknown);
        assert_eq!(
            s("{version: v0.4, status: draft, effective_version: v0.3}"),
            Standing::Draft
        );
        assert_eq!(s("{version: v0.4, status: draft}"), Standing::Draft);
        // 効いている版は escape する
        assert_eq!(
            s("{version: v0.4, status: effective, effective_version: \"<v0.3>\"}"),
            Standing::Pending("&lt;v0.3&gt;".to_string())
        );
        // 表の外の状態と一覧の effective_version は導出できない
        let e = standing_of("{version: v0.3, status: retired, effective_version: v0.3}").unwrap_err();
        assert_eq!(e, "meta.status: 文書の状態 の表に無い値「retired」");
        assert!(
            standing_of("{version: v0.3, status: effective, effective_version: [v0.3]}").is_err()
        );
        // 札の字（§1 (b) の 2・3 の表）
        let p = Standing::Pending("v1.41".to_string());
        assert_eq!(
            p.stamp("v1.42", "発効・拘束力あり"),
            (
                "v1.41".to_string(),
                "発効・拘束力あり・v1.42 は起草・承認待ち".to_string()
            )
        );
        assert_eq!(
            p.cover("v1.42", "発効・拘束力あり（承認 2026-09-25）".to_string()),
            "v1.41 が発効・拘束力あり（承認 2026-09-25）・v1.42 は起草・承認待ち"
        );
        assert_eq!(
            p.lead("v1.42", "発効・拘束力あり"),
            "発効・拘束力あり（v1.42 は起草・承認待ち）"
        );
        assert_eq!(p.card(), "（起草・承認待ち・発効は v1.41）");
        let u = Standing::Unknown;
        assert_eq!(
            u.stamp("v0.3", "発効・拘束力あり"),
            ("v0.3".to_string(), "効く版はまだ分からない".to_string())
        );
        assert_eq!(
            u.cover("v0.3", "発効・拘束力あり（承認 2026-09-05）".to_string()),
            "発効・拘束力あり（承認 2026-09-05）・効く版はまだ分からない"
        );
        assert_eq!(
            u.lead("v0.3", "発効・拘束力あり"),
            "発効・拘束力あり（効く版はまだ分からない）"
        );
        assert_eq!(u.card(), "（効く版はまだ分からない）");
        for k in [Standing::Effective, Standing::Draft] {
            assert_eq!(
                k.stamp("v0.3", "発効・拘束力あり"),
                ("v0.3".to_string(), "発効・拘束力あり".to_string())
            );
            assert_eq!(k.cover("v0.3", "x".to_string()), "x");
            assert_eq!(k.lead("v0.3", "x"), "x");
            assert_eq!(k.card(), "");
        }
    }

    #[test]
    fn f141_live_meanings_name_the_plan_and_keep_the_value() {
        // 凍結の針（delivery-141.md §1 (b) の 1 の字を手で写した）
        let plan = "床は判定しない・憲法が書く実在の予定は ";
        assert_eq!(ce::MechanismLive::ALL.len(), 5);
        for l in ce::MechanismLive::ALL {
            let meaning = mechanism_live_meaning(l);
            if l == ce::MechanismLive::Now {
                assert_eq!(meaning, "今の folio に在る");
            } else {
                assert_eq!(meaning, format!("{plan}{}", l.name()), "{}", l.name());
                assert_eq!(mechanism_live_label(l), "機構がまだ無い");
            }
            assert!(!meaning.contains('段'), "{meaning}");
            assert!(!mechanism_live_label(l).contains('段'));
        }
        // 4 値の字を 1 つずつ（値の字は正本の live のまま）
        for (l, v) in [
            (ce::MechanismLive::M0, "M0"),
            (ce::MechanismLive::Delivery0, "delivery-0"),
            (ce::MechanismLive::M1, "M1"),
            (ce::MechanismLive::Adr, "adr"),
        ] {
            assert_eq!(mechanism_live_meaning(l), format!("{plan}{v}"));
        }
    }
}
