//! `folio check` の判断の記録（`adr/`）の欄の決まりの検査（便 5・docs/design/delivery-5.md §1）。
//! day-1 の床 `scripts/check_draft.py` の adr の節のうち、判断の記録の file と欄の決まりの file だけで閉じる検査を同じ式で写す。
//! 憲法・rules・anchor と突き合わせる検査（amends の対象の実在・amended_by との双方向・対話面の行の実在・
//! 判断の記録の id の参照・本文の英字語）は便 6 の `link.rs` で、読んだ欄の決まりと判断の記録（`Adr`）と床の定数（`floor_strs`）を渡す。
//! 凍結 anchor の列そのものは便 7。
//! 欄の決まりの閾値・値域・置き場は床の定数（`FLOOR`）で持ち、adr/schema.yaml の schema 節はその写し（N-3.1）。
//! 便 45（ADR-9）から schema 節は生成区間で、`folio schema --write` が `FLOOR` から導出する＝説明の注（`_note` で終わる欄）も
//! `FLOOR` の側に file の順と字面のまま持つ（床の突き合わせは注を読まない）。床の機械（木の型・突き合わせ）は `schema.rs`。
//! パターンの文字列は定数として字面で持つだけで、形の判定は字の走査で行う（正規表現は使わない）。
//! 任意の図の節（figures・便 33）は設計ノートの図の節と同じ形（欄の集合・型は部品目録の一覧・spec は表・refs は
//! basis と同じ id の形・図の id は 1 本の記録の中で一意）を見る。行き先の解決は床では数えない（面が「まだ分からない」で表す）。

use std::fs;
use std::path::Path;

use crate::catalog::FigureType;
use crate::constitution_enums::RetreatKind;
use crate::floor::{Floor, floor_diff, keys_floor, strip_notes};
use crate::verdict::Report;
use crate::yaml::{self, Node};

/// 欄の集合（required / optional）。
pub(crate) struct Keys {
    required: &'static [&'static str],
    optional: &'static [&'static str],
}

const ID_PATTERN: &str = "^ADR-[1-9][0-9]*$";
const DATE_FORMAT: &str = r"^\d{4}-\d{2}-\d{2}$";
const RULING_PATTERN: &str = r"[a-z]\d-[0-9a-z]+(\.\d+)?";
const OWNER: &str = "持ち主";
/// 帰結の欄（便 92・ADR-13 決定 (3-b)（ウ））。その判断が発効で生んだものの id の一覧で、根拠の欄 basis とは別に持つ。
const PRODUCED: &str = "produced";
/// 改訂の欄（便 101・ADR-13 決定 (14)）。発効した判断が生きたまま、その決定の範囲を別の判断が変えた対の一覧で、改訂する側だけが持つ。
const REVISES: &str = "revises";
const RECORD: Keys = Keys {
    required: &[
        "id", "title", "status", "date", "context", "decision", "options", "basis", "retreat",
        "plain",
    ],
    optional: &[
        "amends",
        REVISES,
        "grill",
        "approval",
        "consequences",
        PRODUCED,
        "supersedes",
        "superseded_by",
        "note",
        "figures",
    ],
};
const NON_EMPTY: &[&str] = &["title", "context", "decision", "plain"];
const STATUS: &[&str] = &["proposed", "accepted", "retired"];
const VERDICT: &[&str] = &["adopted", "rejected"];
const REVISE_KIND: &[&str] = &["narrow", "widen"];
/// 撤退条件の種類 = 憲法の値域 schema.enums.retreat_kind から組み立て時に導出した名の列（便 49・手書きの写しは持たない）。
const RETREAT_KIND: &[&str] = &RetreatKind::NAMES;
const APPROVER: &[&str] = &["持ち主", "planner 席", "orchestrator 席"];
const SURFACE: &[&str] = &["R-8"];
const EFFECTIVE_STATUS: &[&str] = &["accepted", "retired"];
const OPTION: Keys = Keys {
    required: &["id", "name", "text", "verdict", "reason"],
    optional: &[],
};
const OPTIONS_MIN: usize = 2;
const OPTIONS_ADOPTED: usize = 1;
const RETREAT: Keys = Keys {
    required: &["kind", "condition"],
    optional: &[],
};
const AMENDS_ENTRY: Keys = Keys {
    required: &["target", "field", "version", "previous_text", "new_text"],
    optional: &[],
};
const REVISES_ENTRY: Keys = Keys {
    required: &["target", "decision", "kind", "summary"],
    optional: &[],
};
const GRILL: Keys = Keys {
    required: &["when", "who", "where", "summary"],
    optional: &[],
};
const APPROVAL: Keys = Keys {
    required: &["who", "date", "ruling", "verbatim", "surface"],
    optional: &[],
};
pub(crate) const AMENDED_BY_ENTRY: Keys = Keys {
    required: &[
        "adr",
        "date",
        "approved_by",
        "ruling",
        "previous_text",
        "rationale",
    ],
    optional: &[],
};
/// 図の節の行の欄（設計ノートの欄の決まりの figures.entry と同じ形・便 33）。
const FIGURE_ENTRY: Keys = Keys {
    required: &["id", "type", "caption", "spec"],
    optional: &["refs", "note"],
};
/// 図の型の一覧の置き場（部品目録の図の型・写しの字面）。
const FIGURE_TYPE_ENUM_REF: &str = "design-intent/preview/parts.json figure_type_enum";

