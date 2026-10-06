//! lens の findings の**閉じた category**と母集団（設計 pipeline.md §6 / §17・`s2-07l.188`）。
//!
//! 3 値だけの verdict は「見て 0 件だった」と「見ていない」を弁別できない（監査 2026-09-12
//! 塊 21・`.175`）。本 module は数える観点を閉じた表（[`Findings`]）にし、verdict の 2 key
//! （`findings` / `population`）を読む・書くの 1 か所（[`Tally`]）を持つ。
//!
//! **判断は lens の領分である**。器が持つのは型と件数と母集団だけで、「この抽象は要るか」は
//! ここに 1 行も書かない（機械が持つと観点が増えるたびに判定が器の側へ漏れる）。

/// lens が数える findings の観点（**閉じた enum**・宣言順が verdict の字面の順・憲法 C2）。
///
/// 観点 3 つ（契約適合 / 歯の非空虚 / 憲法）で閉じる。自由文の名を許すと同じ欠陥が別名で数えられ、件数が集計できない
/// （却下案・設計 §17）。過剰設計の 5 種は gate が数えず、契約の審査が数える（`pipe::review` の質・tsuzuri の判断の記録 ADR-63 の決定 (7)）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Findings {
    /// 契約（goal / done / write-set）と diff の食い違い。
    ContractFit,
    /// 歯が空虚（assert が字面の出所を区別しない・測る対象が実装に無い）。
    TeethNonvacuous,
    /// 憲法（生成 file `docs/constitution.md`）の条項に反する面。
    Constitution,
}

impl Findings {
    /// 全 variant（**宣言順**＝`findings` の字面の並び順）。
    pub(super) const ALL: [Self; 3] = [Self::ContractFit, Self::TeethNonvacuous, Self::Constitution];

    /// verdict と雛形に書く字面（**網羅 match**・表に無い名は読む側が断る）。
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ContractFit => "contract-fit",
            Self::TeethNonvacuous => "teeth-nonvacuous",
            Self::Constitution => "constitution",
        }
    }
}

/// 集計を読めなかった理由の **2 値**（設計 gate-cost.md §29・`s2-07l.495`）。
///
/// 「形が読めない」（欠け・重複・表に無い名・件数や母集団の数が数でない）と「読めたが規則で断った」
/// （母集団 0＝lens は読んでいない）は、同じ INCONCLUSIVE でも**撃ち直す側とそうでない側に割れる**
/// ——後者は lens が形どおりに答えた上での主張なので、同じ問いを 2 度出しても向きが変わらない。
/// 理由の字面はどちらも 1 行で、割る前と 1 字も変えない。
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Unread {
    /// 形が読めない（撃ち直す側）。
    Malformed(String),
    /// 読めたが規則で断った（母集団 0・撃ち直さない側）。
    Refused(String),
}

impl Unread {
    /// 理由の字面（どちらの側でも 1 行）。
    pub(super) fn reason(&self) -> &str {
        match self {
            Self::Malformed(reason) | Self::Refused(reason) => reason,
        }
    }
}

/// lens が読んだ母集団（file 数と行数）。
///
/// **0 は「見ていない」**であって「穴が無い」ではない（C10）。0 を持つ [`Population`] は作らない
/// ——読む側で倒すのではなく、型として存在しない側に置く。
#[derive(Debug)]
pub(super) struct Population {
    /// 読んだ file 数。
    files: u64,
    /// 読んだ行数。
    lines: u64,
}

impl Population {
    /// `files:<n>,lines:<n>` を読む。どちらかが欠け / 数でない周は [`Unread::Malformed`]・0 の周は
    /// [`Unread::Refused`]（呼び手が INCONCLUSIVE へ倒す・C11.2・撃ち直すのは前者だけ・設計 §29）。
    fn parse(text: &str) -> Result<Self, Unread> {
        let files = number_of(text, "files").map_err(Unread::Malformed)?;
        let lines = number_of(text, "lines").map_err(Unread::Malformed)?;
        if files == 0 || lines == 0 {
            return Err(Unread::Refused(format!(
                "population が 0（files:{files},lines:{lines}）＝lens は読んでいない"
            )));
        }
        Ok(Self { files, lines })
    }
}

/// verdict の findings の集計（3 category の件数 + 母集団）。**件数は 0 も持つ**。
#[derive(Debug)]
pub(super) struct Tally {
    /// [`Findings::ALL`] と同じ並びの件数。
    counts: Vec<u64>,
    /// lens が読んだ母集団。
    population: Population,
}

