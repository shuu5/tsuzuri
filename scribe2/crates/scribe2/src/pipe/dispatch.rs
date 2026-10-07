//! `pipe dispatch` — 審査を通った契約を器が自動で起こす列（設計 docs/design/dispatcher.md §2〜§4・§6）。
//!
//! 起動に要る判定は**器が既に持っている**（台帳の依存・live 便との write-set の交差・受付の余地と host の
//! memory・審査の verdict）。本 module はそれを読み直して並べるだけで、**判定を 2 本目に実装しない**
//! （憲法 C2）: 交差は受付の [`crate::pipe::cli::crossings`]、余地は受付の [`crate::pipe::cli::judge`]、
//! 枠は [`super::admission::has_room`] を**記帳せずに**撃つ。
//!
//! 順序は 1 関数 [`order`] だけが持つ（散文の順序を持たない・設計 §2）: (1) 介入 `first` (2) 台帳の
//! `priority`（P0 → P4）(3) 起票順（id の数字）。
//!
//! 台帳を読めない周は列を空と読まず [`Unmeasured`] で 1 本も起こさない（`0 件`と融合しない・C10・NFR4）。

use super::cli::{live, Denial, Materials};
use super::commute::Verdict;
use super::contract::Contract;
use super::gate::Limits;
use super::health;
use super::land::MAIN_REF;
use super::table::Pointer;
use super::{current, driver_is_dead, driver_ticket, emit_mark, gate_is_open, git_line, pin};
use crate::fleet::store;
use crate::fleet::{Event, Mark, Stage};
use crate::ledger::form::is_question;
use crate::rules::manifest::Manifest;
use crate::seat::ledger;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// 台帳から候補を組む群（設計 §20・`s2-07l.531` の純移動）。
mod candidates;

/// 1 周の群の段（群の逼迫の通知と自動の移動・設計 account-lifecycle.md §19 形 2〜4・§20）。
pub(crate) mod group;

/// 並列の実測の事実と字面（idle の知らせと heartbeat が共用する 1 関数・設計 §26）。
pub(crate) mod facts;

/// 依存を待つ行に受付の判定を予想の base で先に撃つ事前審査（設計 §27・契約表の行 x）。
pub(in crate::pipe) mod precheck;

/// 事前審査の確定を根で束ねる直しの束（設計 §27・契約表の行 y・終端の周の知らせが集合の変化を読む）。
pub(crate) mod bundle;

/// 起こす側の周が受付の断りを契約ごとに `IntakeRefused` へ記帳する書き手（設計 §32・契約表の行 ag）。
mod refused;

/// 床の検査を sha の木で 1 回撃つ段と、待つ側が読む判定の読み手（設計 §34・契約表の行 ai）。
pub mod floor;

/// 落ちた契約の write-set の行の予約と待ちの理由 reserved の値（設計 row-review.md §7・契約表の行 f）。
pub mod reserve;

/// memo の引き金の満ちを判じる行と審査の置き場の形・読み（設計 §40・契約表の行 ao）。
pub mod memo;

/// 上限の許可の見え方（状態の語と `dispatch ls` の効いている許可の行・設計 limit-permit.md §21・契約表の行 e）。
pub(in crate::pipe) mod permits;

/// 起こす便が 0 の周に、引き金の満ちない memo を間隔と本数の内で裏の審査へ渡す選びと撃ち（設計 §42・契約表の行 aq）。
mod memo_triage;

/// memo の審査の裏の process `pipe dispatch memo-lens`（口座を選び lens の段 memo を撃ち、判定を置き場と event に残す・設計 §41・契約表の行 ap）。
pub(in crate::pipe) mod memo_lens;

/// 未反映の裁定の判定と置き場の file・読み手（設計 §38・契約表の行 am）。
pub mod unreflected;

/// driver の継ぎの判定と起こし直す便の選別・構築（設計 §44・契約表の行 as・純移動）。
mod revive;

/// code の索引の組み立ての口 `pipe index build` と状態の読み（床の検査の撃ち方を共用する兄弟・設計 reverse-index.md §4 の行 a2）。
pub mod index_build;

/// 閉じた bead の便の作業木を畳み、亡骸を名指す段（判断の記録 ADR-45 の門 H6）。
mod closed;

/// 起こす面（起こす前の印・子の起動・launch.log・自分の binary の名・`t3-hub.92.10.5` の純移動）。
mod launch;
// flip-check: moved t3-hub.92.10.5

/// 観測と印の面（`dispatch ls` の行・止めの行・印の記帳・設計 §6・`t3-hub.92.10.5` の純移動）。
mod view;
// flip-check: moved t3-hub.92.10.5

use candidates::{build_index, entry_of, indexed, is_input, marks_of, settle, siblings_of, tools};
use launch::start;
pub(in crate::pipe) use launch::spawn_self;
pub(crate) use launch::myself;
pub use view::{line, listing, mark, observe, render, usage, vessel_line, why_of};
pub use revive::{admits_gated, advance, handoff};
use revive::{progress_of, resume, revivals, revive_of};

/// 観測の 1 周（起こさない）を置き場・repo・rules・台帳 client から撃つ口と、その結果（局面の出力の全部の書き直しが列の判定を得る）。
pub use candidates::{observe_round, Observed};
pub(in crate::pipe) use candidates::pointer_of;