/// 床の定数（値は day-1 の床の FLOOR と同じ）。`_note` で終わる欄は人が読む説明の注（便 45・ADR-9）で、
/// adr/schema.yaml の生成区間に在る順と字面のまま持つ＝床の突き合わせ（`floor_diff`）は読まず、`folio schema` の導出だけが使う。
pub(crate) const FLOOR: Floor = Floor::Map(&[
    (
        "floor_note",
        Floor::Val(
            "以下の欄（*_note を除く）は床の定数の写し。型も値も違わないこと（2.0 と 2・true と 1 も別）＝1 字でも食い違えば床が落とす。変えるときは床の実装の定数を直し、folio schema --write で生成区間を書き直す（取り込みの審査で読む）",
        ),
    ),
    ("id_pattern", Floor::Val(ID_PATTERN)),
    (
        "id_pattern_note",
        Floor::Val(
            "folio2 の他の id（P-1・R-7・FR1）と同じくゼロ詰めしない。4 桁（ADR-0047）は前の版（v1）か scribe2 の記録＝外部の参照で、床は内部 id として数えない。file 名は id と同じ",
        ),
    ),
    ("date_format", Floor::Val(DATE_FORMAT)),
    (
        "date_format_note",
        Floor::Val("date・approval.date・grill.when・amended_by.date は年-月-日"),
    ),
    ("ruling_pattern", Floor::Val(RULING_PATTERN)),
    (
        "ruling_pattern_note",
        Floor::Val(
            "裁定 id は台帳の id（f2-648.2 / s2-07l.149 の形）を 1 つ以上含む。実在と本物かは人が台帳と突き合わせる（P-12.2）",
        ),
    ),
    ("owner", Floor::Val(OWNER)),
    (
        "owner_note",
        Floor::Val(
            "条文を改訂する発効した判断の承認者はこの値（N-4）。orchestrator 席（2026-09-19 までの名は planner 席）は条文を改訂しない判断だけを承認できる",
        ),
    ),
    ("required", Floor::Strs(RECORD.required)),
    ("optional", Floor::Strs(RECORD.optional)),
    ("non_empty", Floor::Strs(NON_EMPTY)),
    (
        "non_empty_note",
        Floor::Val(
            "ほかに床が非空を課す欄 = basis（空の一覧を落とす）・options の name / text / reason・retreat.condition・grill の who / where / summary・approval の ruling / verbatim・amends の field / version / previous_text / new_text・憲法の各条の amended_by の approved_by / ruling / previous_text / rationale・anchor の承認一覧の who / ruling / verbatim",
        ),
    ),
    (
        "enums",
        Floor::Map(&[
            ("status", Floor::Strs(STATUS)),
            ("verdict", Floor::Strs(VERDICT)),
            ("retreat_kind", Floor::Strs(RETREAT_KIND)),
            ("approver", Floor::Strs(APPROVER)),
            ("surface", Floor::Strs(SURFACE)),
            ("revise_kind", Floor::Strs(REVISE_KIND)),
        ]),
    ),
    (
        "enums_note",
        Floor::Val(
            "retreat_kind は憲法 schema.enums.retreat_kind と同じ（食い違えば落ちる）。surface は対話面を rules 行の id で指す（P-5.2・P-12.1 が名指す R-8 = 持ち主と orchestrator 席の対話面）",
        ),
    ),
    (
        "status_note",
        Floor::Map(&[
            (
                "proposed",
                Floor::Val("提案中・拘束力なし（承認欄は空でよい）"),
            ),
            (
                "accepted",
                Floor::Val(
                    "発効（承認欄が必須。条文を改訂する判断なら承認者は 持ち主・grill 必須）",
                ),
            ),
            (
                "retired",
                Floor::Val(
                    "廃止（番号は空けたまま・superseded_by に後継の id が必須・P-7.2）。承認欄を持つ retired は「発効していた」判断として amended_by から参照し続けられる。承認者の要求は retired にも掛かる。後継の列（superseded_by をたどる）は accepted に到達すること（輪・未発効の後継は落ちる）",
                ),
            ),
        ]),
    ),
    ("effective_status", Floor::Strs(EFFECTIVE_STATUS)),
    (
        "effective_status_note",
        Floor::Val(
            "「発効した判断」= この status かつ承認欄あり。amends が効くのは発効した判断だけ",
        ),
    ),
    // 注の鍵は憲法の値域から導出した型の名（便 50 (a2)・値の字面を手書きしない）。注の文と順は adr/schema.yaml の生成区間のまま
    (
        "retreat_kind_note",
        Floor::Map(&[
            (
                RetreatKind::Spike.name(),
                Floor::Val("小さな試し（spike）の結果が条件に当たったら捨てる"),
            ),
            (
                RetreatKind::Measure.name(),
                Floor::Val("数えた値（回数・件数・byte）が条件に達したら捨てる"),
            ),
            (
                RetreatKind::Ruling.name(),
                Floor::Val("持ち主の裁定で捨てる（条件は「何を持ち主に問うか」を書く）"),
            ),
        ]),
    ),
    ("option", keys_floor!(OPTION)),
    (
        "options_rule",
        Floor::Map(&[
            ("min", Floor::Num(OPTIONS_MIN)),
            ("adopted", Floor::Num(OPTIONS_ADOPTED)),
        ]),
    ),
    (
        "options_rule_note",
        Floor::Val("退けた案を 1 つ以上含み、採用は 1 つ"),
    ),
    ("retreat", keys_floor!(RETREAT)),
    (
        "retreat_note",
        Floor::Val("P-8.1。床は非空と kind の値域を見る＝「誰が数えるか」は本文に書く（人が見る）"),
    ),
    (
        "basis_note",
        Floor::Val(
            "条・要件・rules 行・判断の記録の id。各項は id の形（P-5.2）。条・要件・rules 行の未解決も判断の記録の未解決も欄の決まりの規則として床が落とす（rules 行 R-4 の行と母集団〔正本 4 file・内部 3 空間〕は変えない）",
        ),
    ),
    (
        "produced_note",
        Floor::Val(
            "その判断が発効で生んだもの（要件・受入基準・rules 行・後続の判断）の id の一覧。根拠（basis）には書かない＝面の章 04 は根拠だけを描く。各項は id の形（P-5.2）で、条と規範文は書かない（条の改訂は amends と amended_by が持つ）。自分の id と basis に在る id は書かない",
        ),
    ),
    (
        "prose_note",
        Floor::Val(
            "本文（見出し・問題・決定・案・撤退条件・平易文・帰結）と本 file の平易文の英字語は「日本語（原語）」の形で書く。語彙に無い裸の英字語は欄の決まりの規則として床が落とす（rules 行 R-9 の行と母集団〔憲法・rules・要件書・語彙〕は変えない）",
        ),
    ),
    (
        "amends_entry",
        Floor::Map(&[
            ("required", Floor::Strs(AMENDS_ENTRY.required)),
            ("optional", Floor::Strs(AMENDS_ENTRY.optional)),
            ("new_article_marker", Floor::Val("（新設）")),
            ("deleted_marker", Floor::Val("（削除）")),
            ("empty_marker", Floor::Val("（空）")),
        ]),
    ),
    (
        "amends_note",
        Floor::Map(&[
            (
                "target",
                Floor::Val(
                    "条 id か、改訂の範囲の節（憲法 schema.amendment_scope の各節。articles は条の並び）。現行に無くても列（過去の版の anchor）に在った条 id・節名は名指せる（範囲を狭める改訂・過去の版の記録）。条 id は P-n / A-n / N-n の形で、節名とは衝突しない（衝突する憲法は落ちる）",
                ),
            ),
            (
                "version",
                Floor::Val(
                    "改訂で上がる憲法の版（meta.version の値・例 v1.1）。差分検査はその版を名指す amends だけを見る＝1 本の判断が後の版まで許すことはない",
                ),
            ),
            (
                "field",
                Floor::Val(
                    "欄の道。条 = title / tier / binds / statements.order（規範文の並び・共通の id だけ）/ statements.<規範文 id>.text|pattern|strength。節 = 節の中の欄の道（例 enums.tier・text・mechanism.note）。articles = order（条の並び・共通の id だけ）。写しの欄名は文字列で「.」を含まない（欄の道 a.b と欄名「a.b」の衝突を塞ぐ・含む憲法は落ちる）",
                ),
            ),
            (
                "value",
                Floor::Val(
                    r#"値は型付き。文字列はそのまま（json として読める文字列は引用符付き）、一覧・表・数・真偽・空（null）は json の 1 値（一覧は丸ごと 1 値・例 '["always","ask-first","never"]'・空（null）は null）、空文字は（空）の印、前に無かった欄は previous_text が（新設）、今に無い欄は new_text が（削除）。印を値として持つ欄は落ちる"#,
                ),
            ),
            (
                "matching",
                Floor::Val(
                    "直前 anchor と現行の差分を欄単位で全件並べ、amends と 1 対 1 に消し込む。previous_text は前の値と完全一致・new_text は今の値と完全一致。余った差分も余った記録も落とす。同じ消し込みを列の全区間（隣り合う anchor どうし）でも行う＝次の版を凍結した後に前の版の amends を書き換えても落ちる。`folio check --emit-amends` が最新 anchor との差分を amends にそのまま貼れる形（1 行 1 件の json・- {…}）で印字する（違反があれば stderr に出し、終了コードは素の床と同じ）",
                ),
            ),
            (
                "renumber",
                Floor::Val(
                    "規範文の改番（同じ本文を消して別の id で足す）と、過去の版の anchor に在って最新 anchor に無い番号の再利用は落ちる（P-7.1）。本文も変えて付け替えた改番は「削除 + 新設」と弁別できない（限界）",
                ),
            ),
        ]),
    ),
    ("revises_entry", keys_floor!(REVISES_ENTRY)),
    (
        "revises_note",
        Floor::Map(&[
            (
                "target",
                Floor::Val(
                    "改訂する先の判断の記録の id（ADR-n）。自分の id は書かない。実在は判断の記録の全欄の走査が数える。条の改訂は amends と amended_by が持ち、判断を丸ごと置き換える形は supersedes / superseded_by が持つ＝この欄は「発効した判断が生きたまま、その決定の範囲が別の判断で変わる」ときだけに使う",
                ),
            ),
            (
                "decision",
                Floor::Val(
                    "改訂する決定の番号（その判断の decision の中の番号の字・例 (4)）。1 本の記録の中で target と decision の対は一意＝同じ決定を 2 行で書かない",
                ),
            ),
            (
                "kind",
                Floor::Val(
                    "改訂の向き。narrow = 決定の範囲を狭める／widen = 広げる。床は値域だけを見て、向きが本当かは人が読む（P-12.2）",
                ),
            ),
            (
                "summary",
                Floor::Val(
                    "その決定の何をどう変えたかの 1 文。逐語の突き合わせ（amends の previous_text / new_text）は持たない＝判断の記録は版ごとの凍結 anchor を持たないので、床が字面を突き合わせる相手が無い（P-10.2）",
                ),
            ),
            (
                "reverse",
                Floor::Val(
                    "改訂される側に来歴の欄は置かない（片側だけ）。改訂の有無は改訂する側のこの欄から数える＝条の改訂の来歴（amended_by）と違い、凍結 anchor との消し込みが無いので双方向にしても床が確かめられるものが増えない",
                ),
            ),
        ]),
    ),
    ("grill", keys_floor!(GRILL)),
    (
        "grill_note",
        Floor::Val("A-2.3 の記録（条文を改訂する発効した判断に必須）"),
    ),
    ("approval", keys_floor!(APPROVAL)),
    (
        "approval_note",
        Floor::Val(
            "P-12.2（逐語・日付・裁定 id・対話面）。発効した判断に必須。床は逐語の非空・日付の形・裁定 id の形・承認者の値域・対話面の id を見る。凍結後は anchor の承認一覧と 1 字も違わないこと（凍結後の書き換えは落ちる）。台帳 notes の逐語との突合は人が行う",
        ),
    ),
    ("amended_by_entry", keys_floor!(AMENDED_BY_ENTRY)),
    (
        "amended_by_note",
        Floor::Val(
            "発効した判断が条を amends に持つとき、その条の amended_by にその判断を指す 1 件以上が要る（双方向）。amended_by.previous_text はその判断の amends（同じ条）の previous_text のどれかと一致。approved_by はその判断の承認者と一致。来歴は累積し、差分検査はその版の amends だけを見る",
        ),
    ),
    (
        "supersede_note",
        Floor::Val(
            "retired は superseded_by（後継）が必須。後継の supersedes と双方向。後継の列は accepted に到達すること。後継が proposed の間は supersedes を書かない（床は双方向と「後継は発効している」を同時に見るので、下書きは 1 手も置けない）＝承認のときに前の判断の status と superseded_by・後継の status と supersedes の 4 欄を同時に置く",
        ),
    ),
    (
        "figures",
        Floor::Map(&[
            ("entry", keys_floor!(FIGURE_ENTRY)),
            ("type_enum_ref", Floor::Val(FIGURE_TYPE_ENUM_REF)),
        ]),
    ),
    (
        "figures_note",
        Floor::Val(
            "任意の図の節（便 33・ADR-4 決定 (1)）。図の正本は判断の記録の図の節に型付き記述（spec）で置き、面が図の道具で描く。型（type）は部品目録の図の道具の 5 型（archify-architecture・archify-workflow・archify-sequence・archify-dataflow・archify-lifecycle）。refs は basis と同じ id の形（条・要件・rules 行・判断の記録）で、行き先の解決は床では数えず面が「まだ分からない」で表す。図の id は 1 本の記録の中で一意。設計ノートと同じく、手順図（非エンジニア向け）と順序図（エンジニア向け）の対を指針とする",
        ),
    ),
    (
        "anchor",
        Floor::Map(&[
            ("dir", Floor::Val("anchors")),
            ("file_name", Floor::Val("constitution-<version>.yaml")),
            ("index_file", Floor::Val("index.yaml")),
            ("first_version", Floor::Val("v1.0")),
            (
                "root_digest",
                Floor::Val("acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed"),
            ),
            ("version_pattern", Floor::Val(r"^v[0-9]+\.[0-9]+$")),
            ("digest_algo", Floor::Val("sha256-json-1")),
            (
                "file_keys",
                Floor::Strs(&[
                    "kind",
                    "digest_algo",
                    "version",
                    "previous",
                    "projection",
                    "meta_approval",
                    "approvals",
                    "content",
                    "digest",
                ]),
            ),
            (
                "projection_article_fields",
                Floor::Strs(&["id", "title", "tier", "binds", "statements"]),
            ),
            (
                "statement_fields",
                Floor::Strs(&["id", "text", "pattern", "strength"]),
            ),
            (
                "scope_minimum",
                Floor::Strs(&["schema", "precedence", "articles"]),
            ),
        ]),
    ),
    (
        "anchor_note",
        Floor::Strs(&[
            "ADR-2 — A-2 / N-4 の差分検査の比較元（P-10.2 の限界を自認する）。file_name の version は憲法 meta.version の値そのまま（v1.0 → constitution-v1.0.yaml）。索引（index_file）は追記のみで、entries の順が列",
            "projection_article_fields は A-2.2 が名指す条文の 5 欄そのもの（写しの取り方 = 下限）。statement_fields は規範文の写し（中身 = 本文・型・強度・id は欄の道）。他の欄（将来の status 等）は写しの外。scope_minimum は A-2.2 の範囲の下限で、憲法 schema.amendment_scope はこれを含む",
            "digest = anchor の digest 以外の全欄を json（キー順固定・空白なし・ensure_ascii なし）に直列化した sha256。anchor の全欄（版・直前・範囲と取り方・発効の承認の写し・その版の承認一覧・写しの本体）を覆う。どこか 1 字でも手で変えると落ちる。方式（digest_algo）が床と違う anchor は digest を照合できない（まだ分からない）が、写しの内容の照合と版管理の履歴との照合は行う＝方式の欄を書き換えても改憲の違反は消えない",
            "版の綴りは v<数>.<数>（version_pattern）。v1.0.0 のような同じ版の別綴りは列に並べない",
            "現行の写し（憲法 schema.amendment_scope の各節・条は 5 欄）は最新 anchor と一致する（不一致 = 判断の記録と承認を伴わない改憲 → N-4）",
            "最新 anchor の版は憲法 meta.version と同じ（違えば「版を上げたのに凍結していない」→ A-2。凍結するまで記録の消し込みは測らない＝執筆中に前の版の改訂を告発しない。ただし条の消失・規範文の改番・廃止した番号の再利用は執筆中も測る）",
            "anchor は索引の entries の順に previous で列をなし、根は first_version で、根の anchor の digest は床の定数（root_digest）と一致する（根は 1 度きり＝別の写し〔別の版管理・根の無い枝・浅い写し・記録の無い版管理〕で同じ版を凍結し直して持ち帰っても落ちる。根を作り直すのは移行＝床の外の手順）。索引にある anchor file が無い（消された）・索引が在って entries が空、なら差分検査は「まだ分からない」（P-10.3・終了コード 2）——ただし版管理の履歴にその anchor が在れば「履歴に在ったが無い」の違反が先に立って終了コード 1 になる（2 になるのは履歴にも anchor が無い写しだけ）。索引に無い anchor・索引と違う digest・列の付け替え・索引だけの削除・索引を空にして anchor file が残る（列の外の anchor）は落とす（終了コード 1）",
            "発効の承認の写し（meta_approval）は憲法 meta.approval と一致する。承認一覧（approvals）の各項はその判断の記録が実在し、承認欄（承認者・日付・裁定 id・逐語・対話面）と 1 字も違わない",
            "直前 anchor との差分は欄単位で全件を、その版を名指す発効した判断の amends と 1 対 1 に消し込む（amends_note.matching）。変わった条には amended_by が要る。範囲（amendment_scope）の増減も欄単位の（新設）（削除）で記録する（狭めるときは列に在った節名を対象に名指す）。同じ消し込みを列の全区間（隣り合う anchor どうし）でも行う＝凍結後に過去の版の記録を書き換えても落ちる。発効した判断の amends が名指せる版は、列の根より後の版（隣り合う anchor の差分で消し込める）か執筆中の版（根でない）だけ＝列に無い版（最新版の anchor と索引の項を消して前の版へ戻した細工）も、列の根の版（改訂前が無いので突き合わせる差分が存在しない架空の記録）も落とす",
            "anchor に在って現行に無い条は落とす（番号は消さない・P-7。条の廃止（status）の機構は day-1 の憲法 schema に無い＝M0 で決める）。規範文 id の重複・改番・廃止した番号の再利用・印を値に持つ欄も落とす",
            "anchor が 0 本で改訂の記録も無ければ「まだ分からない」（P-10.3・終了コード 2。版管理の履歴に anchor が在れば「履歴に在ったが無い」で終了コード 1）。記録があるのに anchor が無ければ落とす",
            "版管理（git）との照合＝環境変数（GIT_DIR / GIT_WORK_TREE 等）は継承せず、全ての参照（--all）の履歴を見る。落とすもの = 先頭（HEAD）に anchor があって作業ツリーに無い・履歴に一度でも在った anchor が無い（削除を commit しても）・履歴に在った同じ形式の anchor と中身が違う（差し替え・書き換え。固定の欄・digest の方式・写しの取り方が今と違う古い anchor は移行の痕跡として見ない）・anchors/ か anchor file が版管理から除外（ignore）されている（追跡済みでも、file の pattern でも）・最新版以外の anchor が追跡されていない（凍結した anchor は commit する）・design-intent 自体が版管理の根・版管理の根が design-intent の上に無い。「まだ分からない」（終了コード 2）にするもの = 版管理が無い・commit が 1 つも無い（HEAD 無し）・読めない写し、と anchor が 1 本も無い浅い写し（shallow・anchor が揃った浅い写しは残りの検査で進む）＝写しで回すときも git init + commit の中で回す。版管理は比較元ではなく「消された・差し替えられた anchor」を早く止める補助で、列の真偽は索引と digest と根の定数が受け持つ",
            "同じ版の anchor は上書きしない・anchor と索引は消さない・空にしない。生成は folio check --freeze-anchor で、全検査が 0 違反かつ「まだ分からない」が無く、版が最新より新しく、差分とその版の発効した判断があるときだけ書く。列の始め直し（最初の版が first_version でない・版管理の先頭か履歴に anchor が在った・記録が在るのに anchor が無い・根の digest が床の定数と違う）は認めない",
            "正本 4 file（憲法・rules・語彙・要件書）・anchors/・adr/ とその中の file・design-intent 自体は symlink でなく実体",
        ]),
    ),
    (
        "limits_note",
        Floor::Strs(&[
            "前文（precedence）と schema 節の差分は amended_by を持てない（憲法 schema の precedence.optional が空）ので、それを amends に持つ発効した判断の記録の実在と欄単位の対の一致だけを確かめる。前文に amended_by を足すのは A-2 の改訂で行う。",
            "改番と番号の再利用は「同じ本文の付け替え」「過去の版に在って最新 anchor に無い番号の再登場」の 2 つの形だけを見る。本文も変えて付け替えた改番は弁別できない（削除 + 新設として通る）。",
            "anchor・この file・床の実装 は同じ作業ツリーの書き込める file である。床が保証するのは「記録の無い改訂が黙って通らない」ことまでで、anchor と索引と憲法を揃えて書き換え digest を計算し直す改竄（発効の承認の逐語・承認一覧を含む）と、床の実装 の定数（写しの取り方・承認者・対話面・裁定 id の形）を書き換える細工は、版管理の履歴と取り込みの審査（PR）が受け持つ。床の実装 の差分は審査で必ず読む。",
            "凍結と検査は同じ写しの関数を通る（生成物どうしの突き合わせ・P-10.2）。anchor は差分検査の比較元であって P-10.1 の「独立した凍結 anchor」ではない。P-10.1（live は M0）は未発効で、day-1 の代替として tests/floor_cases.yaml を置くが、床の入力ではない（fixture の不在は検出されない）＝M0 で床の入力に取り込む。",
            "条の mechanism（live・kind・note）・plain・rationale・relations と、憲法 meta（approval を除く）・amendment 章は A-2.2 の条文の定義の外なので差分検査の対象でない。変えるときは P-13.3（設計文書の変更は判断の記録か版付きの文書で残す）に従う＝機械は見ない。",
            "rules 行（rules.yaml）は本 anchor の外。P-17.2（裁定 id の無い rules 行の変更を落とす）と P-17.4（deny 行を緩めるときは A-2.3 を前置）の機構は憲法のとおり便 0 の検査（CI）で置く＝day-1 の床では効いていない。対話面 R-8 の中身（rules 行の what）も同じく anchor の外＝床は対話面を rules 行の id の実在と値域（surface）で見るだけで、行の本文の書き換えは止めない（便 0 の検査の領分）。",
            "裁定 id・逐語の「本物か」は床では見ない（形だけ）。--emit-amends の出力を貼れば改訂欄は機械的に埋まるので、記録が在ること自体は判断の内容を保証しない。台帳との突合は人が行う（P-12.2）。",
            "憲法 meta.approval は初回発効の承認で、以後の版では書き換えない（各版の承認は判断の記録の承認欄と anchor の承認一覧が持つ）。書き換えると全 anchor の写しと食い違って落ちる＝訂正するなら列の凍結し直しが要る。",
            "写しの取り方（条の 5 欄・規範文の 4 欄）や anchor の形式（固定の欄・digest の方式・列の根）を変える移行は床の実装 の改訂で、既存 anchor と食い違うので「まだ分からない」に落ちる。旧 anchor を消して同じ版で凍結し直す道は版管理の履歴照合が塞ぐ（削除を commit しても認めない）ので、移行は床の外の手順＝旧列の退避先・新列の根・床の定数の変更を判断の記録に書き、取り込みの審査で床の差分と一緒に読む（day-1 では想定しない）。",
            "「*_note」で終わる欄（この file の説明）は床が一切読まない（欄集合・語彙・参照 id の検査の外）。ここに書いた規則は「散文にしか無い規則」（N-2）であって規則ではない＝規則は *_note でない欄（床の定数の写し）と床の実装 にだけ置き、*_note はその説明に留める。",
            "改訂来歴（amended_by）のうち床が照合するのは承認者（判断の承認欄と一致）と previous_text（判断の amends と一致）で、理由（rationale）・日付・裁定 id は非空と形だけ＝凍結後に来歴の理由の文を書き換えても落ちない。amends の側は列の全区間の消し込みで固定される。",
            "条の廃止（status）の機構は day-1 の憲法 schema に無い（article の欄に status が無く、足しても写しの外）。条は消せず、廃止も表せない＝P-7.2「廃止は状態で」を条に適用する形は M0 で決める。",
            "「日本語（原語）」の括弧の中身は丸ごと英字語の免除になる（機械側の限界）。日本語の専門語の判定は敵対レビュー（天井）。",
            "終了コードは、違反（1）と「まだ分からない」（2）が同時に立てば 1。",
            "版管理の見え方（別の版管理・根の無い枝・浅い写し・記録の無い版管理・環境変数）は床の外で選べる。床が応じるのは環境変数の遮断・全ての参照の照合・同じ形式の anchor の中身の照合・列の根の digest の固定まで。履歴ごと書き換える細工（強制の push・参照の付け替え）と、床の実装 の定数（根の digest・写しの取り方・承認者・対話面・裁定 id の形）を書き換える細工は床の外＝取り込みの審査と remote の保護が受け持つ。",
            "列の根の digest は床の定数なので、床の実装 は folio2 の憲法 v1.0 の凍結に結び付いている。根を作り直す移行では床の定数を直す（取り込みの審査で読む）。",
        ]),
    ),
]);