impl Tally {
    /// verdict の 2 key を読む。
    ///
    /// **3 category を全部**（0 も）要る＝欠け・重複・表に無い名・母集団の不備は `Err` で、
    /// 呼び手（`super::lens::parse_lens`）が INCONCLUSIVE へ倒す（測り直せる側・FR14）。
    /// `Err` は 2 値（[`Unread`]）——母集団 0 だけが「読めたが規則で断った」側で、残りは「形が読めない」側。
    pub(super) fn parse(findings: &str, population: &str) -> Result<Self, Unread> {
        let counts = counts_of(findings, &Findings::ALL.map(Findings::as_str)).map_err(Unread::Malformed)?;
        Ok(Self { counts, population: Population::parse(population)? })
    }

    /// `verdict.json` に書く `findings` の字面（**宣言順・3 category を 0 も含めて全部**）。
    ///
    /// lens が並べ替えて出した周も記帳の順は 1 つである（集計する側が順を持つ・C2）。
    pub(super) fn findings_field(&self) -> String {
        Findings::ALL
            .iter()
            .zip(&self.counts)
            .map(|(found, count)| format!("{}:{count}", found.as_str()))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// `verdict.json` に書く `population` の字面。
    pub(super) fn population_field(&self) -> String {
        format!("files:{},lines:{}", self.population.files, self.population.lines)
    }

    /// 判定と数の食い違いの材料（tsuzuri の判断の記録 ADR-63 の決定 (6)）。3 観点（数えたら PASS を返さない 3 つ・雛形 `lens.txt` の
    /// 判定の決め方の 1 文と同じ）のどれかを 1 以上と数えた周だけ、[`Self::findings_field`] の字面を返す。
    pub(super) fn mismatch(&self) -> Option<String> {
        self.counts.iter().any(|count| *count > 0).then(|| self.findings_field())
    }
}

/// 判定と数の食い違いを FAIL に読んだ周の理由の型（`verdict.json` の `kind`・memo の口が出所の kind に写す）。
pub(super) const MISMATCH_KIND: &str = "verdict-count-mismatch";

/// `<名>:<件数>` を `,` で継いだ字を、`names` の名をちょうど 1 度ずつ持つ周だけ `names` の順の件数に読む（lens の並びは問わない・
/// gate の findings と契約の審査の質〔`pipe::review`〕の読みの 1 本）。欠け・重複・表に無い名・件数が数でない周は理由の 1 行。
pub(crate) fn counts_of(text: &str, names: &[&str]) -> Result<Vec<u64>, String> {
    let pairs = text.split(',').map(count_pair).collect::<Result<Vec<_>, String>>()?;
    if let Some((name, _)) = pairs.iter().find(|(name, _)| !names.contains(name)) {
        return Err(format!("findings の category {name} は表に無い"));
    }
    let mut counts = Vec::with_capacity(names.len());
    for name in names {
        let mut hits = pairs.iter().filter(|(found, _)| found == name);
        let (_, count) = hits.next().ok_or_else(|| format!("findings に {name} が無い"))?;
        if hits.next().is_some() {
            return Err(format!("findings の {name} が 2 度出た"));
        }
        counts.push(*count);
    }
    Ok(counts)
}

/// `findings` の 1 項目（`<category>:<件数>`）を読む。件数は 10 進の非負整数だけ。
fn count_pair(item: &str) -> Result<(&str, u64), String> {
    let split = item.trim().split_once(':');
    let (name, count) = split.ok_or_else(|| format!("findings の項目 {item} が <category>:<件数> でない"))?;
    let parsed = count.parse().map_err(|_| format!("findings の件数 {count} が数でない"))?;
    Ok((name, parsed))
}

/// `population` の 1 つの数（`<名>:<n>`）を引く。
fn number_of(text: &str, name: &str) -> Result<u64, String> {
    let found = text.split(',').find_map(|item| item.trim().strip_prefix(name)?.strip_prefix(':'));
    let value = found.ok_or_else(|| format!("population に {name} が無い"))?;
    value.parse().map_err(|_| format!("population の {name} が数でない（{value}）"))
}

#[cfg(test)]
mod tests {
    use super::{Findings, Tally, Unread};

    /// 3 category の字面（宣言順）。
    const NAMES: [&str; 3] = ["contract-fit", "teeth-nonvacuous", "constitution"];

    /// 母集団の在る周の `population` の字面。
    const READ: &str = "files:7,lines:42";

    /// 観点は **3 つで閉じ**、[`Findings::ALL`] は宣言順そのままである（順が動けば verdict の
    /// 字面が動く＝集計の並びは表 1 つが持つ）。
    #[test]
    fn pipe_gate_findings_all_lists_the_three_categories_in_declaration_order() {
        let shown: Vec<&str> = Findings::ALL.iter().map(|found| found.as_str()).collect();
        assert_eq!(shown, NAMES, "3 variant の宣言順");
    }

