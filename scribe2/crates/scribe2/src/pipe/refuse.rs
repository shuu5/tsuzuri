//! 契約単位の拒否理由と、write-set の交差判定（設計 docs/design/pipeline-conflict.md §2・
//! ADR-0019 §2.1・FR4 / FR11・NFR4）。
//!
//! **主語は「契約 file が読めた後の契約単位の判定」**である。引数の不足（rc 1）と
//! [`super::contract::Contract::load`] の error（rc 2）は従来の口のまま外に在る——入力の型が
//! 違うものを 1 つの enum に集めると、極性一覧の 1 行が 2 種の判定を背負う。
//!
//! [`Unfit`](super::declaration) は **verify 行 1 本**の理由で境界が違うので触らない。
//!
//! 交差の判定は**契約の字面と base の tracked file の一覧**で閉じる: 正規化して項目ごとに突き合わせ、
//! dir 項目は base の file 一覧に**展開してから**数える（設計 contract-source.md §3・新規 file〔`+`〕と
//! dir は交差しない＝dir で書いた snapshot の置き場が配下 1 file の別便と偽の交差を起こさない）。symlink は
//! 解かない。実体が同じ file を別名で持つ 2 契約は入口で見逃す（偽陰性）が、編集時の guard が実体名で塞ぐ
//! （ADR-0009 §2.1 の既知の穴はそのまま）。

use super::closure::ClosureError;
use super::commute::Verdict;
use super::declaration::DECL_FILE;
use super::review::FindingKind;
use super::table::TableError;
use crate::cli_outcome::{RC_BROKEN, RC_REFUSED};
use crate::polarity::{OnFailure, Polarity, Timing};

/// この境界の極性: intake（便を起こす前）で止め、live な便の write-set を読めない周は
/// 断る側へ倒す（読めない store は rc 2・NFR4）。
pub const POLARITY: Polarity = Polarity {
    timing: Timing::InLoop,
    on_failure: OnFailure::FailClosed,
};

/// [`Refuse`] の全 variant の名（**宣言順**・判別子順の pin が読む）。
///
/// payload 付きの enum は `as` で判別子へ写せないので、`&[Refuse]` の const slice
/// （`enum-slices` が測る形）は置けない。名前の slice + [`Refuse::as_str`] の網羅 match を
/// 対にして宣言順を pin する（ADR-0013 §2.1 の形・限界は「末尾の入れ忘れ」を機械が
/// 捕まえないことで、それは `Unfit` / `Guard` と同じ）。
///
/// 読むのは pin の歯だけで、runtime に消費する口は作らない（外形を増やさない）。それでも
/// src に置くのは、base に test 区間だけを写した木で compile error＝flip-check の RED を
/// 構造で作るためである（`xtask::check::shape` と同じ形）。
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "判別子順 pin の歯だけが読む（src 配置で flip-check の RED を作る）")
)]
pub(crate) const REFUSALS: &[&str] = &[
    "not-a-repo",
    "duplicate-run",
    "write-set-overlap",
    "write-set-unreadable",
    "write-set-incomplete",
    "write-set-dir-without-slash",
    "contract-table",
    "write-set-item-unresolved",
    "cap-headroom",
    "name-unresolved",
    "write-set-drift",
    "teeth-place-unresolved",
    "also-names-rust",
    "tests-not-a-teeth-file",
    "fn-undeclared",
    "teeth-outside-write-set",
    "hand-written-contract",
    "same-kind-repeated",
    "finding-unaddressed",
    "promised-field-written",
    "promise-symbol-unresolved",
    "max-live",
    "entrance-not-red",
    "ruling-unresolved",
    "index-building",
    "code-facts",
    "code-facts-unmeasured",
    "run-cap",
    "contract-bead-unreadable",
    "contract-bead-both-forms",
    "contract-bytes-cap",
    "contract-id-taken",
    "contract-open-cap",
];

/// 起動の列の起こす側の 1 周が、受付の理由にこの語で待つ候補が在るとき、HEAD の code の索引の組み立てを裏で起こす契機の語
/// （設計 reverse-index.md §7 (b)・文字列の列で引く＝待ちの理由の型を名指さない・行 d が `index-building` を置き、行 e が
/// `code-facts-unmeasured` を足した）。
pub(crate) const INDEX_BUILD_TRIGGERS: &[&str] = &["index-building", "code-facts-unmeasured"];

/// 欄 `code-facts` を持つ行が測れない周の状態の語のうち、宣言が索引の 2 key を名乗らない周の語（在り処は vessel 宣言の file）。
pub(crate) const STATE_UNDECLARED: &str = "undeclared";