/// 床の定数の値の一覧を欄の道で読む（便 6 の突き合わせの読み口・値は変えない）。道が一覧に着かなければ空。
pub(crate) fn floor_strs(path: &[&str]) -> &'static [&'static str] {
    match floor_at(path) {
        Some(Floor::Strs(items)) => items,
        _ => &[],
    }
}

/// 床の定数の値を欄の道で読む（便 7 の凍結 anchor の読み口・値は変えない）。道が値に着かなければ None。
pub(crate) fn floor_val(path: &[&str]) -> Option<&'static str> {
    match floor_at(path) {
        Some(Floor::Val(v)) => Some(v),
        _ => None,
    }
}

/// 床の定数の数を欄の道で読む（同上）。道が数に着かなければ None。
/// 便 7 の検査は数型を読まない（読み口として置く）。
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn floor_num(path: &[&str]) -> Option<usize> {
    match floor_at(path) {
        Some(Floor::Num(n)) => Some(*n),
        _ => None,
    }
}

fn floor_at(path: &[&str]) -> Option<&'static Floor> {
    let mut cur: &'static Floor = &FLOOR;
    for key in path {
        let Floor::Map(fields) = cur else {
            return None;
        };
        cur = &fields.iter().find(|(k, _)| k == key)?.1;
    }
    Some(cur)
}