/// 台帳の閉じた status の字面（依存が閉じたかの判定が読む）。
const CLOSED: &str = "closed";

/// 列の入力になる status の字面（`in_progress` の bead は既に走っている便が持つ）。
const OPEN: &str = "open";

/// 問いの metadata の effect の字（文書へ写すべき裁定・設計 §38 約束 1）。
const EFFECT_DOCUMENT: &str = "document";

/// 順序を決める依存の種別（`parent-child` は所属であって順序ではない・`.beads/PRIME.md` R2）。
const BLOCKS: &str = "blocks";

/// acceptance が持つ設計 pointer の行の書き出し（受付の `--design` と**同じ字面**・設計 §2）。
const DESIGN_KEY: &str = "design = ";

/// job 1 つが要る memory の rules 行（値は読むだけ・憲法 C5）。
const ROW_JOB_MB: &str = "gate.job_memory_mb";

/// 席と host のために残す memory の rules 行。
const ROW_RESERVE_MB: &str = "host.reserve_memory_mb";

/// 枠が空いていない周の理由の字面。
const SLOT: &str = "slot";

/// 子 process を起こせなかった周の理由の字面。
const SPAWN: &str = "spawn";

/// 起こした事実の印（[`Mark::Launched`]）を書けない・event log を読めず印を測れない周の理由の字面
/// （設計 §17・記帳できない起動を数えない＝測れない側）。
const MARK: &str = "mark";

/// [`WaitReason`] の全 variant の名（宣言順・`enum-slices` が集合完全性を測る）。
pub const WAIT_REASONS: &[&str] =
    &["dependency", "overlap", "admission", "host-busy", "hold", "launched", "settled", "no-design-pointer", "unreflected-ruling", "floor", "reserved", "sibling", "run-cap"];

/// 列に載ったのに起こさない理由（**閉じた型**・設計 §3 の表）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaitReason {
    /// 台帳の依存が閉じていない。
    Dependency {
        /// 閉じていない依存先の bead id（台帳の順）。
        on: Vec<String>,
    },
    /// live な便と write-set が交差する。
    Overlap {
        /// 交差した相手の run id（先頭の 1 本）。
        with: String,
        /// その相手と交差した契約側の file の列（契約が書いた字面・`render` は本数を書く・設計 §26 形 3）。
        files: Vec<String>,
        /// 差の当たりの判じの結末（規則の行が偽の周は `None`＝`render` は本数まで・判断の記録 ADR-60 の決定 (4)）。
        verdict: Option<Verdict>,
    },
    /// 受付（余地・host の memory）を通らない。
    Admission {
        /// 受付が断った名（[`crate::pipe::refuse::Refuse::as_str`] か [`SLOT`] / [`SPAWN`] / [`MARK`]・すべて `'static`）。
        reason: &'static str,
        /// 断りの 1 行の理由（型の断りの先頭の [`crate::pipe::refuse::Refuse::reason`]・型を持たない断りと列自身の語は `None`・
        /// 判定に使わない＝`render` は書かない・断りの記録の detail と局面の出力の欄 `why` に写す）。
        why: Option<String>,
    },
    /// 器の健康の遮断器が「待つ」を返した周（gate と同じ 1 関数 [`health::act`] が [`health::Action::Wait`]・
    /// 設計 §18）。混んだ host に便を起こしても落ちるだけなので、その周は 1 本も起こさない。
    HostBusy,
    /// 介入 `hold` が付いている。
    Hold {
        /// 印を付けた event の ts。
        since: String,
        /// 印の行の理由（detail の `reason:` の後ろ・理由の無い古い印は `None`・判定に使わない＝`render` は書かない）。
        why: Option<String>,
    },
    /// 列が起こした便がまだ受付に届いていない（最新の `launched` の後に同じ bead の `RunCreated` も
    /// `release` も無い・設計 §17）。受付で落ちた便を毎周起こし直さない。
    Launched {
        /// 最新の `launched` の印の ts。
        since: String,
    },
    /// 同じ契約 file の sha で**終端に着いた**便が在る（`Landed` / `Failed` / `Stopped`、審査や gate の
    /// 判定で終端になった段も含む）。契約が改訂されて sha が動けば列に戻る。
    ///
    /// `s2-07l.366` で「審査 FAIL の便」から広げた: 便が終端に着いても bead は台帳で `open` のまま
    /// （器は台帳に書かない・C15）で live な便も無いので、終端が来るたびに同じ契約が起こし直される
    /// （着地から close までの無限再起動）。
    ///
    /// 契約の字が正しいのに器の側の理由で終端に着いた便は、その便の最後の記帳より**後**の `release`
    /// が 1 回だけ列外を外す（[`requeues`] の段だけ・設計 §12・`s2-07l.495`）。
    Settled {
        /// 終端に着いた便の契約 file の sha。
        sha: String,
        /// その便の段（replay が見た最新）。
        stage: Stage,
    },
    /// acceptance に設計 pointer の行が無い。
    NoDesignPointer,
    /// 未反映の裁定（effect が document の閉じた問いの裁定 id で main の先端が引かないもの）を引く・その問いへ blocks の候補が待つ
    /// （値は未反映の列の順で最初の id・設計 §38・行 am）。
    UnreflectedRuling {
        /// 待たせる未反映の裁定 id。
        id: String,
    },
    /// 床の検査が不合格の間、介入 `first` の印の無い候補が待つ（値は今の判定・設計 §35・行 aj）。
    Floor(floor::Judged),
    /// 落ちた契約の行の予約と write-set が交差し、順序でその契約より後ろに並ぶ（設計 row-review.md §7・行 f）。
    Reserved(reserve::Held),
    /// 設計の側の終端に着いた契約 B と同じ設計から出た行で、B の直しが入るまで待つ（値は B の bead id か、期限の行を読めない周は末尾に `/unset`・
    /// 設計 row-review.md §8・行 g）。
    Sibling(String),
    /// 宣言 `run-cap-paths` の dir の下を書く契約で、同じ dir の下を書く live な便と同じ周に起こした便が宣言 `run-cap` の本数に
    /// 達した（値は先頭の 1 本の run id か同じ周に起こした bead id・tsuzuri の判断の記録 ADR-63 の決定 (13)）。
    RunCap(String),
}