/// 契約 file が読めた後の、契約単位の拒否理由。**新しい理由は variant を 1 つ足す**（憲法 C2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Refuse {
    /// 対象が git repo でない。
    NotARepo {
        /// 対象 repo の path の字面。
        repo: String,
    },
    /// 同じ秒に同じ bead を再 intake した（run id が衝突する）。
    DuplicateRun {
        /// 衝突した run id。
        run: String,
    },
    /// live な便の write-set と交差する（**先頭の 1 組**を持つ・全組は stderr に並ぶ）。
    WriteSetOverlap {
        /// 交差した相手の run id。
        run: String,
        /// 交差した契約側の path（字面は契約が書いたまま）。
        path: String,
        /// 差の当たりの判じの結末（規則の行 `pipe.overlap_commute` が偽の周は `None`・判断の記録 ADR-60 の決定 (4)）。
        verdict: Option<Verdict>,
    },
    /// live な便の write-set を読めない（契約の写しが無い / 壊れている / 判定を読めない）。
    WriteSetUnreadable {
        /// 読めなかった run id。
        run: String,
    },
    /// 契約表の行の `touches` の閉包（[`super::closure`]）が write-set に含まれない（設計 contract-source.md §3・
    /// FR48）。**足りない file を全部**持つ。
    WriteSetIncomplete {
        /// write-set に無い閉包の file（repo 相対・辞書順）。
        missing: Vec<String>,
    },
    /// write-set の項目が末尾 `/` 無しで既存の dir を指す（guard は末尾 `/` 無しを字面一致でしか通さず、runner が
    /// 配下を書けない・設計 contract-source.md §9）。
    WriteSetDirWithoutSlash {
        /// 契約が書いた項目の字面。
        path: String,
    },
    /// 契約表そのものの欠陥（区間・parse・id・section・req・verify・depends・設計 contract-source.md §2）。
    ContractTable(TableError),
    /// write-set の項目が base に解けない（実在する file でも・末尾 `/` の dir でも・`+` 接頭辞の新規 file〔base に
    /// 無い〕でも・`-` 接頭辞の縮む file〔base に在る〕でもない・設計 contract-source.md §3「項目の実在と展開」）。
    WriteSetItemUnresolved {
        /// 契約が書いた項目の字面。
        item: String,
    },
    /// 上限の余地（R-C4-2 / R-C4-1 の値と base の行数の差）を file の見込み（行の `growth` に在ればその値・無ければ
    /// `size` の見積・§46）が超える（設計 contract-source.md §3「上限の余地」・受付だけが撃つ）。core の合計の周は
    /// `file` = `core`（見込みは core に属する file の見込みの和）。
    CapHeadroom {
        /// 余地の足りない file（repo 相対・core の合計は `core`）。
        file: String,
        /// 残っている行数。
        headroom: u64,
        /// 契約の `size` の字面。
        size: String,
        /// その file の見込み（行）。
        estimate: u64,
    },
    /// `title` / `done` / 節の本文の名指し（backtick の中身・path 形 / 型の path 形 / fn 形）が base に解けない
    /// （設計 contract-source.md §3「名指しの実在」・FR54 と同型）。
    NameUnresolved {
        /// 名指しの字面。
        name: String,
        /// どこに書かれていたか（`title` / `done` / `section <N> line <L>`）。
        at: String,
    },
    /// 契約表の行の手書きの write-set が導出値（[`super::closure::derive_write_set`]）と集合として一致しない
    /// （設計 contract-source.md §3「手書きの write-set の扱い」・受付だけが撃つ）。**不足も余分も全部**持つ。
    WriteSetDrift {
        /// 導出値に在って手書きに無い項目。
        missing: Vec<String>,
        /// 手書きに在って導出値に無い項目。
        extra: Vec<String>,
    },
    /// verify の filter 語を含む `#[test]` の fn が base に無く、行の `tests` 欄も無い（歯の置き場を解けない・§3
    /// 「write-set の導出」(ii)）。
    TeethPlaceUnresolved {
        /// 解けなかった filter 語。
        filter: String,
    },
    /// 行の `also` に `.rs` が書かれた（Rust の面は `touches` と `tests` から導く・§3 (v)）。
    AlsoNamesRust {
        /// 書かれていた項目。
        item: String,
    },
    /// 行の `tests` の項目が歯の file（`tests/` 配下か test 区間を持つ `.rs`）でない（§3「限界」）。
    TestsNotATeethFile {
        /// 書かれていた項目。
        item: String,
    },
    /// 行の fn 形の `touches`（`crate::<module>::<snake_ident>`）の fn を宣言する file が base に無い（§18・行 r・
    /// 導出値を空集合に潰さない）。
    FnUndeclared {
        /// module の段の字面（`pipe::cli`・`crate` 直下は `crate`）。
        module: String,
        /// fn の名。
        name: String,
    },
    /// Declared 行（新欄を持たず `write-set` を持つ行）の verify の歯の file（base の `#[test]` の fn 名が filter 語を
    /// 含む file）が行の write-set に無い（§20・行 t・受付だけが撃つ）。**足りない file を全部**持つ。
    TeethOutsideWriteSet {
        /// write-set に無い歯の file（repo 相対・辞書順）と、その file を解いた verify 行の filter 語の対（§41）。
        files: Vec<(String, String)>,
    },
    /// 手書きの契約 file を渡された（`--contract`・契約 (b)・FR54）。契約の正本は設計 doc の行だけで、
    /// 器が base の行から写しを作る＝手で書いた file は**使い方の誤りでなく typed な断り**である。
    HandWrittenContract {
        /// 渡された path の字面。
        path: String,
    },
    /// 同じ bead の直前の便から同じ理由の型（[`FindingKind`]）の審査 FAIL が rules 行 `review.same_kind_stop` の
    /// 本数続き、契約 file と節の本文がともに不変のまま run N+1 を求めた（設計 contract-source.md §23 (2)・受付だけが
    /// 撃つ）。**焼き直しは書き直す**＝契約か節のどちらかが変わっていれば通る。
    SameKindRepeated {
        /// 続いた理由の型。
        kind: FindingKind,
        /// 数えた便 id の列（新しい順）。
        runs: Vec<String>,
        /// rules 行の値（本）。
        stop: u64,
    },
    /// 同じ bead の直前の便の指摘（`review.json` の `kind` と `at`）に対応する差分が今回の材料に無い（§23 (3)・受付だけが
    /// 撃つ）。**対応の無かった項目だけ**（辞書順）を持つ。測れない型（goal-done-contradiction / vacuous-assert / other /
    /// unparsed）と `at` の空な周はこの断りに届かない。
    FindingUnaddressed {
        /// 直前の便の理由の型。
        kind: FindingKind,
        /// 対応の無かった `at` の項目（辞書順）。
        at: Vec<String>,
    },
    /// 約束の行を持つ行（Promised・設計 contract-source.md §33）が、約束の行から器が導く欄（`write-set` / `touches` /
    /// `surfaces` / `tests` / `also` / `creates` / `done`）を手で書いた（受付だけが撃つ）。**書かれた欄を全部**持つ。
    PromisedFieldWritten {
        /// 行 id。
        row: String,
        /// 書かれていた欄の名（欄の宣言順）。
        fields: Vec<String>,
    },
    /// 約束の行の `symbols` の名が base と合わない（§33 項 5・受付だけが撃つ）: `+` 無しの名が base に解けない、か
    /// `+` 付きの名（新設の宣言）が base に既に在る。名は書かれた字面のまま（`+` の有無が極性を名乗る）。
    PromiseSymbolUnresolved {
        /// 親の行 id。
        of: String,
        /// 約束の行の番号。
        n: u64,
        /// 名の字面。
        name: String,
    },
    /// 置き場の live な便の本数が rules 行 `pipe.max_live` の値以上（設計 gate-cost.md §24・受付だけが撃つ・走行中の便は
    /// 止めない）。数え方は交差と同じ live の判定。
    MaxLive {
        /// 数えた live な便の本数。
        live: u64,
        /// rules 行の値（本）。
        cap: u64,
    },
    /// 宣言が `entrance-flip = "deny"` を名乗り、契約の nextest の検証行を base の木で撃った結果に base で緑か測れない行が
    /// 1 本以上在る（設計 pipeline.md §56 形 6・ADR-0059・受付だけが撃つ）。
    EntranceNotRed {
        /// base で緑か測れない行の本数。
        count: u64,
        /// 契約の検証行ごとの 4 値の語（検証行の順）。
        values: Vec<String>,
    },
    /// ruling-check が true の repo で、契約の設計の節か契約表の行が、台帳に解けない裁定 id の引用（解けない問い id の形・解けない
    /// `batch:` / `policy:`・線より後の時刻の形）を持つ（設計 dispatcher.md §37・FR83 / AC53・受付が撃つ）。**置き場ごとに id を全て**
    /// 持つ。台帳か線を読めない周は `unmeasured` に語を持ち（rc 2・測れないを通すに読み替えない）、id の列は空。
    RulingUnresolved {
        /// 設計の節の解けない引用の字面（昇順・重複を除く）。
        section: Vec<String>,
        /// 契約表の行の解けない引用の字面（昇順・重複を除く）。
        row: Vec<String>,
        /// 台帳か線を読めなかった周の語（測れた周は `None`）。
        unmeasured: Option<String>,
    },
    /// touches に型の項目を持つ行の受付が、base の code の索引の組み立てが終わっていない（状態が absent か building）ので
    /// 断る（設計 reverse-index.md §7 (b)・受付が撃つ・起動の列は受付の理由 `index-building` で待たせる）。
    IndexBuilding {
        /// 索引の状態の語（`absent` か `building`）。
        state: String,
    },
    /// 欄 `code-facts` の要素の名乗りの値が、base の code の索引の実測と違う（設計 reverse-index.md §7 (c)・行 e・受付が撃つ・起動の列は
    /// 受付の理由 `code-facts` で待たせる）。1 件は先頭の違う要素で、残りの要素は断りの行の後ろに並ぶ。
    CodeFacts(Box<Difference>),
    /// 欄 `code-facts` を持つ行が、code の索引を測れない（状態が absent・building・failed・状態なし・undeclared・設計 reverse-index.md §7 (c)・
    /// 行 e・名乗った事実を測らずに起動しない）。起動の列は受付の理由 `code-facts-unmeasured` で待たせる。
    CodeFactsUnmeasured {
        /// 名乗りの先頭の要素の字面。
        element: String,
        /// 測れない理由の状態の語（`absent`・`building`・`failed:<失敗の語>`・`none`・`undeclared`）。
        state: String,
    },
    /// 宣言 `run-cap-paths` の dir の下を書く契約で、同じ dir の下を書く live な便が宣言 `run-cap` の本数に達した（tsuzuri の判断の記録
    /// ADR-63 の決定 (13)・受付と列の待ちが同じ判じを読む・走行中の便は止めない）。
    RunCap {
        /// 数えた live な便の先頭の run id。
        run: String,
        /// 宣言の本数。
        cap: u64,
    },
    /// 契約を台帳の bead に置く形（欄 acceptance の `[[contract]]` の行と本文・設計 contract-source.md の切り替えの行 v-bead-intake）の
    /// bead が読めない（台帳に無い・形が無い・写しの字を組めない）。理由は句 1 つ。
    ContractBeadUnreadable {
        /// bead の id。
        bead: String,
        /// 読めない理由の句。
        reason: String,
    },
    /// bead の欄 acceptance が契約表の pointer の行（`design = `）と bead の形の行（`[[contract]]`）を両方持つ。
    ContractBeadBothForms {
        /// bead の id。
        bead: String,
    },
    /// bead の本文か欄 acceptance が rules 行の byte の上限を超える（値ちょうどは通す）。
    ContractBytesCap {
        /// 超えた欄（`description` か `acceptance`）。
        field: &'static str,
        /// 欄の byte の数。
        bytes: u64,
        /// rules 行の値（byte）。
        cap: u64,
    },
    /// bead の契約の行の id が、契約表の行か、ほかの bead の契約の行の id と同じ。
    ContractIdTaken {
        /// 行の id。
        id: String,
        /// 先に持つ側（`<doc>#<id>` か bead の id）。
        by: String,
    },
    /// 開いた契約の bead の本数が rules 行の値を超える。
    ContractOpenCap {
        /// 数えた開いた契約の bead の本数。
        open: u64,
        /// rules 行の値（本）。
        cap: u64,
    },
}