const SCHEMA_FILE: &str = "adr/schema.yaml";

/// 読めた欄の決まり（adr/schema.yaml の木）と判断の記録（id と木の組・名前順）。
pub(crate) struct Adr {
    pub schema: Node,
    pub records: Vec<(String, Node)>,
}

/// `dir/adr/` の欄の決まりと判断の記録を検査する。欄の決まりが読めなければ None（「まだ分からない」は立ててある）。
pub fn check_adr(dir: &Path, report: &mut Report) -> Option<Adr> {
    let adr_dir = dir.join("adr");
    if adr_dir.is_symlink() || !adr_dir.is_dir() {
        report.unknown(format!(
            "adr/ が dir でない（symlink・file・不在）: {}",
            adr_dir.display()
        ));
        return None;
    }
    let schema = load_schema(&adr_dir, report)?;
    let mut drift = Vec::new();
    floor_diff(
        &strip_notes(schema.get("schema").unwrap_or(&Node::Null)),
        &FLOOR,
        "",
        &mut drift,
    );
    for path in drift {
        report.violation(
            "adr",
            format!(
                "{SCHEMA_FILE} schema.{path} が床の定数と違う（欄の決まりの閾値・値域・置き場は床の定数の写し＝data 側で動かせない・N-3.1）"
            ),
        );
    }
    let records = load_records(dir, &adr_dir, report);
    check_between(&records, report);
    check_decided_by(&schema, &records, report);
    Some(Adr { schema, records })
}