impl WaitReason {
    /// 一覧と pin が読む名（kebab・宣言順は [`WAIT_REASONS`]）。
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Dependency { .. } => "dependency",
            Self::Overlap { .. } => "overlap",
            Self::Admission { .. } => "admission",
            Self::HostBusy => "host-busy",
            Self::Hold { .. } => "hold",
            Self::Launched { .. } => "launched",
            Self::Settled { .. } => "settled",
            Self::NoDesignPointer => "no-design-pointer",
            Self::UnreflectedRuling { .. } => "unreflected-ruling",
            Self::Floor(_) => "floor",
            Self::Reserved(_) => "reserved",
            Self::Sibling(_) => "sibling",
            Self::RunCap(_) => "run-cap",
        }
    }

    /// `dispatch ls` の `reason=` に書く字面（名 + 値・値を持たない variant は名だけ）。
    pub fn render(&self) -> String {
        let name = self.as_str();
        match *self {
            Self::Dependency { ref on } => format!("{name}:{}", on.join(",")),
            Self::Overlap { ref with, ref files, verdict: None } => format!("{name}:{with}/{}", files.len()),
            Self::Overlap { ref with, ref files, verdict: Some(found) } => format!("{name}:{with}/{}/{}", files.len(), found.as_str()),
            Self::Admission { reason, .. } => format!("{name}:{reason}"),
            Self::Hold { ref since, .. } | Self::Launched { ref since } => format!("{name}:{since}"),
            Self::UnreflectedRuling { ref id } | Self::Sibling(ref id) | Self::RunCap(ref id) => format!("{name}:{id}"),
            Self::Settled { ref sha, stage } => format!("{name}:{sha}/{}", stage.as_str()),
            Self::Floor(ref found) => format!("{name}:{}", found.rc.map_or_else(|| found.word.as_str().to_owned(), |rc| rc.to_string())),
            Self::Reserved(ref held) => format!("{name}:{}", held.value()),
            Self::HostBusy | Self::NoDesignPointer => name.to_owned(),
        }
    }
}

/// 列の 1 件（台帳の bead + 介入の印 + 起こさない理由）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// bead id。
    pub bead: String,
    /// 台帳の priority（P0 → P4・読めない周は `None`＝最後尾）。
    pub priority: Option<u64>,
    /// 介入の印（`release` は印を外すので `None` に戻る）。
    pub mark: Option<Mark>,
    /// 起こさない理由（`None` = 起こせる）。
    pub reason: Option<WaitReason>,
}

/// 台帳を読めなかった理由（**閉じた型**・`0 件`と融合しない・C10）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unmeasured {
    /// 待ち上限の rules 行が不発効・別の形・不在。
    NoRule,
    /// 台帳の子 process が起動できない・rc 非 0・JSON 不能・待ち上限超過。
    Ledger,
    /// 実装役の口（`--runner`）が無い＝起こせないので列を測らない（見る口は `dispatch ls`）。
    NoRunner,
}

impl Unmeasured {
    /// `[DISPATCH-UNMEASURED reason=…]` に書く字面。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoRule => "no-rule",
            Self::Ledger => "ledger",
            Self::NoRunner => "no-runner",
        }
    }
}

/// `pipe` の subcommand の書き出し（子 process の argv の先頭）。
const PIPE: &str = "pipe";

/// **便の自走を選ぶ flag**（`pipe run` / `pipe resume`・設計 §5「便の自走は起こす側の引数で選ぶ」）。
///
/// 列が起こす便（起こす側・起こし直す側・継ぎの子）には**常に**付ける——道具の pass-through
/// （[`tools`]・値を持つ flag の対の配列・全部か皆無か）とは**別の定数**である。列の判断で起きた便は
/// 自走する、が意味であって、呼び手が道具を渡したかとは関係しない。
pub const DRIVE: &str = "--drive";

/// 段の動き（**閉じた 3 形**・pure・設計 §5「渡す周と渡さない周」）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Advance {
    /// 段が進んだ。
    Forward,
    /// 入口と同じ段のまま。
    Same,
    /// 段が戻った。
    Backward,
}