/// 欄 `code-facts` の名乗りと実測が違う要素 1 つ（[`Refuse::CodeFacts`] の中身・断りの型を小さく保つために箱に入れる）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Difference {
    /// 要素の字面（`<列>:<項目>=<値>`）。
    pub element: String,
    /// 名乗りの値の字。
    pub claimed: String,
    /// 実測の値の字。
    pub measured: String,
    /// 実測の site の先頭 3 つ（`<path>:<行>`）。
    pub sites: Vec<String>,
    /// 先頭 3 つの後ろに残る site の件数。
    pub rest: usize,
    /// 母集団の字（`text=<件数>`・測れない周は `text=-`）。
    pub text: String,
}

impl Refuse {
    /// 一覧と pin が読む名（kebab・宣言順は [`REFUSALS`]）。
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            Self::NotARepo { .. } => "not-a-repo",
            Self::DuplicateRun { .. } => "duplicate-run",
            Self::WriteSetOverlap { .. } => "write-set-overlap",
            Self::WriteSetUnreadable { .. } => "write-set-unreadable",
            Self::WriteSetIncomplete { .. } => "write-set-incomplete",
            Self::WriteSetDirWithoutSlash { .. } => "write-set-dir-without-slash",
            Self::ContractTable(_) => "contract-table",
            Self::WriteSetItemUnresolved { .. } => "write-set-item-unresolved",
            Self::CapHeadroom { .. } => "cap-headroom",
            Self::NameUnresolved { .. } => "name-unresolved",
            Self::WriteSetDrift { .. } => "write-set-drift",
            Self::TeethPlaceUnresolved { .. } => "teeth-place-unresolved",
            Self::AlsoNamesRust { .. } => "also-names-rust",
            Self::TestsNotATeethFile { .. } => "tests-not-a-teeth-file",
            Self::FnUndeclared { .. } => "fn-undeclared",
            Self::TeethOutsideWriteSet { .. } => "teeth-outside-write-set",
            Self::HandWrittenContract { .. } => "hand-written-contract",
            Self::SameKindRepeated { .. } => "same-kind-repeated",
            Self::FindingUnaddressed { .. } => "finding-unaddressed",
            Self::PromisedFieldWritten { .. } => "promised-field-written",
            Self::PromiseSymbolUnresolved { .. } => "promise-symbol-unresolved",
            Self::MaxLive { .. } => "max-live",
            Self::EntranceNotRed { .. } => "entrance-not-red",
            Self::RulingUnresolved { .. } => "ruling-unresolved",
            Self::IndexBuilding { .. } => "index-building",
            Self::CodeFacts(_) => "code-facts",
            Self::CodeFactsUnmeasured { .. } => "code-facts-unmeasured",
            Self::RunCap { .. } => "run-cap",
            Self::ContractBeadUnreadable { .. } => "contract-bead-unreadable",
            Self::ContractBeadBothForms { .. } => "contract-bead-both-forms",
            Self::ContractBytesCap { .. } => "contract-bytes-cap",
            Self::ContractIdTaken { .. } => "contract-id-taken",
            Self::ContractOpenCap { .. } => "contract-open-cap",
        }
    }

    /// findings の行の見出し（契約表の欠陥は `contract-table:<理由の名>`・他は [`Self::as_str`]）。
    pub(crate) fn label(&self) -> String {
        match *self {
            Self::ContractTable(ref found) => format!("{}:{}", self.as_str(), found.as_str()),
            _ => self.as_str().to_owned(),
        }
    }

    /// 断る理由の 1 行（run id と path を名乗る）。
    pub(crate) fn reason(&self) -> String {
        match *self {
            Self::NotARepo { ref repo } => format!("{repo} は git repo でない"),
            Self::DuplicateRun { ref run } => format!("run {run} は既に在る（同じ秒の再 intake）"),
            Self::WriteSetOverlap { ref run, ref path, verdict } => {
                let tail = verdict.map_or_else(String::new, |found| format!("・{}", found.as_str()));
                format!("write-set が live な run {run} と交差する（{path}{tail}）")
            }
            Self::WriteSetUnreadable { ref run } => {
                format!("live な run {run} の write-set を読めない")
            }
            Self::WriteSetIncomplete { ref missing } => format!("touches の閉包の file が write-set に無い（{}）", missing.join(", ")),
            Self::WriteSetDirWithoutSlash { ref path } => format!("write-set の {path} は既存の dir を末尾 / 無しで指す（配下を書くなら {path}/）"),
            Self::ContractTable(ref found) => found.reason(),
            Self::WriteSetItemUnresolved { ref item } => {
                format!("write-set の {item} は base に解けない（実在する file・末尾 / の dir・+ 接頭辞の新規 file・- 接頭辞の縮む file のどれでもない）")
            }
            Self::CapHeadroom { ref file, headroom, ref size, estimate } => format!("{file} の上限の余地が {headroom} 行で見込み {estimate} 行（size {size}・growth で上書き可）に足りない"),
            Self::NameUnresolved { ref name, ref at } => format!("名指し {name} が base に無い（{at}）"),
            // 8 理由の字面は導出の側（`ClosureError`）と同じ 1 本（受付が写すだけ・2 面に書かない）。
            Self::WriteSetDrift { ref missing, ref extra } => {
                ClosureError::WriteSetDrift { missing: missing.clone(), extra: extra.clone() }.reason()
            }
            Self::TeethPlaceUnresolved { ref filter } => ClosureError::TeethPlaceUnresolved { filter: filter.clone() }.reason(),
            Self::AlsoNamesRust { ref item } => ClosureError::AlsoNamesRust { item: item.clone() }.reason(),
            Self::TestsNotATeethFile { ref item } => ClosureError::TestsNotATeethFile { item: item.clone() }.reason(),
            Self::FnUndeclared { ref module, ref name } => {
                ClosureError::FnUndeclared { module: module.clone(), name: name.clone() }.reason()
            }
            Self::TeethOutsideWriteSet { ref files } => ClosureError::TeethOutsideWriteSet { files: files.clone() }.reason(),
            Self::HandWrittenContract { ref path } => {
                format!("手書きの契約 file は受け付けない（{path}）＝契約の正本は設計 doc の行で、--design <doc>#<id> を渡す")
            }
            Self::SameKindRepeated { kind, ref runs, stop } => repeated_reason(kind, runs, stop),
            Self::FindingUnaddressed { kind, ref at } => {
                format!("直前の便の審査の指摘（{}）に対応する差分が無い（{}）", kind.as_str(), at.join(", "))
            }
            Self::PromisedFieldWritten { ref row, ref fields } => format!(
                "行 {row} は約束の行を持つ（Promised）ので {} を書けない（write-set / touches / surfaces / tests / also / creates / done は約束の行から導く）",
                fields.join(", ")
            ),
            Self::PromiseSymbolUnresolved { ref of, n, ref name } => match name.strip_prefix(NEW_FILE) {
                Some(fresh) => format!("約束 {of} の n {n} の symbols の {name} は base に既に在る（+ は base に無い新設の名・{fresh} は + を外す）"),
                None => format!("約束 {of} の n {n} の symbols の {name} が base に無い（新設の名なら + を前置する）"),
            },
            // stderr の 1 行は `pipe: max-live live=<n> cap=<c>`（設計 gate-cost.md §24・名と 2 値だけ）。
            Self::MaxLive { live, cap } => format!("{} live={live} cap={cap}", self.as_str()),
            Self::EntranceNotRed { count, ref values } => {
                format!("{} base で緑か測れない契約の検証行が {count} 本在る（deny の名乗り・行ごと {}）", self.as_str(), values.join(","))
            }
            Self::RulingUnresolved { ref section, ref row, ref unmeasured } => ruling_reason(self.as_str(), [section, row], unmeasured.as_deref()),
            Self::IndexBuilding { ref state } => index_building_reason(self.as_str(), state),
            Self::CodeFacts(ref found) => code_facts_reason(self.as_str(), found),
            Self::CodeFactsUnmeasured { ref element, ref state } => unmeasured_reason(self.as_str(), element, state),
            // stderr の 1 行は `pipe: run-cap run=<id> cap=<n>`（max-live と同じ名と 2 値の形）。
            Self::RunCap { ref run, cap } => format!("{} run={run} cap={cap}", self.as_str()),
            // 契約の bead の 5 断りの stderr の 1 行は名と key=値の並び（unreadable だけ末に理由の句）。
            Self::ContractBeadUnreadable { ref bead, ref reason } => format!("{} bead={bead} {reason}", self.as_str()),
            Self::ContractBeadBothForms { ref bead } => format!("{} bead={bead}", self.as_str()),
            Self::ContractBytesCap { field, bytes, cap } => format!("{} field={field} bytes={bytes} cap={cap}", self.as_str()),
            Self::ContractIdTaken { ref id, ref by } => format!("{} id={id} by={by}", self.as_str()),
            Self::ContractOpenCap { open, cap } => format!("{} open={open} cap={cap}", self.as_str()),
        }
    }

    /// 証拠の在り処（**網羅の match 1 本**・設計 docs/design/dispatcher.md §27 形 3）。契約表の欠陥は理由の側が持つ
    /// （[`TableError::evidence`]）。variant が増えた便は compile が止めて、その断りが何に依るかを決めさせる。
    pub(crate) fn evidence(&self) -> Evidence {
        match *self {
            Self::AlsoNamesRust { .. }
            | Self::HandWrittenContract { .. }
            | Self::PromisedFieldWritten { .. }
            | Self::ContractBeadUnreadable { .. }
            | Self::ContractBeadBothForms { .. }
            | Self::ContractBytesCap { .. }
            | Self::ContractIdTaken { .. } => Evidence::Row,
            Self::WriteSetDirWithoutSlash { path: ref file }
            | Self::WriteSetItemUnresolved { item: ref file }
            | Self::CapHeadroom { ref file, .. }
            | Self::TestsNotATeethFile { item: ref file } => Evidence::Files(vec![file.clone()]),
            Self::WriteSetIncomplete { .. }
            | Self::NameUnresolved { .. }
            | Self::WriteSetDrift { .. }
            | Self::TeethPlaceUnresolved { .. }
            | Self::FnUndeclared { .. }
            | Self::TeethOutsideWriteSet { .. }
            | Self::PromiseSymbolUnresolved { .. }
            | Self::CodeFacts(_) => Evidence::Name,
            Self::CodeFactsUnmeasured { ref state, .. } if state == STATE_UNDECLARED => Evidence::Files(vec![DECL_FILE.to_owned()]),
            Self::NotARepo { .. }
            | Self::DuplicateRun { .. }
            | Self::WriteSetOverlap { .. }
            | Self::WriteSetUnreadable { .. }
            | Self::SameKindRepeated { .. }
            | Self::FindingUnaddressed { .. }
            | Self::MaxLive { .. }
            | Self::EntranceNotRed { .. }
            | Self::RulingUnresolved { .. }
            | Self::IndexBuilding { .. }
            | Self::CodeFactsUnmeasured { .. }
            | Self::RunCap { .. }
            | Self::ContractOpenCap { .. } => Evidence::Place,
            Self::ContractTable(ref found) => found.evidence(),
        }
    }

    /// **rc は variant が持つ**。読めない周だけが「壊れた store」の rc 2 で、
    /// 残りは前提違反の rc 1 である（NFR4）。契約表の欠陥は理由の側が持つ（読めない表だけ rc 2）。
    pub(crate) fn rc(&self) -> u8 {
        match *self {
            Self::NotARepo { .. }
            | Self::DuplicateRun { .. }
            | Self::WriteSetOverlap { .. }
            | Self::WriteSetIncomplete { .. }
            | Self::WriteSetDirWithoutSlash { .. }
            | Self::WriteSetItemUnresolved { .. }
            | Self::CapHeadroom { .. }
            | Self::NameUnresolved { .. }
            | Self::WriteSetDrift { .. }
            | Self::TeethPlaceUnresolved { .. }
            | Self::AlsoNamesRust { .. }
            | Self::TestsNotATeethFile { .. }
            | Self::FnUndeclared { .. }
            | Self::TeethOutsideWriteSet { .. }
            | Self::HandWrittenContract { .. }
            | Self::SameKindRepeated { .. }
            | Self::FindingUnaddressed { .. }
            | Self::PromisedFieldWritten { .. }
            | Self::PromiseSymbolUnresolved { .. }
            | Self::MaxLive { .. }
            | Self::EntranceNotRed { .. }
            | Self::IndexBuilding { .. }
            | Self::CodeFacts(_)
            | Self::CodeFactsUnmeasured { .. }
            | Self::RunCap { .. }
            | Self::ContractBeadUnreadable { .. }
            | Self::ContractBeadBothForms { .. }
            | Self::ContractBytesCap { .. }
            | Self::ContractIdTaken { .. }
            | Self::ContractOpenCap { .. } => RC_REFUSED,
            Self::RulingUnresolved { ref unmeasured, .. } if unmeasured.is_some() => RC_BROKEN,
            Self::RulingUnresolved { .. } => RC_REFUSED,
            Self::WriteSetUnreadable { .. } => RC_BROKEN,
            Self::ContractTable(ref found) => found.rc(),
        }
    }
}