fn read(path: &Path) -> Result<yaml::Doc, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    yaml::parse(&text)
}

/// 重複キーは床の読み手と同じく「読めない」（まだ分からない）。在れば true。
fn duplicates(file: &str, doc: &yaml::Doc, report: &mut Report) -> bool {
    for dup in &doc.duplicates {
        report.unknown(format!(
            "{file} {} 行: 読めない（重複キー「{}」＝同じ表に 2 度書いている）",
            dup.line, dup.key
        ));
    }
    !doc.duplicates.is_empty()
}

/// (a) 欄の決まりの file。読めない・形が違う は「まだ分からない」。
fn load_schema(adr_dir: &Path, report: &mut Report) -> Option<Node> {
    let path = adr_dir.join("schema.yaml");
    if path.is_symlink() {
        report.unknown(format!("{SCHEMA_FILE}: symlink は認めない"));
        return None;
    }
    if !path.exists() {
        report.unknown(format!("{SCHEMA_FILE}: 欄の決まりが無い"));
        return None;
    }
    if !path.is_file() {
        report.unknown(format!("{SCHEMA_FILE}: file でない"));
        return None;
    }
    let doc = match read(&path) {
        Ok(d) => d,
        Err(e) => {
            report.unknown(format!("{SCHEMA_FILE}: 読めない: {e}"));
            return None;
        }
    };
    if duplicates(SCHEMA_FILE, &doc, report) {
        return None;
    }
    let sections_ok = doc.root.as_map().is_some_and(|m| {
        m.iter()
            .all(|(k, _)| matches!(k.as_str(), "meta" | "schema" | "plain"))
    });
    if !sections_ok || !matches!(doc.root.get("schema"), Some(Node::Map(_))) {
        report.unknown(format!(
            "{SCHEMA_FILE}: 形が違う（節は meta / schema / plain・schema 節は欄の表）"
        ));
        return None;
    }
    Some(doc.root)
}

/// (c) 判断の記録の file を名前順に読み、(d) の欄を見る。読めた記録を id と組で返す。
fn load_records(dir: &Path, adr_dir: &Path, report: &mut Report) -> Vec<(String, Node)> {
    let mut names: Vec<String> = match fs::read_dir(adr_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".yaml") && n != "schema.yaml")
            .collect(),
        Err(e) => {
            report.unknown(format!("adr/: 読めない: {e}"));
            return Vec::new();
        }
    };
    names.sort();
    let root = fs::canonicalize(dir).ok();
    let mut records: Vec<(String, Node)> = Vec::new();
    for name in names {
        let path = adr_dir.join(&name);
        let mut real = true;
        if path.is_symlink() {
            report.violation("adr", format!("{name}: symlink は認めない"));
            real = false;
        }
        let inside = match (&root, fs::canonicalize(&path)) {
            (Some(r), Ok(p)) => p.starts_with(r),
            _ => false,
        };
        if !inside {
            report.violation("adr", format!("{name}: design-intent の外を指している"));
            real = false;
        }
        if !real {
            continue;
        }
        let doc = match read(&path) {
            Ok(d) => d,
            Err(e) => {
                report.violation(
                    "adr",
                    format!("{name}: 判断の記録が欄の表でない（読めない: {e}）"),
                );
                continue;
            }
        };
        if duplicates(&format!("adr/{name}"), &doc, report) {
            continue;
        }
        let d = doc.root;
        if d.as_map().is_none() {
            report.violation("adr", format!("{name}: 判断の記録が欄の表でない"));
            continue;
        }
        let id = scalar(d.get("id")).unwrap_or("?").to_string();
        check_keys("adr", &id, &d, &RECORD, report);
        if !is_adr_id(&id) {
            report.violation(
                "adr",
                format!("{name}: id「{id}」が形 {ID_PATTERN} でない（ゼロ詰めしない）"),
            );
        }
        if name.strip_suffix(".yaml") != Some(id.as_str()) {
            report.violation(
                "adr",
                format!("{name}: file 名が id {id} と違う（1 判断 = 1 file・file 名 = id）"),
            );
        }
        if find(&records, &id).is_some() {
            report.violation("adr", format!("{id}: id が重複（P-7）"));
            continue;
        }
        check_fields(&id, &d, report);
        records.push((id, d));
    }
    records
}