/// 自分の便を次の driver へ渡すか（**閉じた 5 値**・設計 §5「渡す周と渡さない周」）。
///
/// 渡さない周は理由を名乗る（C10・黙って止まらない）。設計が名指す 3 つの理由に
/// [`Handoff::Unmeasured`] を足してある——段も生死も読めない周を「終端に着いた」に読み替えると、
/// 測れなかった事実が記録から消える。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handoff {
    /// 渡す（前進 ∧ 待ちの段でない ∧ 終端でない）。
    Pass,
    /// 人の手を待つ段に着いた（[`WAITING`]）。
    Waiting,
    /// 終端に着いた。
    Settled,
    /// 段が動かなかった（同じ段・戻った段）。
    NoProgress,
    /// 段か生死を読めなかった（**終端に読み替えない**・C10）。
    Unmeasured,
}

/// [`Handoff`] の全 variant の字面（`drive=` の値・宣言順）。
pub const HANDOFFS: &[&str] = &["pass", "waiting", "settled", "no-progress", "unmeasured"];

impl Handoff {
    /// `drive=` に載る字面。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Waiting => "waiting",
            Self::Settled => "settled",
            Self::NoProgress => "no-progress",
            Self::Unmeasured => "unmeasured",
        }
    }
}

/// 呼び手が渡す自分の便（`--drive` を持つ `pipe run` / `pipe resume` の周だけ・設計 §5）。
///
/// この便は**札の所有者（＝呼び手）が生きていても**起こし直しの候補に入れる——1 段進めて抜ける
/// driver の後を誰も継がないと、便は止まったまま次の契機を待つ（`s2-07l.482` の実測: 起こし直した
/// 便が `Implemented` で止まった）。
pub struct Driving<'a> {
    /// 便 id。
    pub run: &'a str,
    /// **入口で読んだ段**（便を作る `pipe run` は入口に段が無いので `None`＝前進）。
    pub entry: Option<Stage>,
}

/// 起動の構築点（`pipe run` の引数まで組んだ 1 件・[`start`] がそのまま子 process へ渡す）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    /// bead id。
    pub bead: String,
    /// `pipe` に続く引数（`run --design <pointer> --bead … --repo … --state-dir …`）。設計 pointer は
    /// この列の中に在る（同じ値を 2 つの field で持たない）。
    pub argv: Vec<String>,
}

/// 起こし直しの構築点（`pipe resume` の引数まで組んだ 1 件・設計 §5「driver の死亡」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revive {
    /// 便 id。
    pub run: String,
    /// `pipe` に続く引数（`resume --run <id> --repo … --state-dir …` + 列に渡された道具）。
    pub argv: Vec<String>,
}

/// 列の 1 周の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    /// 列の全件（[`order`] の順・`reason` が `None` の件が起こせる便・`dispatch ls` はこれを描く）。
    pub candidates: Vec<Candidate>,
    /// 起こす便の構築点（`candidates` の `reason` が `None` の件と同じ順・同じ本数）。
    pub launches: Vec<Launch>,
    /// 起こし直す便（driver の札の所有者が死んでいる live 便と、関門が開いて driver の居ない待ちの便・run id の順）。
    pub revives: Vec<Revive>,
    /// 台帳を読めなかった周の理由（`Some` なら他の 2 つは空）。
    pub unmeasured: Option<Unmeasured>,
    /// 呼び手の便を次の driver へ渡したか（`--drive` の周だけ `Some`・設計 §5）。
    pub drive: Option<Handoff>,
    /// 終端の周の軸を評価した周の値（起こす側の [`fire`] だけ `Some` になりうる・見る側の [`turn`] は常に `None`・
    /// 設計 consumer-sync.md §15 形 2）。
    pub vessel: Option<crate::hook::vessel::Upstream>,
    /// 局面の出力の全部の書き直し（契機 (a)）の返りのうち `Written`・`Unchanged`・`Coalesced` の外の語（起こす側の [`fire`] だけ
    /// `Some` になりうる・呼び手が stderr の `lifecycle=<語>` の 1 行にする・設計 case-lifecycle.md §12 約束 8）。
    pub lifecycle: Option<&'static str>,
    /// memo の審査の渡しが規則の行を読めず撃たなかった周の語 `no-rule`（起こす側の [`fire`] だけ `Some` になりうる・呼び手が stderr の
    /// `triage=<語>` の 1 行にする・設計 §42 約束 5）。
    pub triage: Option<&'static str>,
    /// 閉じた bead の便の段の 1 行（`closed-runs folded=…`・起こす側の [`fire`] だけ `Some` になりうる・呼び手が stderr へ足す・
    /// 判断の記録 ADR-45 の門 H6）。
    pub closed: Option<String>,
}

