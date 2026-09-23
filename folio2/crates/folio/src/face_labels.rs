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

/// 機構の live → 名札。
pub fn mechanism_live_label(l: ce::MechanismLive) -> &'static str {
    match l {
        ce::MechanismLive::Now => "いま動く",
        ce::MechanismLive::M0 => "M0 で動く",
        ce::MechanismLive::Delivery0 => "便 0 で動く",
        ce::MechanismLive::M1 => "M1 で動く",
        ce::MechanismLive::Adr => "判断の記録の欄の決まりの後",
    }
}

/// 機構の live → 意味（段の中身は要件書の正本が持つので、段の名と定義の在る節を指すだけ・便 84）。
pub fn mechanism_live_meaning(l: ce::MechanismLive) -> &'static str {
    match l {
        ce::MechanismLive::Now => "今の folio に在る",
        ce::MechanismLive::M0 => "M0 = 要件書の scope の 作る の側に在る段",
        ce::MechanismLive::Delivery0 => "便 0 = 最初の便の段",
        ce::MechanismLive::M1 => "M1 = 要件書の scope_m1 の 作る の側に在る段",
        ce::MechanismLive::Adr => "判断の記録の欄の決まりが定まった後",
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