mod evidence;
mod overlap;
mod reason;

pub(crate) use evidence::{discern, Certainty, Evidence};
pub(crate) use overlap::{covered, normalize, overlaps, DELETE_FILE, NEW_FILE, PLACE_ONLY_FILE, SHRINK_FILE};
use reason::{code_facts_reason, index_building_reason, repeated_reason, ruling_reason, unmeasured_reason};

#[cfg(test)]
mod tests {
    // flip-check: moved t3-hub.92.10.6
    use super::{Difference, Evidence, FindingKind, Refuse, INDEX_BUILD_TRIGGERS, REFUSALS};
    use crate::cli_outcome::{RC_BROKEN, RC_REFUSED};
    use crate::pipe::table::TableError;

    /// 宣言順に 1 つずつ組んだ全 variant（payload は測らないので固定値）。
    pub(super) fn samples() -> Vec<Refuse> {
        vec![
            Refuse::NotARepo { repo: "/tmp/x".to_owned() },
            Refuse::DuplicateRun { run: "r-1".to_owned() },
            Refuse::WriteSetOverlap { run: "r-1".to_owned(), path: "src/lib.rs".to_owned(), verdict: None },
            Refuse::WriteSetUnreadable { run: "r-1".to_owned() },
            Refuse::WriteSetIncomplete { missing: vec!["src/a.rs".to_owned(), "src/b.rs".to_owned()] },
            Refuse::WriteSetDirWithoutSlash { path: "src".to_owned() },
            Refuse::ContractTable(TableError::SectionMissing { line: 3, section: "9".to_owned() }),
            Refuse::WriteSetItemUnresolved { item: "src/none.rs".to_owned() },
            Refuse::CapHeadroom { file: "src/big.rs".to_owned(), headroom: 7, size: "M".to_owned(), estimate: 30 },
            Refuse::NameUnresolved { name: "Guard::Rules".to_owned(), at: "done".to_owned() },
            Refuse::WriteSetDrift { missing: vec!["src/a.rs".to_owned()], extra: vec!["docs/x.md".to_owned()] },
            Refuse::TeethPlaceUnresolved { filter: "fresh_".to_owned() },
            Refuse::AlsoNamesRust { item: "src/a.rs".to_owned() },
            Refuse::TestsNotATeethFile { item: "src/a.rs".to_owned() },
            Refuse::FnUndeclared { module: "pipe::cli".to_owned(), name: "missing".to_owned() },
            Refuse::TeethOutsideWriteSet {
                files: vec![("src/a.rs".to_owned(), "alpha_".to_owned()), ("tests/b.rs".to_owned(), "beta_".to_owned())],
            },
            Refuse::HandWrittenContract { path: "contract.toml".to_owned() },
            Refuse::SameKindRepeated {
                kind: FindingKind::LiteralMismatch,
                runs: vec!["b-2".to_owned(), "b-1".to_owned()],
                stop: 2,
            },
            Refuse::FindingUnaddressed { kind: FindingKind::TeethOutsideWriteSet, at: vec!["src/a.rs".to_owned()] },
            Refuse::PromisedFieldWritten { row: "ag".to_owned(), fields: vec!["write-set".to_owned(), "done".to_owned()] },
            Refuse::PromiseSymbolUnresolved { of: "ag".to_owned(), n: 2, name: "+Refuse::Fresh".to_owned() },
            Refuse::MaxLive { live: 3, cap: 2 },
            Refuse::EntranceNotRed { count: 1, values: vec!["green-on-base".to_owned(), "absent".to_owned()] },
            Refuse::RulingUnresolved {
                section: vec!["batch:b7".to_owned(), "s2-07l.9:20260930T0000Z-1".to_owned()],
                row: vec!["policy:x".to_owned()],
                unmeasured: None,
            },
            Refuse::IndexBuilding { state: "absent".to_owned() },
            Refuse::CodeFacts(Box::new(Difference {
                element: "refs:crate::pipe::refuse::Refuse=3".to_owned(),
                claimed: "3".to_owned(),
                measured: "5".to_owned(),
                sites: vec!["src/a.rs:1".to_owned(), "src/b.rs:2".to_owned(), "src/c.rs:3".to_owned()],
                rest: 2,
                text: "text=9".to_owned(),
            })),
            Refuse::CodeFactsUnmeasured { element: "refs:crate::pipe::refuse::Refuse=3".to_owned(), state: "absent".to_owned() },
            Refuse::RunCap { run: "r-1".to_owned(), cap: 1 },
            Refuse::ContractBeadUnreadable { bead: "s2-b".to_owned(), reason: "台帳を読めない".to_owned() },
            Refuse::ContractBeadBothForms { bead: "s2-b".to_owned() },
            Refuse::ContractBytesCap { field: "description", bytes: 65537, cap: 65536 },
            Refuse::ContractIdTaken { id: "a".to_owned(), by: "docs/design/toy.md#a".to_owned() },
            Refuse::ContractOpenCap { open: 3, cap: 2 },
        ]
    }