/// 列の 1 周に要る材料（すべて永続面から解いたもの・process の記憶を持たない）。
pub struct Input<'a> {
    /// 置き場。
    pub state_dir: &'a Path,
    /// 対象 repo（anchor・base = `HEAD`）。
    pub repo: &'a Path,
    /// 規則の値。
    pub manifest: &'a Manifest,
    /// 台帳 client（`--bd` か [`ledger::DEFAULT_BD`]）。列自身が読むときの値。
    pub bd: &'a str,
    /// `--bd` が**引数で名指されていた**か（起こす便へ渡すのはこちら・既定は渡さない＝便の側が持つ）。
    pub bd_flag: Option<&'a str>,
    /// 規則の写しの path（`--rules`）。起こす便へ**そのまま渡す**（列と便が同じ規則で動く）。
    pub rules: Option<&'a str>,
    /// 審査の lens の口（`--lens`）。渡された周だけ起こす便へそのまま渡す（既定は便の側が持つ）。
    pub lens: Option<&'a str>,
    /// 口座残量の計測の口（`--curl`）。起こす便と起こし直す便へそのまま渡す（既定は便の側が持つ）。
    pub curl: Option<&'a str>,
    /// 実装役の口（`--runner`）。**無ければ 1 本も起こさない**——`pipe run` は `--runner` を要り、
    /// 器は既定を持たない（宣言にも rules 行にも無い・2026-09-19 の実測）。列が勝手な既定を作ると、
    /// 「何を起こすか」が契約の外で決まる（C5 / C1）。
    pub runner: Option<&'a str>,
    /// 呼び手が渡す自分の便（`--drive` の周だけ `Some`・設計 §5「渡す周と渡さない周」）。
    /// **観測の口（[`turn`]）は見ない**——見るだけで便が動くと `dispatch ls` が起こす口になる（§6）。
    pub driving: Option<Driving<'a>>,
    /// 呼び手が自分で段を進めた便の id（`pipe run` / `pipe resume` の周・**flag の有無に依らず** `Some`・
    /// 設計 §15「flag の無い driver は自分の便をこの候補にしない」）。
    ///
    /// [`revivals`] の **PASS の `Gated` の枝だけ**がこの便を候補から外す。flag の無い resume が
    /// Implemented → Gated（PASS）で抜けた直後の自分の 1 周は、札の外れた自分の便を拾って着地まで運んで
    /// しまう＝「1 段だけ」（§5）が破れる。flag の在る driver の自分の便は今までどおり [`Driving`] の
    /// 経路（handoff）が運ぶ。他の枝と観測の口（[`turn`]）は見ない。
    pub driven: Option<&'a str>,
}

/// 列を 1 周する（**判定は器の既存の関数・記帳はしない**）。
///
/// 台帳を読めない周は [`Unmeasured`] を持って返り、1 本も起こさない（fail-closed・設計 §7）。
pub fn turn(input: &Input<'_>) -> Turn {
    measure(input).0
}

/// 1 周の読み（台帳の全件と材料・起こす側の事前審査が同じ 1 回を借りる・設計 §27 形 4）。
struct Read {
    /// 台帳の全件。
    issues: Vec<ledger::Issue>,
    /// 1 周ぶんの repo の材料（読めない周は断り）。
    materials: Result<Materials, Denial>,
    /// 周が読んだ event の列（event log を読めない周は `None`・断りの記帳が同じ 1 回を借りる・設計 §32）。
    events: Option<Vec<Event>>,
    /// 未反映の裁定の判定（起こす側の周が置き場の file に書く・読めない周は `None`・設計 §38）。
    unreflected: Option<unreflected::Judged>,
}

/// 列を 1 周して読みも返す（[`turn`] の本体・台帳を読めない周は読みが `None`）。
fn measure(input: &Input<'_>) -> (Turn, Option<Read>) {
    let Some(timeout) = ledger::timeout_of(input.manifest) else {
        return (unmeasured(Unmeasured::NoRule), None);
    };
    // 台帳の子 process は **`--repo` の中で**撃つ（設計 §14）: 台帳 client は cwd から台帳を解くので、
    // process の cwd を継がせると設計 doc と契約表は `--repo`・台帳は cwd 側という食い違った 1 周になる。
    // `--repo` が dir でない周は spawn が落ちて `Unreadable`＝`unmeasured reason=ledger`（0 件と融合しない・C10）。
    let Ok(issues) = ledger::read_ledger(input.bd, input.repo, timeout) else {
        return (unmeasured(Unmeasured::Ledger), None);
    };
    // 1 周ぶん固定な材料は**ここで 1 回だけ**解く（候補ごとに rules 行と台帳を読み直さない）。
    // event log を読めない周は起こした事実の印を測れない＝`launched` を `None` に持ち、起こせる候補を
    // [`MARK`] で待たせる（読めないを「印が無い」に読み替えない・fail-closed・設計 §17）。
    let read = store::read_all(input.state_dir);
    let unreadable = read.is_err();
    let events = read.unwrap_or_default();
    let marks = marks_of(&events);
    let sha = git_line(input.repo, &["rev-parse", MAIN_REF]);
    let unreflected = sha.as_deref().and_then(|sha| unreflected_of(input, sha, &issues));
    let (mut turn, materials, events) = {
        let ledger = Ledger {
            marks: marks.order,
            why: marks.why,
            launched: (!unreadable).then_some(marks.launched),
            events,
            closed: issues.iter().filter(|issue| issue.status == CLOSED).map(|issue| issue.id.as_str()).collect(),
            materials: Materials::of(input.repo, input.manifest, input.bd).map(|found| indexed(input, found)),
        };
        // 行の予約は周の 1 回の導き（読み済みの台帳と event log を借りる・記帳しない・設計 row-review.md §7）。兄弟の待ちの元の列も同じ導きから組む（§8）。
        let reserved = reserve::derive(input, &issues, &ledger.marks, &ledger.events, crate::seat::state::now_secs());
        let kins = siblings_of(input, &issues, &reserved, &ledger);
        let mut ready: BTreeMap<String, (Option<Pointer>, Contract)> = BTreeMap::new();
        let mut candidates: Vec<Candidate> = Vec::new();
        let floor = sha.as_deref().and_then(|sha| floor::judgement(input.state_dir, sha)).filter(|found| found.word != floor::Word::Pass);
        for issue in issues.iter().filter(|issue| is_input(issue)) {
            let (mut candidate, mut found) = entry_of(input, issue, &issues, &ledger, &kins);
            // 床の検査が不合格の周は、`first` の印・起こした事実・終端の記録のどれも持たない候補を準備の表から外して待たせる（設計 §35 約束 2）。
            let kept = matches!(candidate.reason, Some(WaitReason::Launched { .. } | WaitReason::Settled { .. }));
            if let (Some(judged), false, false) = (&floor, kept, candidate.mark == Some(Mark::First)) {
                (candidate.reason, found) = (Some(WaitReason::Floor(judged.clone())), None);
            }
            // 未反映の裁定を引く・その問いへ blocks の候補は、床の待ちと起こした事実・終端の記録でない限り待たせる（設計 §38 約束 4）。
            let floored = matches!(candidate.reason, Some(WaitReason::Floor(_)));
            let wanted = unreflected.as_ref().and_then(|judged| judged.reason_of(&candidate.bead));
            if let (Some(id), false, false) = (wanted, kept, floored) {
                (candidate.reason, found) = (Some(WaitReason::UnreflectedRuling { id: id.to_owned() }), None);
            }
            if let Some(entry) = found {
                ready.insert(candidate.bead.clone(), entry);
            }
            candidates.push(candidate);
        }
        // **順序は [`order`] の 1 本だけが決める**（生産経路も歯も同じ関数を通る・C2）。
        (settle(input, order(candidates), &ready, &reserved, ledger.materials.as_ref().ok()), ledger.materials, ledger.events)
    };
    if !turn.launches.is_empty() && host_busy(input.manifest) {
        hold_for_host(&mut turn);
    }
    (turn, Some(Read { issues, materials, events: (!unreadable).then_some(events), unreflected }))
}