/// (d) 1 本の判断の記録の欄。
fn check_fields(id: &str, d: &Node, report: &mut Report) {
    if !in_enum(d.get("status"), STATUS) {
        report.violation(
            "adr",
            format!("{id}: status が値域外: {}", show(d.get("status"))),
        );
    }
    check_date("adr", &format!("{id}.date"), d.get("date"), report);
    for k in NON_EMPTY {
        if !non_empty(d.get(k)) {
            report.violation("adr", format!("{id}: {k} が空"));
        }
    }

    let options: &[Node] = match d.get("options") {
        Some(Node::Seq(items)) => items,
        _ => {
            report.violation("adr", format!("{id}: options が一覧でない"));
            &[]
        }
    };
    if options.len() < OPTIONS_MIN {
        report.violation(
            "adr",
            format!(
                "{id}: 案が {} 件（退けた案を含めて {OPTIONS_MIN} 件以上）",
                options.len()
            ),
        );
    }
    for o in options {
        let at = format!("{id}.options[{}]", show(o.get("id")));
        if !check_keys("adr", &at, o, &OPTION, report) {
            continue;
        }
        if !in_enum(o.get("verdict"), VERDICT) {
            report.violation(
                "adr",
                format!("{at}: verdict が値域外: {}", show(o.get("verdict"))),
            );
        }
        for k in ["name", "text", "reason"] {
            if !non_empty(o.get(k)) {
                report.violation("adr", format!("{at}.{k} が空"));
            }
        }
    }
    let adopted = options
        .iter()
        .filter(|o| scalar(o.get("verdict")) == Some("adopted"))
        .count();
    if adopted != OPTIONS_ADOPTED {
        report.violation(
            "adr",
            format!("{id}: 採用の案が {adopted} 件（{OPTIONS_ADOPTED} 件）"),
        );
    }

    let retreat = d.get("retreat").unwrap_or(&Node::Null);
    if check_keys("P-8", &format!("{id}.retreat"), retreat, &RETREAT, report) {
        if !in_enum(retreat.get("kind"), RETREAT_KIND) {
            report.violation(
                "P-8",
                format!("{id}: retreat.kind が値域外: {}", show(retreat.get("kind"))),
            );
        }
        if !non_empty(retreat.get("condition")) {
            report.violation("P-8", format!("{id}: 撤退条件が空（P-8.1）"));
        }
    }

    match d.get("basis") {
        Some(Node::Seq(items)) if !items.is_empty() => {
            for b in items {
                if !b.as_str().is_some_and(is_basis_id) {
                    report.violation(
                        "adr",
                        format!(
                            "{id}: basis「{}」が id の形（条・要件・rules 行・判断の記録）でない（P-5.2）",
                            show(Some(b))
                        ),
                    );
                }
            }
        }
        _ => report.violation("adr", format!("{id}: basis（根拠の id）が空")),
    }

    // 帰結の欄（便 92）: 条でない id の形・自分と basis に無い id。実在は link.rs の網が数える
    match present(d, PRODUCED) {
        None => {}
        Some(Node::Seq(items)) => {
            let basis = d.get("basis").and_then(Node::as_seq).unwrap_or(&[]);
            for p in items {
                let v = p.as_str();
                if !v.is_some_and(|s| is_basis_id(s) && !is_article_id(s)) {
                    report.violation(
                        "adr",
                        format!(
                            "{id}: {PRODUCED}「{}」が id の形でない（要件・rules 行・判断の記録だけ・P-5.2）",
                            show(Some(p))
                        ),
                    );
                } else if v == Some(id) || basis.iter().any(|b| b.as_str() == v) {
                    report.violation(
                        "adr",
                        format!(
                            "{id}: {PRODUCED}「{}」が自分の id か根拠（basis）に在る",
                            show(Some(p))
                        ),
                    );
                }
            }
        }
        Some(_) => report.violation("adr", format!("{id}: {PRODUCED} が一覧でない")),
    }

    let amends: &[Node] = match present(d, "amends") {
        None => &[],
        Some(Node::Seq(items)) => items,
        Some(_) => {
            report.violation("A-2", format!("{id}: amends が一覧でない"));
            &[]
        }
    };
    for e in amends {
        let at = format!("{id}.amends[{}]", show(e.get("target")));
        if !check_keys("A-2", &at, e, &AMENDS_ENTRY, report) {
            continue;
        }
        for k in ["field", "version", "previous_text", "new_text"] {
            if !non_empty(e.get(k)) {
                report.violation("A-2", format!("{at}.{k} が空（空の値は印（空）で書く）"));
            }
        }
    }

    check_revises(id, d, report);

    let approval = present(d, "approval");
    if in_enum(d.get("status"), EFFECTIVE_STATUS) && approval.is_none_or(Node::is_blank) {
        report.violation(
            "N-4",
            format!(
                "{id}: {} なのに approval（逐語・日付・裁定 id・対話面）が無い",
                show(d.get("status"))
            ),
        );
    }
    if let Some(ap) = approval {
        check_approval(&format!("{id}.approval"), ap, report);
    }

    if let Some(grill) = present(d, "grill") {
        let at = format!("{id}.grill");
        if check_keys("A-2", &at, grill, &GRILL, report) {
            check_date("A-2", &format!("{at}.when"), grill.get("when"), report);
            for k in ["who", "where", "summary"] {
                if !non_empty(grill.get(k)) {
                    report.violation("A-2", format!("{at}.{k} が空"));
                }
            }
        }
    }

    check_figures(id, d, report);
}

/// (d) 任意の改訂の欄（便 101）。欄の集合・4 欄の非空・target は自分でない判断の記録の id・kind の値域・
/// target と decision の対は 1 本の記録の中で一意。実在は link.rs の網が数える。
fn check_revises(id: &str, d: &Node, report: &mut Report) {
    let items: &[Node] = match present(d, REVISES) {
        None => return,
        Some(Node::Seq(items)) => items,
        Some(_) => {
            report.violation("adr", format!("{id}: {REVISES} が一覧でない"));
            return;
        }
    };
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for (i, e) in items.iter().enumerate() {
        let at = format!("{id}.{REVISES}[{i}]");
        if !check_keys("adr", &at, e, &REVISES_ENTRY, report) {
            continue;
        }
        for k in REVISES_ENTRY.required {
            if !non_empty(e.get(k)) {
                report.violation("adr", format!("{at}.{k} が空"));
            }
        }
        let target = scalar(e.get("target"));
        if target.is_some_and(|t| !t.trim().is_empty())
            && !target.is_some_and(|t| is_adr_id(t) && t != id)
        {
            report.violation(
                "adr",
                format!(
                    "{at}.target が判断の記録の id でない（自分の id も書かない）: {}",
                    show(e.get("target"))
                ),
            );
        }
        if non_empty(e.get("kind")) && !in_enum(e.get("kind"), REVISE_KIND) {
            report.violation(
                "adr",
                format!("{at}: kind「{}」が値域でない", show(e.get("kind"))),
            );
        }
        if let (Some(t), Some(dec)) = (target, scalar(e.get("decision"))) {
            if seen.contains(&(t, dec)) {
                report.violation(
                    "adr",
                    format!("{at}: {t} の decision「{dec}」が 2 行に在る（対は一意）"),
                );
            } else {
                seen.push((t, dec));
            }
        }
    }
}