    /// 読んだ 2 key は**宣言順の字面へ正規化**される（lens が並べ替えても記帳の順は 1 つ・0 も書く）。
    #[test]
    fn pipe_gate_findings_tally_renders_every_category_in_declaration_order() {
        let shuffled = "constitution:2,contract-fit:1,teeth-nonvacuous:0";
        let tally = Tally::parse(shuffled, READ).expect("3 category と母集団が揃えば読める");
        assert_eq!(tally.findings_field(), "contract-fit:1,teeth-nonvacuous:0,constitution:2", "宣言順で 3 つとも（0 件も 0 と）書く");
        assert_eq!(tally.population_field(), READ, "母集団はそのまま載る");
    }

    /// 欠け・重複・表に無い名・母集団の不備は**読めない**（呼び手が INCONCLUSIVE へ倒す・C10）。
    ///
    /// 読めなさは 2 値に割れる（設計 gate-cost.md §29）: **母集団 0 だけ**が「読めたが規則で断った」
    /// （[`Unread::Refused`]・撃ち直さない側）で、残りは全部「形が読めない」（[`Unread::Malformed`]）。
    #[test]
    fn pipe_gate_findings_tally_refuses_incomplete_categories_and_zero_population() {
        let all = "contract-fit:0,teeth-nonvacuous:0,constitution:0";
        let short = "contract-fit:0,teeth-nonvacuous:0";
        let unknown = format!("{all},typo:1");
        let not_a_number = all.replace("constitution:0", "constitution:x");
        let cases = [
            (short, READ, "constitution", true),
            ("contract-fit:0,contract-fit:0", READ, "2 度", true),
            (unknown.as_str(), READ, "表に無い", true),
            ("contract-fit", READ, "<category>:<件数> でない", true),
            (not_a_number.as_str(), READ, "数でない", true),
            (all, "files:0,lines:42", "population が 0", false),
            (all, "files:7,lines:0", "population が 0", false),
            (all, "lines:42", "population に files が無い", true),
            (all, "files:7", "population に lines が無い", true),
            (all, "files:~7,lines:42", "population の files が数でない（~7）", true),
        ];
        for (findings, population, reason, malformed) in cases {
            let read = Tally::parse(findings, population);
            let err = read.expect_err(&format!("読めてはならない: {findings} / {population}"));
            assert!(err.reason().contains(reason), "理由に {reason} が要る: {err:?}");
            assert_eq!(
                matches!(err, Unread::Malformed(_)),
                malformed,
                "形が読めない側は母集団 0 以外の全部: {findings} / {population} → {err:?}"
            );
        }
        assert!(Tally::parse(all, READ).is_ok(), "対: 3 つ揃い母集団が 0 でなければ読める");
    }

    /// 食い違いの材料は 3 観点のどれか 1 つを 1 以上と数えた周だけ 3 観点の件数を宣言順で返し、3 観点が 0 なら返さない
    /// （tsuzuri の判断の記録 ADR-63 の決定 (6)）。
    #[test]
    fn vgfind_tally_names_the_three_fit_counts_only_when_one_is_counted() {
        let cases = [
            ("contract-fit:2,teeth-nonvacuous:0,constitution:0", Some("contract-fit:2,teeth-nonvacuous:0,constitution:0")),
            ("teeth-nonvacuous:1,contract-fit:0,constitution:0", Some("contract-fit:0,teeth-nonvacuous:1,constitution:0")),
            ("contract-fit:0,teeth-nonvacuous:0,constitution:3", Some("contract-fit:0,teeth-nonvacuous:0,constitution:3")),
            ("contract-fit:0,teeth-nonvacuous:0,constitution:0", None),
        ];
        for (fit, want) in cases {
            let tally = Tally::parse(fit, READ).expect("3 category と母集団が揃えば読める");
            assert_eq!(tally.mismatch().as_deref(), want, "{fit}");
        }
    }

    /// gate は質を数えない（tsuzuri の判断の記録 ADR-63 の決定 (7)・乙'）: 3 観点に質の 5 観点のどれか 1 つを 0 件で足した findings は、
    /// その名を表に無い名として読めず（形の誤りの側）、3 観点だけの findings は読める。
    #[test]
    fn vrqual_gate_findings_refuse_every_quality_name() {
        let fit = "contract-fit:0,teeth-nonvacuous:0,constitution:0";
        for name in ["delete", "stdlib", "native", "yagni", "shrink"] {
            let err = Tally::parse(&format!("{fit},{name}:0"), READ).expect_err(name);
            assert_eq!(err, Unread::Malformed(format!("findings の category {name} は表に無い")), "{name}");
        }
        assert!(Tally::parse(fit, READ).is_ok(), "対: 3 観点だけなら読める");
    }
}