/// 未反映の裁定の判定（main の先端の sha と追跡された file を読めない周は `None`＝待たせない・file は書かない・設計 §38）。
/// 母集団は閉じた問い（effect = document）、関わりを測るのは閉じていない問い以外の bead。
fn unreflected_of(input: &Input<'_>, sha: &str, issues: &[ledger::Issue]) -> Option<unreflected::Judged> {
    let asked: Vec<unreflected::Question<'_>> = issues
        .iter()
        .filter(|issue| issue.status == CLOSED && issue.effect == EFFECT_DOCUMENT && is_question(issue))
        .map(|issue| unreflected::Question { id: &issue.id, notes: &issue.notes })
        .collect();
    let beads: Vec<unreflected::Bead<'_>> = issues
        .iter()
        .filter(|issue| issue.status != CLOSED && !is_question(issue))
        .map(|issue| unreflected::Bead {
            id: &issue.id,
            text: format!("{}\n{}", issue.acceptance, issue.notes),
            blocks: issue.deps.iter().filter(|dep| dep.kind == BLOCKS).map(|dep| dep.on.as_str()).collect(),
        })
        .collect();
    unreflected::judge(input.repo, input.state_dir, sha, &asked, &beads)
}

/// 器の健康の遮断器が「待つ」を返すか（**gate と同じ 1 関数**・設計 §18・C2）。
///
/// 倍率は gate と同じ 2 行を同じ読み手（[`Limits::of`]）で読み、host は [`health::now`] の 1 回で読む。行を読めない
/// 周と host を測れない周は [`health::act`] のとおり起こす側（`Unmeasured` は撃つ・gate と同じ極性）である。
fn host_busy(manifest: &Manifest) -> bool {
    let Ok(limits) = Limits::of(manifest) else {
        return false;
    };
    health::act(health::now(limits.breaker().per_core)).0 == health::Action::Wait
}

/// 起こせる候補を全部 [`WaitReason::HostBusy`] で待たせる（**その周は 1 本も起こさない**・理由の無い件だけ）。
fn hold_for_host(turn: &mut Turn) {
    turn.launches.clear();
    for candidate in &mut turn.candidates {
        if candidate.reason.is_none() {
            candidate.reason = Some(WaitReason::HostBusy);
        }
    }
}

/// 1 周ぶん固定な台帳側の材料（候補ごとに読み直さない）。
struct Ledger<'a> {
    /// bead ごとの最後の介入の印（`first` / `hold`）。
    marks: BTreeMap<String, (Mark, String)>,
    /// bead ごとの効いている `hold` の理由（[`Marks`] の `why`）。
    why: BTreeMap<String, String>,
    /// 起こしたのにまだ受付に届いていない bead と最新の `launched` の ts（event log を読めない周は `None`＝測れない）。
    launched: Option<BTreeMap<String, String>>,
    /// 置き場の event の並び（`release` が終端の便の最後の記帳より後かを位置で引く・設計 §12）。
    events: Vec<Event>,
    /// 閉じた bead の id（依存が閉じたかを同じ一覧の中で引く）。
    closed: BTreeSet<&'a str>,
    /// 1 周ぶんの repo の材料（**読みは 1 周に 1 回**・設計 §5・読めない周は断りの名を全候補が受ける）。
    materials: Result<Materials, Denial>,
}