/// (d) 任意の図の節（便 33）。欄の集合・id と caption の非空・型は部品目録の一覧・spec は表・refs は basis と同じ
/// id の形・図の id は 1 本の記録の中で一意。行き先の解決は数えない。
fn check_figures(id: &str, d: &Node, report: &mut Report) {
    let figures: &[Node] = match present(d, "figures") {
        None => &[],
        Some(Node::Seq(items)) => items,
        Some(_) => {
            report.violation("adr", format!("{id}: figures が一覧でない"));
            &[]
        }
    };
    let mut seen: Vec<&str> = Vec::new();
    for f in figures {
        let at = format!("{id}.figures[{}]", show(f.get("id")));
        if !check_keys("adr", &at, f, &FIGURE_ENTRY, report) {
            continue;
        }
        // 欠落は check_keys が数えてあるので、非空は在る欄だけ見る
        for k in ["id", "caption"] {
            if f.get(k).is_some() && !non_empty(f.get(k)) {
                report.violation("adr", format!("{at}.{k} が空"));
            }
        }
        if let Some(t) = f.get("type")
            && !t
                .as_str()
                .is_some_and(|v| FigureType::from_name(v).is_some())
        {
            report.violation(
                "adr",
                format!(
                    "{at}: 図の型「{}」が部品目録の一覧に無い（{FIGURE_TYPE_ENUM_REF}）",
                    show(Some(t))
                ),
            );
        }
        if let Some(spec) = f.get("spec")
            && !matches!(spec, Node::Map(_))
        {
            report.violation("adr", format!("{at}: spec が表でない"));
        }
        match present(f, "refs") {
            None => {}
            Some(Node::Seq(items)) => {
                for r in items {
                    if !r.as_str().is_some_and(is_basis_id) {
                        report.violation(
                            "adr",
                            format!(
                                "{at}: refs「{}」が id の形（条・要件・rules 行・判断の記録）でない",
                                show(Some(r))
                            ),
                        );
                    }
                }
            }
            Some(_) => report.violation("adr", format!("{at}: refs が一覧でない")),
        }
        if let Some(fid) = scalar(f.get("id")) {
            if seen.contains(&fid) {
                report.violation(
                    "adr",
                    format!("{id}: 図の id「{fid}」が重複（1 本の記録の中で一意）"),
                );
            } else {
                seen.push(fid);
            }
        }
    }
}

fn check_approval(at: &str, ap: &Node, report: &mut Report) {
    const KIND: &str = "N-4";
    if !check_keys(KIND, at, ap, &APPROVAL, report) {
        return;
    }
    for k in ["ruling", "verbatim"] {
        if !non_empty(ap.get(k)) {
            report.violation(KIND, format!("{at}.{k} が空"));
        }
    }
    check_date(KIND, &format!("{at}.date"), ap.get("date"), report);
    if !in_enum(ap.get("who"), APPROVER) {
        report.violation(KIND, format!("{at}.who が値域外: {}", show(ap.get("who"))));
    }
    if !scalar(ap.get("ruling")).is_some_and(has_ledger_id) {
        report.violation(
            KIND,
            format!(
                "{at}.ruling「{}」に台帳 id（{RULING_PATTERN}）が無い",
                show(ap.get("ruling"))
            ),
        );
    }
    if !in_enum(ap.get("surface"), SURFACE) {
        report.violation(
            KIND,
            format!(
                "{at}.surface が値域外（対話面は rules 行の id で指す・R-8）: {}",
                show(ap.get("surface"))
            ),
        );
    }
}

/// (e) 判断の記録どうし。
fn check_between(records: &[(String, Node)], report: &mut Report) {
    for (id, d) in records {
        let effective = in_enum(d.get("status"), EFFECTIVE_STATUS)
            && matches!(d.get("approval"), Some(Node::Map(_)));
        let amends = matches!(d.get("amends"), Some(Node::Seq(items)) if items.iter().any(|e| e.as_map().is_some()));
        if effective && amends {
            let who = d.get("approval").and_then(|a| a.get("who"));
            if scalar(who) != Some(OWNER) {
                report.violation(
                    "N-4",
                    format!(
                        "{id}: 条文を改訂する発効した判断の承認者が {OWNER} でない（{}）",
                        show(who)
                    ),
                );
            }
            if !matches!(d.get("grill"), Some(Node::Map(_))) {
                report.violation(
                    "A-2",
                    format!("{id}: 条文を改訂する発効した判断に grill の記録が無い（A-2.3）"),
                );
            }
        }
        for k in ["supersedes", "superseded_by"] {
            if let Some(x) = present(d, k)
                && x.as_str().and_then(|x| find(records, x)).is_none()
            {
                report.violation(
                    "adr",
                    format!("{id}: {k} {} の判断の記録が実在しない", show(Some(x))),
                );
            }
        }
        let status = scalar(d.get("status"));
        let next = present(d, "superseded_by");
        if status == Some("retired") && next.is_none() {
            report.violation(
                "adr",
                format!("{id}: retired なのに superseded_by（後継）が無い（P-7.2）"),
            );
        }
        if let Some(n) = next {
            if status != Some("retired") {
                report.violation(
                    "adr",
                    format!("{id}: superseded_by を持つのに status が retired でない（P-7.2）"),
                );
            }
            if let Some(nx) = n.as_str().and_then(|n| find(records, n))
                && scalar(nx.get("supersedes")) != Some(id.as_str())
            {
                report.violation(
                    "adr",
                    format!(
                        "{id}: 後継 {} の supersedes に {id} が無い（双方向）",
                        show(Some(n))
                    ),
                );
            }
        }
        if let Some(p) = present(d, "supersedes")
            && let Some(pv) = p.as_str().and_then(|p| find(records, p))
            && scalar(pv.get("superseded_by")) != Some(id.as_str())
        {
            report.violation(
                "adr",
                format!(
                    "{id}: 置き換えた {} の superseded_by が {id} でない（双方向）",
                    show(Some(p))
                ),
            );
        }
    }

    // retired の後継の列は accepted に着く（輪・未発効の後継は落とす）。
    for (id, d) in records {
        if scalar(d.get("status")) != Some("retired") {
            continue;
        }
        let mut seen: Vec<&str> = vec![id];
        let mut cur = d;
        while let Some(nid) = scalar(cur.get("superseded_by")) {
            let Some(nx) = find(records, nid) else {
                break; // 実在しない後継は上で数えてある
            };
            if seen.contains(&nid) {
                report.violation(
                    "adr",
                    format!(
                        "{id}: retired の後継の列が輪になっている（{}→{nid}）＝発効している後継が無い（P-7.2）",
                        seen.join("→")
                    ),
                );
                break;
            }
            seen.push(nid);
            match scalar(nx.get("status")) {
                Some("accepted") => break,
                Some("retired") => cur = nx,
                other => {
                    report.violation(
                        "adr",
                        format!(
                            "{id}: retired の後継の列の先 {nid} が発効していない（status {}）",
                            other.unwrap_or("（無い）")
                        ),
                    );
                    break;
                }
            }
        }
    }
}

/// (a) meta.decided_by の非空と実在。
fn check_decided_by(schema: &Node, records: &[(String, Node)], report: &mut Report) {
    let items: Vec<&Node> = match schema.get("meta").and_then(|m| m.get("decided_by")) {
        Some(Node::Seq(items)) => items.iter().collect(),
        Some(n) if !n.is_blank() => vec![n],
        _ => Vec::new(),
    };
    if items.is_empty() {
        report.violation(
            "adr",
            format!("{SCHEMA_FILE} meta.decided_by が空（欄の決まりの出所の判断が無い）"),
        );
    }
    for x in items {
        if x.as_str().and_then(|x| find(records, x)).is_none() {
            report.violation(
                "adr",
                format!(
                    "{SCHEMA_FILE} meta.decided_by の {} が実在しない（欄の決まりの出所の判断が消えている）",
                    show(Some(x))
                ),
            );
        }
    }
}