    /// 名の列の末尾の語（末は契約を台帳の bead に置く形の 5 語・run-cap は末から 6 つ目・その前に code-facts の 2 語と索引の組み立て中・
    /// その前に裁定 id の引用・base の木の断り・同時本数・約束の行の語）。
    fn assert_tail_words(names: &[&str]) {
        assert_eq!(names.last().copied(), Some("contract-open-cap"), "末尾は開いた契約の bead の本数の上限");
        assert_eq!(
            names.iter().rev().take(5).rev().copied().collect::<Vec<&str>>(),
            ["contract-bead-unreadable", "contract-bead-both-forms", "contract-bytes-cap", "contract-id-taken", "contract-open-cap"],
            "末の 5 語"
        );
        assert_eq!(names.iter().rev().nth(5).copied(), Some("run-cap"), "run-cap は末から 6 つ目");
        assert_eq!(names.iter().rev().skip(6).take(3).copied().collect::<Vec<&str>>(), ["code-facts-unmeasured", "code-facts", "index-building"]);
        assert_eq!(
            names.iter().rev().skip(9).take(4).copied().collect::<Vec<&str>>(),
            ["ruling-unresolved", "entrance-not-red", "max-live", "promise-symbol-unresolved"]
        );
    }

    // flip-check: retroactive s2-07l.738.37.4
    // flip-check: retroactive s2-07l.736.33.21.5
    // flip-check: retroactive s2-07l.736.33.21.6
    /// 名前の slice は **宣言順**で、`as_str` の網羅 match と 1 対 1 である（ADR-0013 §2.1）。
    #[test]
    fn refuse_names_are_pinned_in_declaration_order() {
        let names: Vec<&str> = samples().iter().map(Refuse::as_str).collect();
        assert_eq!(names, REFUSALS, "名前の slice は宣言順（母集団 {} 値）", REFUSALS.len());
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len(), "名は一意: {names:?}");
        // 末尾 2 つ（本便が足した理由）は交差と読めなさである。
        assert_eq!(names.get(2).copied(), Some("write-set-overlap"), "{names:?}");
        assert_eq!(names.get(3).copied(), Some("write-set-unreadable"), "{names:?}");
        // 約束の行の 2 理由（設計 contract-source.md §33・`s2-07l.512`）が同時本数の上限（gate-cost.md §24・`s2-07l.398`）の
        // 手前に並び、base の木で撃った入口の断り（pipeline.md §56・`s2-07l.557`）がその次で、裁定 id の引用の断り（dispatcher.md
        // §37・行 al）がその次で、索引の組み立て中の断り（reverse-index.md §7 (b)・行 d）がその次で、欄 code-facts の違いと測れない周の
        // 2 断り（§7 (c)・行 e）がその次で、宣言の同時の数の上限（tsuzuri の判断の記録 ADR-63 の決定 (13)）がその次で、契約を台帳の bead に
        // 置く形の 5 断り（行 v-bead-intake）が末尾で、母集団は 33 値。
        assert_eq!(REFUSALS.len(), 33, "母集団 33 値");
        assert_tail_words(&names);
        let entrance = samples().get(22).map(|found| (found.reason(), found.rc()));
        let want = "entrance-not-red base で緑か測れない契約の検証行が 1 本在る（deny の名乗り・行ごと green-on-base,absent）".to_owned();
        assert_eq!(entrance, Some((want, RC_REFUSED)), "本数と行ごとの 4 値を名乗る 1 行・rc 1");
        let cap = samples().get(21).map(|found| (found.reason(), found.rc()));
        assert_eq!(cap, Some(("max-live live=3 cap=2".to_owned(), RC_REFUSED)), "名と live / cap の 2 値だけの 1 行・rc 1");
        let found = samples();
        let written = found.get(19).map(Refuse::reason).unwrap_or_default();
        assert!(written.contains("行 ag") && written.contains("write-set, done"), "行 id と欄を全部名乗る: {written}");
        let fresh = found.get(20).map(Refuse::reason).unwrap_or_default();
        assert!(fresh.contains("約束 ag の n 2") && fresh.contains("+Refuse::Fresh は base に既に在る"), "{fresh}");
        let bare = Refuse::PromiseSymbolUnresolved { of: "ag".to_owned(), n: 1, name: "Nope".to_owned() }.reason();
        assert!(bare.contains("Nope が base に無い"), "+ 無しは不在を名乗る: {bare}");
        assert!(found.iter().skip(19).all(|refuse| refuse.rc() == RC_REFUSED && !refuse.reason().contains('\n')), "rc 1・1 行");
    }

    // flip-check: retroactive s2-07l.738.37.4
    // flip-check: retroactive s2-07l.736.33.21.5
    // flip-check: retroactive s2-07l.736.33.21.6
    /// 在り処は `REFUSALS` の 23 語の母集団で 1 語に 1 つずつ決まる（設計 dispatcher.md §27 形 3・宣言順）: 行の字だけで
    /// 決まる 3 語・名指した file の 4 語（file は payload の字面）・本文の読み手の 7 語・置き場と host の 8 語・契約表の
    /// 欠陥は理由の側（samples の `section-missing` は行）。名の 23 は歯を置いた時の語数で、行 al が置き場と host に 1 語足し（24 語）、
    /// 行 d が置き場と host に索引の組み立て中の 1 語を足し（25 語）、行 e が本文の読み手に 1 語（code-facts）・置き場と host に 1 語
    /// （code-facts-unmeasured・宣言を名乗らない周だけ vessel 宣言の file）を足した（27 語）。tsuzuri の判断の記録 ADR-63 の決定 (13) の
    /// 行 v-one-vessel が置き場と host に宣言の同時の数の上限の 1 語（run-cap）を足し（28 語）、行 v-bead-intake が行の字だけで決まる 4 語
    /// （contract-bead-unreadable・contract-bead-both-forms・contract-bytes-cap・contract-id-taken）と置き場と host に 1 語（contract-open-cap）を足した（33 語）。
    #[test]
    fn pipe_refuse_evidence_is_decided_once_for_each_of_the_23_words() {
        let found: Vec<(&str, String)> = samples().iter().map(|refuse| (refuse.as_str(), refuse.evidence().render())).collect();
        let want = [
            ("not-a-repo", "place"),
            ("duplicate-run", "place"),
            ("write-set-overlap", "place"),
            ("write-set-unreadable", "place"),
            ("write-set-incomplete", "name"),
            ("write-set-dir-without-slash", "files:src"),
            ("contract-table", "row"),
            ("write-set-item-unresolved", "files:src/none.rs"),
            ("cap-headroom", "files:src/big.rs"),
            ("name-unresolved", "name"),
            ("write-set-drift", "name"),
            ("teeth-place-unresolved", "name"),
            ("also-names-rust", "row"),
            ("tests-not-a-teeth-file", "files:src/a.rs"),
            ("fn-undeclared", "name"),
            ("teeth-outside-write-set", "name"),
            ("hand-written-contract", "row"),
            ("same-kind-repeated", "place"),
            ("finding-unaddressed", "place"),
            ("promised-field-written", "row"),
            ("promise-symbol-unresolved", "name"),
            ("max-live", "place"),
            ("entrance-not-red", "place"),
            ("ruling-unresolved", "place"),
            ("index-building", "place"),
            ("code-facts", "name"),
            ("code-facts-unmeasured", "place"),
            ("run-cap", "place"),
            ("contract-bead-unreadable", "row"),
            ("contract-bead-both-forms", "row"),
            ("contract-bytes-cap", "row"),
            ("contract-id-taken", "row"),
            ("contract-open-cap", "place"),
        ];
        let want: Vec<(&str, String)> = want.iter().map(|(name, at)| (*name, (*at).to_owned())).collect();
        assert_eq!(found, want, "母集団 {} 語の在り処", REFUSALS.len());
        assert_eq!(found.len(), REFUSALS.len(), "全語に 1 つ");
        let unreadable = Refuse::ContractTable(crate::pipe::table::TableError::Unreadable { line: 0, reason: "r".to_owned() });
        assert_eq!(unreadable.evidence(), Evidence::Place, "契約表の欠陥は理由の側が決める（読めない表は測れない）");
    }

    /// 索引の組み立てを裏で起こす契機の語の列は断りの file の 1 か所に在り、行 d が置いた `index-building` に行 e が
    /// `code-facts-unmeasured` を足した 2 語で、その語は断りの語（[`REFUSALS`]）の 2 つである。
    #[test]
    fn refuse_index_building_trigger_words_are_the_two_words() {
        assert_eq!(INDEX_BUILD_TRIGGERS, ["index-building", "code-facts-unmeasured"], "契機の語は 2 語");
        assert!(INDEX_BUILD_TRIGGERS.iter().all(|word| REFUSALS.contains(word)), "契機の語は断りの語");
    }

    /// **rc は variant が持つ**: 読めない周だけ rc 2 で、残りは rc 1。理由は run / path を名乗る。
    #[test]
    fn refuse_carries_its_own_rc_and_names_the_run() {
        for found in samples() {
            let rc = found.rc();
            let expected = match found {
                Refuse::WriteSetUnreadable { .. } => RC_BROKEN,
                _ => RC_REFUSED,
            };
            assert_eq!(rc, expected, "{} の rc", found.as_str());
            assert!(!found.reason().is_empty(), "{} は理由を 1 行で名乗る", found.as_str());
        }
        let overlap = Refuse::WriteSetOverlap { run: "r-1".to_owned(), path: "src/lib.rs".to_owned(), verdict: None };
        let line = overlap.reason();
        assert!(line.contains("r-1"), "相手の run id を名乗る: {line}");
        assert!(line.contains("src/lib.rs"), "交差した path を名乗る: {line}");
        assert!(!line.contains('\n'), "理由は 1 行: {line}");
    }
}