/// 列を 1 周して**起こす**（契機の口＝終端の直後・手動の 1 周・印の直後・設計 §5）。
///
/// [`turn`] との違いは**起こすかどうかだけ**である（判定は同じ 1 本・C2）。観測の口（`dispatch ls`）は
/// [`turn`] を撃つ＝**見るだけでは 1 本も起こらない**（§6）。起こせなかった便は起こした数に数えず、
/// 理由つきで待ちに残す（終端の rc は呼び手が変えない・次の契機で拾う・C10）。
pub fn fire(input: &Input<'_>) -> Turn {
    // **群の段は起こす側の 1 周の先頭で走る**（設計 account-lifecycle.md §19 形 2 / 7）: 台帳を読まないので道具
    // （`--runner`）の無い周も走り、見る側（[`turn`]・`dispatch ls`）は撃たない。群 0 の host は 1 語も出さない。移動（§20）も
    // この段の中で便の列の前に走り、段が typed に止まった周（lock の残り・読めない面）も列の rc と行は変えない（§20 形 7）。
    let _ = group::round(input);
    // **床の検査の段は群の段の後・台帳を読む前**（設計 §34 約束 2）: main の先端の sha の宣言に key が在る周だけ 1 回撃つ。
    floor::round(input);
    // **driver の死んだ便を先に起こし直す**（設計 §5）: 起こし直した便は live のままなので列の交差は
    // 動かない。起こす側より先に撃つのは、同じ 1 周の中で「止まっている便」を先に動かすためである。
    // **実装役の口が無い周は列を測らない**（`pipe run` は `--runner` を要り、器は既定を持たない）。
    // 起こせないと分かっている周に台帳の子 process を撃つと、便の終端ごとに読みが 1 回乗る（実測: e2e
    // 全体が 21 秒 → 111 秒）。測っていないので `0 件`とも言わない（C10）——列を見る口は `dispatch ls`。
    if input.runner.is_none() {
        return unmeasured(Unmeasured::NoRunner);
    }
    let (mut turn, read) = measure(input);
    // 未反映の裁定の置き場の file は起こす側の周だけが上書きする（0 件の周も空の列・観測の口は書かない・設計 §38 約束 3）。
    if let Some(found) = read.as_ref().and_then(|found| found.unreflected.as_ref()) {
        unreflected::write(input.state_dir, found);
    }
    // **測れなかった周は 1 つも動かさない**（起こすのも起こし直すのも同じ 1 周の中の手・fail-closed）。
    // 起こし直しは台帳を読まないが、列を 1 周として成立させられない周に片方だけ動かすと、
    // `dispatch=unmeasured` の行が「何もしなかった」を意味しなくなる（C10）。
    if turn.unmeasured.is_some() {
        return turn;
    }
    // **閉じた bead の便の段は列を測れた周の頭で 1 回**（判断の記録 ADR-45 の門 H6）: 同じ周の台帳と event の列を借りる（2 度読まない）。
    let swept = read.as_ref().map(|found| closed::round(input, &found.issues, found.events.as_deref())).unwrap_or_default();
    turn.closed = swept.line();
    // **索引の組み立ては起こす側の周だけが裏で起こす**（設計 reverse-index.md §7 (b)・待たない・観測の口は起こさない）。
    build_index(input, &turn);
    // **関門が開いた待ちの便は、driver の周なら段を前へ進めた周だけ起こす**（設計 §13・[`admits_gated`]）。
    // 段を読めない driver の周は 0 本（測れないを「前進」に読み替えない・fail-closed）。
    let progress = progress_of(input);
    let gated = match progress {
        None => admits_gated(None),
        Some((moved, _)) => moved.is_some_and(|found| admits_gated(Some(found))),
    };
    turn.revives = revivals(input, gated);
    // 亡骸は起こし直さない（閉じた契約を着地させない・止めるかは人か席・判断の記録 ADR-45 の門 H6）。
    turn.revives.retain(|revive| !swept.corpses.contains(&revive.run));
    // **呼び手の便を継ぐ**（設計 §5「1 段進めた driver は終端の 1 周で自分の便を次の driver に渡す」）:
    // 自分の札は生きている（いま握っているのは自分である）ので [`revivals`] は拾わない。渡す周だけ
    // 足し、渡さなかった周は理由を [`Turn::drive`] に残す（C10）。
    turn.drive = progress.map(|(_, handoff)| handoff);
    if let (Some(Handoff::Pass), Some(driving)) = (turn.drive, input.driving.as_ref()) {
        if !turn.revives.iter().any(|revive| revive.run == driving.run) {
            turn.revives.push(revive_of(input, driving.run));
            turn.revives.sort_by(|left, right| left.run.cmp(&right.run));
        }
    }
    turn.revives.retain(|revive| resume(input, revive));
    // **`launches` は印を書けて起こせた分だけ**（設計 §17）: 印を書けない便は [`MARK`]、起こせない便は [`SPAWN`]。
    let failed: BTreeMap<String, &'static str> = turn
        .launches
        .iter()
        .filter_map(|launch| start(input, launch).err().map(|reason| (launch.bead.clone(), reason)))
        .collect();
    turn.launches.retain(|launch| !failed.contains_key(&launch.bead));
    for candidate in &mut turn.candidates {
        if let Some(&reason) = failed.get(&candidate.bead) {
            candidate.reason = Some(WaitReason::Admission { reason, why: None });
        }
    }
    // **受付の断りの記帳は上書きの後・事前審査の前**（設計 §32）: 同じ周が読んだ event の列を借りる（2 度読まない）。
    refused::record(input, &turn.candidates, read.as_ref().and_then(|found| found.events.as_deref()));
    // **事前審査は起こし終えた後**（設計 §27 形 4・起こす便を遅らせない）: 同じ周の台帳と材料を借りる（2 度読まない）。
    if let Some(found) = read.as_ref() {
        precheck::round(input, &turn, &found.issues, found.materials.as_ref().ok());
        // **局面の出力の全部の書き直しは事前審査の後**（設計 case-lifecycle.md §12 約束 8 (a)）: 同じ周の台帳と列の判定を借りる（2 度読まない）。
        turn.lifecycle = candidates::lifecycle_round(input, &turn, found);
        turn.triage = memo_triage::round(input, &turn.launches, found);
    }
    // **終端の周の軸は起こし終えた後に 1 回**（設計 consumer-sync.md §15 形 2）: この周に起こした便・起こし直した便が
    // 在れば live は 0 でない（子の `RunCreated` を待たずに数える＝走り出した便の下で binary を入れ替えない）。
    turn.vessel = crate::hook::vessel::sync(input.state_dir, idle(input, &turn, &swept.corpses));
    turn
}