fn find<'a>(records: &'a [(String, Node)], id: &str) -> Option<&'a Node> {
    records.iter().find(|(k, _)| k == id).map(|(_, d)| d)
}

/// 欄の集合が keys と一致するか（required の欠落・未知の欄を 1 件ずつ）。表でなければ false。
pub(crate) fn check_keys(
    kind: &str,
    at: &str,
    node: &Node,
    keys: &Keys,
    report: &mut Report,
) -> bool {
    let Some(entries) = node.as_map() else {
        report.violation(kind, format!("{at}: 型が違う（欄の表でない）"));
        return false;
    };
    let missing: Vec<&str> = keys
        .required
        .iter()
        .copied()
        .filter(|k| node.get(k).is_none())
        .collect();
    if !missing.is_empty() {
        report.violation(kind, format!("{at}: 必須欄が無い: {}", missing.join("・")));
    }
    let unknown: Vec<&str> = entries
        .iter()
        .map(|(k, _)| k.as_str())
        .filter(|k| !keys.required.contains(k) && !keys.optional.contains(k))
        .collect();
    if !unknown.is_empty() {
        report.violation(
            kind,
            format!("{at}: 未知の欄（N-3）: {}", unknown.join("・")),
        );
    }
    true
}

/// 欄が在って値を持つ（null でない）。
pub(crate) fn present<'a>(node: &'a Node, key: &str) -> Option<&'a Node> {
    node.get(key).filter(|v| !matches!(v, Node::Null))
}

pub(crate) fn scalar(node: Option<&Node>) -> Option<&str> {
    node.and_then(Node::as_str)
}

pub(crate) fn show(node: Option<&Node>) -> String {
    match node {
        Some(Node::Scalar(s)) => s.clone(),
        Some(Node::Seq(_)) => "（一覧）".to_string(),
        Some(Node::Map(_)) => "（表）".to_string(),
        Some(Node::Null) | None => "（無い）".to_string(),
    }
}

pub(crate) fn in_enum(node: Option<&Node>, values: &[&str]) -> bool {
    scalar(node).is_some_and(|s| values.contains(&s))
}

/// 前後の空白を落として空でない（null と空白だけの文字列が空）。
pub(crate) fn non_empty(node: Option<&Node>) -> bool {
    match node {
        None | Some(Node::Null) => false,
        Some(Node::Scalar(s)) => !s.trim().is_empty(),
        Some(_) => true,
    }
}

pub(crate) fn check_date(kind: &str, at: &str, node: Option<&Node>, report: &mut Report) {
    if !scalar(node).is_some_and(is_date) {
        report.violation(kind, format!("{at}「{}」が年-月-日でない", show(node)));
    }
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// 年 4 桁-月 2 桁-日 2 桁（date_format）。
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && digits(&s[..4])
        && digits(&s[5..7])
        && digits(&s[8..])
}

/// ADR- の後に 1〜9 で始まる数字列だけ（id_pattern）。
fn is_adr_id(s: &str) -> bool {
    s.strip_prefix("ADR-")
        .is_some_and(|n| digits(n) && !n.starts_with('0'))
}

/// basis の各項の id の形（条・要件・rules 行・判断の記録の全体一致）。要件書の図の refs（`check.rs`・便 34）も同じ判定を呼ぶ。
pub(crate) fn is_basis_id(s: &str) -> bool {
    let article = is_article_id(s);
    let req = ["FR", "NFR", "AC", "CON", "GOAL"]
        .iter()
        .any(|p| s.strip_prefix(p).is_some_and(digits));
    let row = ["R-", "D-"]
        .iter()
        .any(|p| s.strip_prefix(p).is_some_and(digits));
    article || req || row || is_adr_id(s)
}

/// 条と規範文の id（P-n / A-n / N-n と P-n.m の形）。帰結の欄 produced は受けない（便 92）。
fn is_article_id(s: &str) -> bool {
    ["P-", "A-", "N-"].iter().any(|p| {
        s.strip_prefix(p)
            .is_some_and(|rest| match rest.split_once('.') {
                Some((n, sub)) => digits(n) && digits(sub),
                None => digits(rest),
            })
    })
}

/// 小文字の英字 1 字 + 数字 1 字 + 「-」+ 小文字の英字か数字 1 字 の並びを含む（ruling_pattern の search）。
pub(crate) fn has_ledger_id(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    chars.windows(4).any(|w| {
        w[0].is_ascii_lowercase()
            && w[1].is_ascii_digit()
            && w[2] == '-'
            && (w[3].is_ascii_lowercase() || w[3].is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_shapes_are_scanned_without_regex() {
        for ok in [
            "P-1", "A-2.3", "FR12", "NFR3", "GOAL2", "R-14", "D-1", "ADR-4",
        ] {
            assert!(is_basis_id(ok), "{ok}");
        }
        for ng in ["P-", "P-1.", "ADR-0047", "FR", "R-1.2", "X-1", "p-1"] {
            assert!(!is_basis_id(ng), "{ng}");
        }
        assert!(has_ledger_id("f2-648.2 notes"));
        assert!(!has_ledger_id("F2-648"));
        assert!(is_date("2026-09-17"));
        assert!(!is_date("2026-9-17"));
    }

    #[test]
    fn floor_values_and_numbers_are_read_through_the_floor() {
        assert_eq!(
            floor_val(&["anchor", "root_digest"]),
            Some("acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed")
        );
        assert_eq!(floor_val(&["anchor", "first_version"]), Some("v1.0"));
        assert_eq!(floor_val(&["amends_entry", "empty_marker"]), Some("（空）"));
        assert_eq!(floor_val(&["anchor", "file_keys"]), None);
        assert_eq!(floor_num(&["options_rule", "min"]), Some(2));
        assert_eq!(floor_num(&["options_rule", "adopted"]), Some(1));
        assert_eq!(floor_num(&["owner"]), None);
    }

    /// 床の突き合わせは FLOOR の説明の注（`_note`）を読まない＝注を持つ FLOOR と注の無い写しの差は 0。
    #[test]
    fn floor_notes_are_outside_the_diff() {
        let Floor::Map(fields) = &FLOOR else {
            unreachable!()
        };
        let notes = fields.iter().filter(|(k, _)| k.ends_with("_note")).count();
        assert_eq!(notes, 24);
        let mut out = Vec::new();
        floor_diff(&strip_notes(&Node::Map(Vec::new())), &FLOOR, "", &mut out);
        // 空の写し = 値の欄が全部（欠落）・注は 1 本も立たない
        assert_eq!(out.len(), fields.len() - notes, "{out:?}");
        assert!(out.iter().all(|p| p.ends_with("（欠落）")), "{out:?}");
        assert!(out.iter().all(|p| !p.contains("_note")), "{out:?}");
    }

    /// 床の木の導出は凍結 anchor（設計判断の席が独立の実装で組んだ・P-10.1）と byte 一致（便 58 §1 (e)3・
    /// `ceiling.rs` / `rules.rs` / `note.rs` の同名の歯と同じ形で `schema.rs` の pub の `derive` を呼ぶ）。
    #[test]
    fn adr_floor_derives_the_frozen_anchor_byte_for_byte() {
        let anchor = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/schema/adr-region.txt"
        ))
        .unwrap();
        assert_eq!(crate::floor::derive(&FLOOR), anchor);
    }

    /// 承認者の値域は 持ち主・planner 席・orchestrator 席 の 3 つ（席の呼び名の裁定 2026-09-20・便 58 §1 (a)1・
    /// planner 席 は凍結の場合と過去の記録のために残す）。
    #[test]
    fn adr_floor_approver_enum_has_three_values() {
        assert_eq!(APPROVER, ["持ち主", "planner 席", "orchestrator 席"]);
    }
}