/// live な便が 0 の周か（`None` = 置き場か便の生死を測れない・live 0 に読み替えない・C10）。
///
/// 生死は起こし直しと同じ 1 本（[`live`]）で読む。この周に起こした便か起こし直した便が在れば `Some(false)`。閉じた bead の亡骸
/// （`corpses`）は数えない（判断の記録 ADR-45 の門 H6）。
fn idle(input: &Input<'_>, turn: &Turn, corpses: &BTreeSet<String>) -> Option<bool> {
    if !turn.launches.is_empty() || !turn.revives.is_empty() {
        return Some(false);
    }
    let state = current(input.state_dir).ok()?;
    let lives: Vec<Option<bool>> =
        state.runs.iter().filter(|(id, _)| !corpses.contains(*id)).map(|(id, run)| live(input.state_dir, id, run.stage)).collect();
    if lives.contains(&Some(true)) {
        return Some(false);
    }
    lives.iter().all(Option::is_some).then_some(true)
}

/// 列の順序を決める 1 関数（**pure**・設計 §2「順序」）: (1) 介入 `first` (2) 台帳の `priority`（P0 → P4）
/// (3) 起票順（id の数字）。同順は id の辞書順（全順序ゆえ待ちは循環しない）。
pub fn order(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    candidates.sort_by_key(key_of);
    candidates
}

/// 並べ替えの鍵（`first` は 0・priority は読めない周を最後尾に倒す・id は数字の列で比べる）。
fn key_of(candidate: &Candidate) -> (u8, u64, Vec<u64>, String) {
    let first = u8::from(candidate.mark != Some(Mark::First));
    let priority = candidate.priority.unwrap_or(u64::MAX);
    (first, priority, digits_of(&candidate.bead), candidate.bead.clone())
}

/// id の数字の列（`s2-07l.345` → `[2, 7, 345]`・起票順の鍵・数字を持たない id は空）。
fn digits_of(bead: &str) -> Vec<u64> {
    bead.split(|glyph: char| !glyph.is_ascii_digit()).filter_map(|run| run.parse().ok()).collect()
}

/// 台帳を読めなかった周の 1 周（1 本も起こさない）。
fn unmeasured(reason: Unmeasured) -> Turn {
    Turn {
        candidates: Vec::new(),
        launches: Vec::new(),
        revives: Vec::new(),
        unmeasured: Some(reason),
        drive: None,
        vessel: None,
        lifecycle: None,
        triage: None,
        closed: None,
    }
}

/// 印の畳み込みの結果（[`marks_of`]）。
struct Marks {
    /// bead ごとの**最後の**介入の印（`first` / `hold`・`release` が外す・設計 §4）。
    order: BTreeMap<String, (Mark, String)>,
    /// bead ごとの最新の `launched` の ts（その後に同じ bead の `RunCreated` も `release` も無いものだけ・設計 §17）。
    /// 介入の印とは**独立の値**で持つ（`hold` と同じ側に畳まない）。
    launched: BTreeMap<String, String>,
    /// bead ごとの効いている `hold` の理由（最後の印が理由つきの `hold` の bead だけ・`first` と `release` が外す）。
    why: BTreeMap<String, String>,
}

/// 値を持たない欄の字面（priority が読めない・理由が無い・印が無い）。
const DASH: &str = "-";

/// 止めの印の行の detail の頭（後ろは `--reason` の理由そのもの・`pipe stop --all` の逐語と同じ形）。
const WHY_PREFIX: &str = "reason:";

// 歯（`mod tests`）だけが引く 5 名（`requeues` は親の doc の link からも引かれるが、link は use に数えられない）。
// 歯の区間の直前に置く（file の最初の行頭 `#[cfg(test)]` を src の本体より後に保つ・設計 §20）。
#[cfg(test)]
use candidates::{launch_of, released_after, requeues, review_unmeasured, section_keyed};
// 歯だけが引く 1 名（群は子の `revive` へ移った・`pub(super)`）。
#[cfg(test)]
use revive::rank;
// 歯だけが引く 1 名（子の `view` の `pub(super)`）。
#[cfg(test)]
use view::hold_line;

#[cfg(test)]
mod tests;
// flip-check: moved t3-hub.92.10.5
