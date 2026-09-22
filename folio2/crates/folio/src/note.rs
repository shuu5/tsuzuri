//! `folio check` の設計ノートの正本（`design-note/*.yaml`・ADR-3 決定 (1)・要件書 FR9 / FR10）の形の検査
//! （便 23・docs/design/delivery-23.md §1）。
//! 数えるのは 欄の決まり（同じ dir の `schema.yaml`）の schema 節が定める形（文書と meta の欄・節の番号と型・
//! 型ごとの行の欄と値域・承認欄の要否）と、契約表の節の欄（器 scribe2 の導出 file `contracts/schema.toml` から読む・
//! 欄の一覧も値域も自分の型にも散文にも持たない・FR10）と、参照 id の解決（folio2 が所有する id 空間）である。
//! 散文の門（FR12・rules 行 R-16）は便 24 で入り、索引（FR14）は 2026-09-22 に着地した（folio graph --print・graph.rs）。導出物（FR11）は未実装（欄の決まりの注が明記する）。
//! 欄の決まりの閾値・値域・置き場は床の定数（`FLOOR`）で持ち、`design-note/schema.yaml` の schema 節はその写し
//! （判断の記録の欄の決まり `adr.rs` と同じ作り・N-3.1）。パターンの文字列は定数として字面で持つだけで、
//! 形の判定は字の走査で行う（正規表現は使わない）。器の導出 file（TOML）も行走査で読む（外部 crate を足さない）。
//! 便 46 から床の機械（床の木の型・突き合わせ）は `schema.rs` のものを使い、schema 節は生成区間で
//! `folio schema --write` が `FLOOR` から導出する＝説明の注（`_note` で終わる欄）も FLOOR が file の順と字面のまま持つ。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::adr::Adr;
use crate::check::{duplicate_ids, non_empty, row_id, unknown_sections};
use crate::link;
use crate::parts::catalog::FigureType;
use crate::prose;
use crate::refs;
use crate::schema::{Floor, floor_diff, keys_floor, strip_notes};
use crate::verdict::Report;
use crate::yaml::{self, Node};

/// 設計ノートの置き場（正本の dir の直下）と欄の決まりの file。
const DIR: &str = "design-note";
const SCHEMA_FILE: &str = "design-note/schema.yaml";

/// 違反の種別（設計ノートの形・未知の欄・散文の門）。
const KIND: &str = "note";
const UNKNOWN_FIELD: &str = "未知の欄";
const PROSE_GATE: &str = "prose-gate";

// ── 床の定数（design-note/schema.yaml の schema 節の正本） ──

/// 欄の集合（required / optional）。
struct Keys {
    required: &'static [&'static str],
    optional: &'static [&'static str],
}

impl Keys {
    fn has(&self, key: &str) -> bool {
        self.required.contains(&key) || self.optional.contains(&key)
    }

    fn all(&self) -> Vec<&'static str> {
        self.required
            .iter()
            .chain(self.optional.iter())
            .copied()
            .collect()
    }
}

const PATH_BASE: &str = "repo-root";
const DATE_FORMAT: &str = r"^\d{4}-\d{2}-\d{2}$";
const DOC: Keys = Keys {
    required: &["meta", "sections"],
    optional: &["figures", "sources"],
};
const DOC_META: Keys = Keys {
    required: &["id", "title", "version", "status", "generated", "profile"],
    optional: &["approval", "supersedes", "superseded_by", "note"],
};
const ID_PATTERN: &str = "^[a-z][a-z0-9-]*$";
const VERSION_PATTERN: &str = r"^v[0-9]+\.[0-9]+$";
const STATUS_ENUM: &[&str] = &["draft", "effective", "retired", "example"];
const EFFECTIVE_STATUS: &[&str] = &["effective", "retired"];
const STATUS_EXAMPLE: &str = "example";
const STATUS_RETIRED: &str = "retired";
const APPROVAL_REQUIRED: &[&str] = &["who", "date", "ruling", "verbatim", "surface"];
const SURFACE_ENUM: &[&str] = &["R-8"];
const SECTION: Keys = Keys {
    required: &["n", "type", "title"],
    optional: &["body", "rows", "note"],
};
const TYPE_ENUM: &[&str] = &[
    "prose",
    "parts-table",
    "ports-table",
    "fields-table",
    "teeth-table",
    "contract-table",
];
const PROSE: &str = "prose";
const CONTRACT_TABLE: &str = "contract-table";
/// 節の型ごとの required / forbid（by_type）。
const NEEDS_BODY: &[&str] = &["body"];
const NEEDS_ROWS: &[&str] = &["rows"];
const FORBIDS_ROWS: &[&str] = &["rows"];
const PARTS_ROW: Keys = Keys {
    required: &["id", "name", "role"],
    optional: &["ref", "note"],
};
const PORTS_ROW: Keys = Keys {
    required: &["id", "name", "input", "output", "refuses"],
    optional: &["ref", "note"],
};
const REFUSES_NONE_MARKER: &str = "なし";
const FIELDS_ROW: Keys = Keys {
    required: &["id", "name", "need", "shape"],
    optional: &["enum", "note"],
};
const NEED_ENUM: &[&str] = &["required", "optional"];
const SHAPE_ENUM: &[&str] = &["text", "list", "number", "bool", "table"];
const TEETH_ROW: Keys = Keys {
    required: &["id", "name", "red_when", "fixture"],
    optional: &["ref", "note"],
};
/// 器（scribe2）の導出 file の置き場（repo の根からの相対）と読み手の期待する形。
const EXTERNAL_PATH: &str = "contracts/schema.toml";
const EXTERNAL_HEAD: &str = "schema = 1";
const EXTERNAL_ROWS_KEY: &str = "field";
const EXTERNAL_ROW_FIELDS: &[&str] = &["name", "need", "shape"];
/// 導出 file の need / shape の値域（読めない値は「まだ分からない」）。
const EXTERNAL_NEED: &[&str] = &["required", "optional"];
const EXTERNAL_SHAPE: &[&str] = &["text", "list"];
const ROW_ID_PATTERN: &str = "^[a-z][a-z0-9-]*$";
const FIGURE_ENTRY: Keys = Keys {
    required: &["id", "type", "caption", "spec"],
    optional: &["refs", "note"],
};

/// 床の定数（値は欄の決まり design-note/schema.yaml の schema 節の字面と 1 字も違わない）。`_note` で終わる欄は
/// 人が読む説明の注（便 46・ADR-9）で、生成区間に在る順と字面のまま持つ＝床の突き合わせ（`floor_diff`）は読まず、
/// `folio schema` の導出だけが使う。真偽は Val の字面（true / false）で持つ（導出は裸の true / false を出す）。
/// 注は 1 欄 1 行で持つ（file の 1 行と対にして読めるように・rustfmt は掛けない）。
#[rustfmt::skip]
pub(crate) const FLOOR: Floor = Floor::Map(&[
    ("floor_note", Floor::Val("以下の欄は床の実装の定数からの導出物である（生成区間・判断の記録 ADR-9）。型も値も床の定数と違わないことを folio check が数える（判断の記録の欄の決まりと同じ）。変えるときは床の実装の定数を直し、folio schema --write で書き直す。")),
    ("path_base", Floor::Val(PATH_BASE)),
    ("path_base_note", Floor::Val("本 file と設計ノートの中の path（fixture・parts.json・導出物）は repo の根からの相対で書く。")),
    ("date_format", Floor::Val(DATE_FORMAT)),
    (
        "doc",
        Floor::Map(&[
            ("required", Floor::Strs(DOC.required)),
            ("optional", Floor::Strs(DOC.optional)),
            (
                "sources",
                Floor::Map(&[("in_ref_population", Floor::Val("false"))]),
            ),
            ("doc_note", Floor::Val("1 設計ノート = YAML 1 file。節（sections）の並びが本文。図（figures）は判断の記録 ADR-4 の型付き記述で持つ。外部への参照（報告 HTML 等）は sources に置き、参照 id の母集団に入れない（in_ref_population = false・要件書 NFR3 と同じ）")),
        ]),
    ),
    (
        "doc_meta",
        Floor::Map(&[
            ("required", Floor::Strs(DOC_META.required)),
            ("optional", Floor::Strs(DOC_META.optional)),
            ("id_pattern", Floor::Val(ID_PATTERN)),
            ("id_note", Floor::Val("文書 id（doc id）= file 名の stem。append-only＝改名は「新しい id + 旧 id の廃止（status retired・superseded_by）」で表し、番号や名を再利用しない（P-7）。契約 id は「<doc id>#<row id>」の形で、前半がこの id（器 scribe2 の契約表と同じ形）")),
            ("version_pattern", Floor::Val(VERSION_PATTERN)),
            ("status_enum", Floor::Strs(STATUS_ENUM)),
            ("effective_status", Floor::Strs(EFFECTIVE_STATUS)),
            ("approval_required_when", Floor::Val("effective_status")),
            (
                "status_note",
                Floor::Map(&[
                    ("draft", Floor::Val("未承認・拘束力なし（承認欄は空でよい）")),
                    ("effective", Floor::Val("発効（承認欄に持ち主の逐語・日付・裁定 id・対話面が必須）")),
                    ("retired", Floor::Val("廃止（superseded_by 必須・P-7.2・承認欄を持つ）")),
                    ("example", Floor::Val("見本（拘束力なし・承認欄を持たない・凍結 anchor の材料）")),
                ]),
            ),
            ("profile_enum", Floor::Strs(crate::parts::catalog::PROFILES)),
            ("profile_note", Floor::Val("密度 profile は 1 行（見せ方だけを持つ・拘束の旗を置かない・ADR-3 決定 (1)・N-3）。文書の種類による違いは節の型で表す（P-5.3）")),
            (
                "approval",
                Floor::Map(&[
                    ("required", Floor::Strs(APPROVAL_REQUIRED)),
                    ("surface_enum", Floor::Strs(SURFACE_ENUM)),
                ]),
            ),
            ("approval_note", Floor::Val("P-12.2。effective_status の文書にだけ必須（approval_required_when）。承認者の値域・裁定 id の形は判断の記録の欄の決まり（adr/schema.yaml）と同じ定数を床が持つ。持ち主との対話面（R-8）を通っていない記録に承認欄を置かない（P-12.3）")),
        ]),
    ),
    (
        "section",
        Floor::Map(&[
            ("required", Floor::Strs(SECTION.required)),
            ("optional", Floor::Strs(SECTION.optional)),
            (
                "n_rule",
                Floor::Map(&[
                    ("start", Floor::Num(1)),
                    ("order", Floor::Val("ascending")),
                    ("append_only", Floor::Val("true")),
                    ("gaps_allowed", Floor::Val("true")),
                ]),
            ),
            ("n_note", Floor::Val("節番号（§N の N）。folio2 の自前の決まり（P-7.1 の番号の扱いを節に当てたもの・判断の記録と要件書には無い）。契約表の行の section 欄はこの n を指す（見出しの字面ではない）。器（scribe2）の設計文書の「## N.」と同じ意味")),
            ("type_enum", Floor::Strs(TYPE_ENUM)),
            ("type_note", Floor::Val("節の型の閉じた一覧（P-2.4・裁定は meta.type_enum_ruling）。判断の記録 ADR-3 決定 (1) が名指す 部品の表・口の表・欄の表・歯の表・契約表 に、散文の節（要件書 FR12 の母集団）を足した 6 つ。要件書 FR9 = 一覧に無い型の節を持つ正本は生成せずに落とす")),
            (
                "by_type",
                Floor::Map(&[
                    (
                        "prose",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_BODY)),
                            ("forbid", Floor::Strs(FORBIDS_ROWS)),
                            ("prose_gate_rules_row", Floor::Val("R-16")),
                            ("body_note", Floor::Val("散文。規範の印を持つ文の門（同じ文に参照 id・数と単位を持たない）は rules 行 R-16 の値（印・「禁止」の直後の文字・単位）と population（母集団の除外・文の区切り）が持ち、ここには写さない（要件書 FR12）")),
                        ]),
                    ),
                    (
                        "parts-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("row", keys_floor!(PARTS_ROW)),
                            ("row_note", Floor::Val("部品の表。role = その部品が何をするか（1 行）。ref = 参照 id（条・要件・rules 行・判断の記録）の一覧")),
                        ]),
                    ),
                    (
                        "ports-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("row", keys_floor!(PORTS_ROW)),
                            ("refuses_none_marker", Floor::Val(REFUSES_NONE_MARKER)),
                            ("row_note", Floor::Val("口の表（命令・関数・接点）。refuses = 何を断るか（黙って飛ばさない・P-4）。断らない口は refuses_none_marker の値を書く（空にしない）")),
                        ]),
                    ),
                    (
                        "fields-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("row", keys_floor!(FIELDS_ROW)),
                            ("need_enum", Floor::Strs(NEED_ENUM)),
                            ("shape_enum", Floor::Strs(SHAPE_ENUM)),
                            ("row_note", Floor::Val("欄の表（folio2 自身の型付きデータの欄を記述する節）。need / shape の値域は folio2 の自前（契約表の外部の欄の決まりとは別物＝器の値域を写したものではない）")),
                        ]),
                    ),
                    (
                        "teeth-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("row", keys_floor!(TEETH_ROW)),
                            ("row_note", Floor::Val("歯の表（検査・test）。red_when = 何を壊せば落ちるか（1 文）。fixture = 固定の材料の path（凍結 anchor・P-10.1・repo の根からの相対）。要件書の受入基準の red_test と同じ形")),
                        ]),
                    ),
                    (
                        "contract-table",
                        Floor::Map(&[
                            ("required", Floor::Strs(NEEDS_ROWS)),
                            ("section_ref_type", Floor::Val(PROSE)),
                            ("section_ref_note", Floor::Val("行の section 欄は同じ文書の節番号 n を指し、その節は prose の型であること（folio2 側の導出の成立条件 = 節の body の逐語を goal へ写すため。器 scribe2 は「節が在り本文が非空」だけを見る＝folio2 が導出のために足す条件で、器の受付を狭めない）")),
                            ("rows_note", Floor::Val("契約表。行の欄の集合と値域は本 file に書かない＝schema.contract_table.external_schema が指す器（scribe2）の導出 file をそのまま読む（ADR-3 決定 (2)・要件書 FR10）")),
                        ]),
                    ),
                ]),
            ),
        ]),
    ),
    (
        "contract_table",
        Floor::Map(&[
            (
                "external_schema",
                Floor::Map(&[
                    ("owner", Floor::Val("scribe2")),
                    ("path", Floor::Val(EXTERNAL_PATH)),
                    ("format", Floor::Val("toml")),
                    (
                        "reader_expects",
                        Floor::Map(&[
                            ("head", Floor::Val(EXTERNAL_HEAD)),
                            ("rows_key", Floor::Val(EXTERNAL_ROWS_KEY)),
                            ("row_fields", Floor::Strs(EXTERNAL_ROW_FIELDS)),
                        ]),
                    ),
                    ("value_domains", Floor::Val("from-file")),
                    ("unknown_value", Floor::Val("まだ分からない")),
                ]),
            ),
            ("external_schema_note", Floor::Val("器（scribe2）が自分の型（pipe/table.rs の定数）から導出した生成物。folio2 はこれを読んで契約表の節の欄を登録し、欄の一覧も値域も自分の型にも散文にも持たない（P-5.1・P-6.3・P-6.4・N-2）＝reader_expects は読み手の期待する形であって正本ではなく、need / shape の値域は file の値をそのまま受ける（value_domains = from-file）。欄の追加・値域の変更は器の版上げで足り、folio2 の判断の記録は要らない。file が読めない・期待する形でない・知らない値が在るときは「まだ分からない」（unknown_value・要件書 FR10・AC8）")),
            (
                "reads",
                Floor::Strs(&["design-doc-contract-table", "external-schema-file"]),
            ),
            ("never_reads", Floor::Strs(&["per-run-contract-file"])),
            ("reads_note", Floor::Val("folio2 が読むのは設計文書の中の契約表（scribe2 では docs/design の [[contract]] の区間）と外部の欄の決まりの file だけ。便ごとの契約 file（run dir の contract.toml）は読まない（scribe2 planner の助言 2026-09-16・形が変わる途中）")),
            (
                "row_id",
                Floor::Map(&[
                    ("pattern", Floor::Val(ROW_ID_PATTERN)),
                    ("owner", Floor::Val("folio2")),
                    ("scope", Floor::Val("own-id-space")),
                ]),
            ),
            ("row_id_note", Floor::Val("行の id は文書内で一意・append-only。この形は folio2 が所有する文書の id 空間の解決（R-4）の範囲で folio2 が自前に持つもので、器の値域（器は「文書内で一意」だけを言う）を写したものではない。文書 id と同じ形（ハイフン可）")),
            ("semantic_check_owner", Floor::Val("scribe2")),
            ("semantic_check_note", Floor::Val("行の id の一意・要件の欄が要件書に実在・節の欄が同じ文書に実在し本文が非空・依存の解決と輪の無さ・検証の欄の形・触る型の閉包が書き込み範囲に収まること、は器（scribe2）の 1 つの関数（編集時・黙って飛ばさない）が持つ。folio2 は持たない（ADR-3 決定 (3)・要件書 scope_m1.not_build）")),
            (
                "folio_check",
                Floor::Strs(&["yaml-form", "derived-diff-zero", "own-id-space"]),
            ),
            ("folio_check_note", Floor::Val("契約表について folio2 が持つ検査は 3 つだけ = 正本の形（重複キー・未知の欄・欄の非空・要件書 FR5 の構造の床）/ 導出物の差分 0（FR11・事後の検出・P-18.2）/ folio2 が所有する文書の id 空間の解決（R-4・母集団は広げない）。このうち導出物の差分 0（derived-diff-zero）は未実装である = 要件書 FR11 の便が入るまで folio はこの検査を回さず、folio build --check もこれを数えない（その間この検査の結果は「まだ分からない」として扱う・P-4.2）")),
        ]),
    ),
    (
        "derived",
        Floor::Map(&[
            ("format", Floor::Val("toml-subset")),
            ("extension", Floor::Val(".toml")),
            ("granularity", Floor::Val("one-file-per-doc")),
            ("head", Floor::Val(EXTERNAL_HEAD)),
            ("array", Floor::Val("contract")),
            (
                "row_fields",
                Floor::Val("external_schema の field の name をそのまま + goal"),
            ),
            (
                "goal",
                Floor::Val(
                    "行の section が指す節の body の逐語を単一行に写す（各行を trim し空行を落とし空白 1 つで繋ぐ・引用符と逆斜線は escape しない）",
                ),
            ),
            ("section_value_shape", Floor::Val("text")),
            ("empty_list", Floor::Val("omit-key")),
            ("empty_list_note", Floor::Val("器の読み手は空の配列を拒む（緩めない）ので、空の一覧は key ごと省いて表す。YAML 正本の側では空の一覧（depends が空 等）を書いてよく、導出器が省く")),
            (
                "value_grammar",
                Floor::Strs(&["text", "list-of-text", "number", "bool"]),
            ),
            (
                "placement",
                Floor::Val(
                    "消費側（器 scribe2）の repo に版管理で置く。path は消費側が宣言する（拡張子 .toml・全文を同じ parser に渡す）",
                ),
            ),
            (
                "check",
                Floor::Map(&[
                    ("command", Floor::Val("folio build --check")),
                    ("verdict_on_diff", Floor::Val("nonzero")),
                    ("stage", Floor::Val("post")),
                ]),
            ),
            ("derived_note", Floor::Val("判断の記録 ADR-3 決定 (4)・要件書 FR11。器は統合先（main）へ着地した後の受付からしか新しい表を読まない（正本の改訂 → 取り込みの要求 → 着地 → 再受付）。契約 file の読み手は共有の scalar の読み手で escape を解かず複数行の値も扱わない（scribe2 contract-source.md §2・実測 2026-09-16）＝goal の単一行化と section を文字列で出す（section_value_shape）のはそのため。導出物を組む口と差分を数える口は未実装である（要件書 FR11 の便で入る・それまで derived の節は決めた形の記録）")),
        ]),
    ),
    (
        "landing",
        Floor::Map(&[
            ("source_of_truth", Floor::Val("scribe2 の記録（record）")),
            (
                "read_port",
                Floor::Val(
                    "統合先の commit（squash）の本文の末尾の印（trailer・契約 id と要件 id・器が書く）",
                ),
            ),
            (
                "trailer_name_source",
                Floor::Map(&[
                    ("owner", Floor::Val("scribe2")),
                    ("derived_from", Floor::Val("器の名前の定数")),
                    ("unreadable", Floor::Val("まだ分からない")),
                ]),
            ),
            (
                "verdict_values",
                Floor::Map(&[
                    ("used", Floor::Strs(&["着地", "まだ分からない"])),
                    ("never", Floor::Strs(&["未着地"])),
                ]),
            ),
            (
                "verdict_cases",
                Floor::Seq(&[
                    Floor::Map(&[
                        ("when", Floor::Val("印が在る")),
                        (
                            "verdict",
                            Floor::Val("着地（印の commit の要約値（sha）を添える）"),
                        ),
                    ]),
                    Floor::Map(&[
                        ("when", Floor::Val("印が無い")),
                        ("verdict", Floor::Val("まだ分からない")),
                    ]),
                    Floor::Map(&[
                        (
                            "when",
                            Floor::Val(
                                "自分の repo への取り込みの要求で終わる形（印も記録も持たない）",
                            ),
                        ),
                        ("verdict", Floor::Val("まだ分からない（恒久）")),
                    ]),
                    Floor::Map(&[
                        ("when", Floor::Val("印の名か commit が読めない")),
                        ("verdict", Floor::Val("まだ分からない")),
                    ]),
                ]),
            ),
            ("forbidden_wording", Floor::Val("未着地")),
            ("landing_note", Floor::Val("要件書 FR13・ADR-3 決定 (5)(6)。着地の判定の語は要件書 FR13 のとおり「着地」と「まだ分からない」の 2 つで、床の検査結果の語（合格・不合格）は使わない（P-3.3・床の合格と紛れさせない）。「未着地」は出さない（印の不在は未着地と弁別できないため・P-4.2）。印の名は器の名前の定数から導出され、folio2 は名を手で持たない（trailer_name_source）。他の repo の要件 id・契約 id・便の id は参照 id の床（R-4）の母集団に入れず、出所付きの測定値として扱う")),
        ]),
    ),
    (
        "index",
        Floor::Map(&[
            ("node_fields_ref", Floor::Val("design-intent/graph.yaml node")),
            ("node_kinds_ref", Floor::Val("design-intent/graph.yaml node_kinds")),
            ("edge_fields_ref", Floor::Val("design-intent/graph.yaml edge")),
            ("edge_types_ref", Floor::Val("design-intent/graph.yaml edge_types")),
            ("index_note", Floor::Val("要件書 FR14。機械が読む id の索引は、設計文書の正本から毎回組み直す導出物として口 folio graph --print が出す（2026-09-22 着地・実装 crates/folio/src/graph.rs・台帳 f2-648.132）。索引は節点（設計文書の中で id を持つ行）と辺（両端の節点の id と型）を持ち、節点と辺の欄も、節点の種類と辺の型の閉じた一覧も、索引の欄の決まり design-intent/graph.yaml が正本として持つ＝この節はその置き場を指すだけで写しを持たない（P-6.3）。判断の記録 ADR-14 決定 (1) のとおり節点は id を持つ行に閉じるので、設計ノートの節と契約表の行は節点にならない。索引の中身そのものは版管理に置かず、中身を席へ届ける経路は器の役割の注入が持つ（要件書 CON9）")),
        ]),
    ),
    (
        "figures",
        Floor::Map(&[
            (
                "spec",
                Floor::Val(
                    "図の道具（archify）の型付き記述（JSON の 5 型の欄の決まりそのまま・ADR-4 決定 (1)）",
                ),
            ),
            ("entry", keys_floor!(FIGURE_ENTRY)),
            (
                "type_enum_ref",
                Floor::Val("design-intent/preview/parts.json figure_type_enum"),
            ),
            (
                "body_classes_ref",
                Floor::Val("design-intent/preview/parts.json figure_body_classes"),
            ),
            ("body_classes_rules_row", Floor::Val("R-3")),
            ("semantic_attrs", Floor::Val("keep")),
            ("viewer_chrome", Floor::Val("discard")),
            ("quality_rules_row", Floor::Val("R-14")),
            ("tool_version_rules_row", Floor::Val("R-15")),
            ("network_commands", Floor::Val("forbid")),
            ("skill_listing", Floor::Val("forbid")),
            ("retry_rules_row", Floor::Val("R-7")),
            ("retry_record", Floor::Val("ledger")),
            ("figures_note", Floor::Val("図の正本は設計ノートの figures 節に型付き記述で置き、別 file にも散文にも持たない（ADR-4 決定 (1)）。生成は要件書 FR15（検査を通らない図は生成しない・前の生成物を上書きしない・凍結 anchor が落ちたら「まだ分からない」・決定 (2)(6)）。図の本体の意味の属性は捨てず（semantic_attrs = keep・決定 (3)）、閲覧の仕掛けは捨て（viewer_chrome = discard・決定 (3)）、意味を表す class は部品目録に載り色・字の大きさ・線の太さは design token で塗る（body_classes・R-3・決定 (3)）。道具の通信する命令は使わず（network_commands = forbid）、AI 向けの説明（skill）として載せない（skill_listing = forbid・R-1 の母集団外・決定 (5)）。修正の往復は R-7 が上限で、往復の記録は台帳に残し撤退条件の測定に使う（retry_record = ledger・決定 (7)）。図の対（持ち主の裁定 2026-09-19・f2-648 notes）＝設計ノートの図は、非エンジニア向けの手順図（専門の言葉を使わず「誰が・どの順で・何をして・だめならどうなるか」）と、エンジニア向けの順序図（命令の名・旗・終了コード・file 名をそのまま）を対で置く。見本は design-note/figures.yaml。これは書き方の指針であり床は数えない")),
        ]),
    ),
    (
        "guards",
        Floor::Map(&[
            ("in_loop", Floor::Strs(&[])),
            (
                "post",
                Floor::Strs(&[
                    "yaml-form",
                    "derived-diff-zero",
                    "own-id-space",
                    "prose-gate",
                    "prose-mentions",
                ]),
            ),
            ("polarity_list_feed", Floor::Val("true")),
            ("p18_4_judged_by", Floor::Val("R-13")),
            ("guards_note", Floor::Val("設計ノートの編集を編集の時点で止める仕掛け（in-loop）は folio2 側に 1 本も無い（器 scribe2 の受付は別 repo の guard で、folio2 の設計ノートの編集を止めない）。この節は極性一覧（P-18.3）へ寄せる材料であり、P-18.4 の判定は folio2 全体を数える rules 行 R-13 の 1 面に委ねる（判定面を 2 つにしない・P-6.3）。post の検査は編集時に止めることの代わりにしない（P-18.2）。post のうち derived-diff-zero は未実装である（要件書 FR11 の便が入るまで、極性一覧へは「まだ無い検査」として寄せる）。prose-mentions は規則の表の行 R-17 の床の歯（2026-09-22 着地・実装 crates/folio/src/mentions.rs・台帳 f2-648.131）で、対象の file の閉じた一覧に design-note/ が在る＝設計ノートの散文の欄に現れた id が、その行の型付きの欄にも相手の行の型付きの欄にも無ければ事後に数える")),
        ]),
    ),
]);

// ── 入口 ──

/// 読めた設計ノート 1 本（file 名・id（file 名の stem）・木）。
struct NoteDoc {
    file: String,
    id: String,
    root: Node,
}

/// 器（scribe2）の導出 file の 1 欄。
struct Field {
    name: String,
    need: String,
    shape: String,
}

/// (a) `<dir>/design-note/` の欄の決まりの写しと設計ノートを検査する。
/// dir が無い = 設計ノート 0 本（違反でも「まだ分からない」でもない）。
pub fn check_note(
    dir: &Path,
    constitution: &Node,
    rules: &Node,
    srs: &Node,
    adr: Option<&Adr>,
    report: &mut Report,
) {
    let nd = dir.join(DIR);
    if !nd.exists() {
        return;
    }
    if nd.is_symlink() || !nd.is_dir() {
        report.unknown(format!(
            "{DIR}/ が dir でない（symlink・file）: {}",
            nd.display()
        ));
        return;
    }
    check_schema_copy(&nd, report);
    let notes = load_notes(&nd, report);
    if notes.is_empty() {
        return;
    }
    // 器の導出 file は契約表の節を持つ設計ノートが 1 本以上あるときだけ読む
    let external = if notes.iter().any(|n| has_contract_table(&n.root)) {
        load_external(dir, report)
    } else {
        None
    };
    // 散文の門の一覧は検査のたびに rules 行 R-16 の value から読む（R-16 の note・P-5.1）
    let gate = prose::gate(rules)
        .inspect_err(|e| report.unknown(format!("rules.yaml: R-16 の value が読めない: {e}")))
        .ok();
    let known = base_known_ids(constitution, rules, srs, adr);
    let requirements = requirement_ids(srs);
    let note_ids: HashSet<&str> = notes.iter().map(|n| n.id.as_str()).collect();
    for note in &notes {
        check_one(
            note,
            &note_ids,
            &known,
            &requirements,
            external.as_deref(),
            gate.as_ref(),
            report,
        );
    }
}

// ── (b) 欄の決まりの写し ──

/// 欄の決まりの file を読み、schema 節（`_note` で終わらない欄）が床の定数と 1 字も違わないことを確かめる。
fn check_schema_copy(nd: &Path, report: &mut Report) {
    let path = nd.join("schema.yaml");
    if path.is_symlink() {
        report.unknown(format!("{SCHEMA_FILE}: symlink は認めない"));
        return;
    }
    if !path.exists() {
        report.unknown(format!("{SCHEMA_FILE}: 欄の決まりが無い"));
        return;
    }
    if !path.is_file() {
        report.unknown(format!("{SCHEMA_FILE}: file でない"));
        return;
    }
    let doc = match read(&path) {
        Ok(d) => d,
        Err(e) => {
            report.unknown(format!("{SCHEMA_FILE}: 読めない: {e}"));
            return;
        }
    };
    if !doc.duplicates.is_empty() {
        for dup in &doc.duplicates {
            report.unknown(format!(
                "{SCHEMA_FILE} {} 行: 読めない（重複キー「{}」）",
                dup.line, dup.key
            ));
        }
        return;
    }
    let Some(schema @ Node::Map(_)) = doc.root.get("schema") else {
        report.unknown(format!(
            "{SCHEMA_FILE}: 形が違う（schema 節が欄の表でない）"
        ));
        return;
    };
    let mut drift = Vec::new();
    floor_diff(&strip_notes(schema), &FLOOR, "", &mut drift);
    for path in drift {
        report.violation(
            KIND,
            format!("{SCHEMA_FILE}: 床の定数と違う: schema.{path}"),
        );
    }
}

// ── (a) 正本の読み ──

fn read(path: &Path) -> Result<yaml::Doc, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    yaml::parse(&text)
}

/// dir の直下の `.yaml`（`schema.yaml` を除く）を名の昇順に読む。
/// symlink・読めない・parse できない・最上位が欄の表でない は「まだ分からない」（他の 6 file と同じ読み手）。
fn load_notes(nd: &Path, report: &mut Report) -> Vec<NoteDoc> {
    let mut names: Vec<String> = match fs::read_dir(nd) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".yaml") && n != "schema.yaml")
            .collect(),
        Err(e) => {
            report.unknown(format!("{DIR}/: 読めない: {e}"));
            return Vec::new();
        }
    };
    names.sort();
    let mut notes = Vec::new();
    for name in names {
        let path = nd.join(&name);
        let file = format!("{DIR}/{name}");
        if path.is_symlink() {
            report.unknown(format!("{file}: symlink は認めない"));
            continue;
        }
        if !path.is_file() {
            report.unknown(format!("{file}: file でない"));
            continue;
        }
        let doc = match read(&path) {
            Ok(d) => d,
            Err(e) => {
                report.unknown(format!("{file}: parse できない: {e}"));
                continue;
            }
        };
        for dup in &doc.duplicates {
            report.violation(
                "重複キー",
                format!(
                    "{file} {} 行: 同じ表にキー「{}」を 2 度書いている",
                    dup.line, dup.key
                ),
            );
        }
        if doc.root.as_map().is_none() {
            report.unknown(format!("{file}: 最上位が欄の表でない"));
            continue;
        }
        notes.push(NoteDoc {
            id: name.trim_end_matches(".yaml").to_string(),
            file: name,
            root: doc.root,
        });
    }
    notes
}

/// 契約表の節を 1 つでも持つか。
fn has_contract_table(root: &Node) -> bool {
    root.get("sections")
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .any(|s| s.get("type").and_then(Node::as_str) == Some(CONTRACT_TABLE))
}

// ── (a) 器の導出 file（行走査で読む） ──

/// `<dir>` の親 dir の `contracts/schema.toml` を読む。読めない・期待する形でない は「まだ分からない」。
fn load_external(dir: &Path, report: &mut Report) -> Option<Vec<Field>> {
    let mut unreadable = |why: String| -> Option<Vec<Field>> {
        report.unknown(format!("{EXTERNAL_PATH}: 器の導出 file が読めない: {why}"));
        None
    };
    let Some(parent) = dir.parent() else {
        return unreadable("正本の置き場の親 dir が無い".to_string());
    };
    let path = parent.join(EXTERNAL_PATH);
    if path.is_symlink() {
        return unreadable("symlink は認めない".to_string());
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return unreadable(e.to_string()),
    };
    let head = format!("[[{EXTERNAL_ROWS_KEY}]]");
    let mut rows: Vec<Vec<(String, String)>> = Vec::new();
    let mut seen_head = false;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if !seen_head {
            if t != EXTERNAL_HEAD {
                return unreadable(format!("先頭が「{EXTERNAL_HEAD}」でない: {t}"));
            }
            seen_head = true;
            continue;
        }
        if t == head {
            rows.push(Vec::new());
            continue;
        }
        if let Some(pair) = quoted_pair(t)
            && let Some(row) = rows.last_mut()
        {
            row.push(pair);
        }
    }
    if !seen_head {
        return unreadable(format!("「{EXTERNAL_HEAD}」の行が無い"));
    }
    if rows.is_empty() {
        return unreadable(format!("{head} の行が無い"));
    }
    let mut fields = Vec::with_capacity(rows.len());
    for row in &rows {
        let value = |key: &str| {
            row.iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .filter(|v| !v.trim().is_empty())
        };
        let (Some(name), Some(need), Some(shape)) = (value("name"), value("need"), value("shape"))
        else {
            return unreadable(format!(
                "{head} の行に {} が揃わない",
                EXTERNAL_ROW_FIELDS.join("・")
            ));
        };
        if !EXTERNAL_NEED.contains(&need.as_str()) {
            return unreadable(format!("欄「{name}」の need「{need}」を知らない"));
        }
        if !EXTERNAL_SHAPE.contains(&shape.as_str()) {
            return unreadable(format!("欄「{name}」の shape「{shape}」を知らない"));
        }
        fields.push(Field { name, need, shape });
    }
    Some(fields)
}

/// `名 = "値"` の行（引用符で囲んだ値だけ・正規表現は使わない）。
fn quoted_pair(line: &str) -> Option<(String, String)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let rest = rest.trim();
    let body = rest.strip_prefix('"')?.strip_suffix('"')?;
    if key.is_empty() || body.contains('"') {
        return None;
    }
    Some((key.to_string(), body.to_string()))
}

// ── id 空間 ──

/// rules 行の節（便 1 と同じ）。
const RULE_SECTIONS: [&str; 2] = ["thresholds", "discipline"];

/// 要件書の id を持つ節（便 1 と同じ）。
const SRS_ID_SECTIONS: [&str; 7] = [
    "goals",
    "requirements",
    "nonfunctional",
    "acceptance",
    "constraints",
    "actors",
    "outputs",
];

/// 節の行（表）。一覧でない節・表でない行は数えない（便 0・便 1 の側が数えてある）。
fn maps<'a>(root: &'a Node, section: &str) -> Vec<&'a Node> {
    root.get(section)
        .and_then(Node::as_seq)
        .unwrap_or_default()
        .iter()
        .filter(|n| n.as_map().is_some())
        .collect()
}

/// 既知の id = 条 id と規範文 id・rules 行 id・要件書の 7 節の id（便 1 の `refs.rs`）+ 判断の記録の id（便 6 の `link.rs`）。
/// 読めない形の申告は便 1・便 5 の検査が数えてあるので、ここでは捨てる。
fn base_known_ids(
    constitution: &Node,
    rules: &Node,
    srs: &Node,
    adr: Option<&Adr>,
) -> HashSet<String> {
    let articles = maps(constitution, "articles");
    let rule_rows: Vec<&Node> = RULE_SECTIONS.iter().flat_map(|s| maps(rules, s)).collect();
    let mut known = refs::known_ids(&articles, &rule_rows, srs, &mut Report::default());
    if let Some(adr) = adr {
        known.extend(link::adr_ids(adr).iter().map(|id| (*id).to_string()));
    }
    known
}

/// 要件書の id（契約表の行の req の解決先）。
fn requirement_ids(srs: &Node) -> HashSet<String> {
    let mut ids = HashSet::new();
    for section in SRS_ID_SECTIONS {
        for row in maps(srs, section) {
            if let Some(id) = row.get("id").and_then(Node::as_str) {
                ids.insert(id.to_string());
            }
        }
    }
    ids
}

// ── (c) 設計ノート 1 本 ──

#[allow(clippy::too_many_arguments)]
fn check_one(
    note: &NoteDoc,
    note_ids: &HashSet<&str>,
    base: &HashSet<String>,
    requirements: &HashSet<String>,
    external: Option<&[Field]>,
    gate: Option<&prose::Gate>,
    report: &mut Report,
) {
    let file = format!("{DIR}/{}", note.file);
    let root = &note.root;

    for key in DOC.required {
        if root.get(key).is_none() {
            report.violation(KIND, format!("{file}: 節「{key}」が無い"));
        }
    }
    unknown_sections(&file, root, &DOC.all(), report);

    check_meta(&file, note, note_ids, report);

    // 参照 id の母集団に、この文書の契約表の行 id（裸の形と「<doc id>#<row id>」の形）を足す
    let sections = row_list(&file, "sections", root, "sections", report);
    let mut known = base.clone();
    for section in &sections {
        if section.get("type").and_then(Node::as_str) != Some(CONTRACT_TABLE) {
            continue;
        }
        for row in section
            .get("rows")
            .and_then(Node::as_seq)
            .unwrap_or_default()
        {
            let id = row_id(row);
            if id != "?" {
                known.insert(format!("{}#{id}", note.id));
                known.insert(id);
            }
        }
    }

    // prose の節の番号（契約表の行の section の解決先）
    let prose_ns: HashSet<&str> = sections
        .iter()
        .filter(|s| s.get("type").and_then(Node::as_str) == Some(PROSE))
        .filter_map(|s| s.get("n").and_then(Node::as_str))
        .collect();

    let mut previous: Option<u64> = None;
    for section in &sections {
        check_section(
            &file,
            section,
            &mut previous,
            &known,
            requirements,
            &prose_ns,
            external,
            gate,
            report,
        );
    }

    check_figures(&file, root, &known, report);
}

/// meta の欄。
fn check_meta(file: &str, note: &NoteDoc, note_ids: &HashSet<&str>, report: &mut Report) {
    let blank = Node::Null;
    let meta = note.root.get("meta").unwrap_or(&blank);
    non_empty(file, "meta", meta, DOC_META.required, report);
    unknown_fields(file, "meta", meta, &DOC_META, report);

    if let Some(id) = field(meta, "id") {
        if id != note.id {
            report.violation(
                KIND,
                format!(
                    "{file}: meta.id「{id}」が file 名の stem「{}」と違う",
                    note.id
                ),
            );
        }
        if !is_lower_id(id) {
            report.violation(
                KIND,
                format!("{file}: meta.id「{id}」が形 {ID_PATTERN} でない"),
            );
        }
    }
    if let Some(v) = field(meta, "version")
        && !is_version(v)
    {
        report.violation(
            KIND,
            format!("{file}: meta.version「{v}」が形 {VERSION_PATTERN} でない"),
        );
    }
    if let Some(v) = field(meta, "generated")
        && !is_date(v)
    {
        report.violation(
            KIND,
            format!("{file}: meta.generated「{v}」が年-月-日でない"),
        );
    }
    if let Some(v) = field(meta, "profile")
        && !crate::parts::catalog::PROFILES.contains(&v)
    {
        report.violation(KIND, format!("{file}: meta.profile「{v}」が一覧に無い"));
    }
    let status = field(meta, "status");
    if let Some(v) = status
        && !STATUS_ENUM.contains(&v)
    {
        report.violation(KIND, format!("{file}: meta.status「{v}」が一覧に無い"));
    }

    let approval = row_list(file, "meta の approval", meta, "approval", report);
    let effective = status.is_some_and(|v| EFFECTIVE_STATUS.contains(&v));
    if effective && approval.is_empty() {
        report.violation(
            KIND,
            format!(
                "{file}: meta: status {} なのに approval（承認欄）が無い",
                status.unwrap_or("")
            ),
        );
    }
    if status == Some(STATUS_EXAMPLE) && !approval.is_empty() {
        report.violation(
            KIND,
            format!("{file}: meta: status {STATUS_EXAMPLE} は approval（承認欄）を持たない"),
        );
    }
    for (i, row) in approval.iter().enumerate() {
        let at = format!("meta の approval[{i}]");
        non_empty(file, &at, row, APPROVAL_REQUIRED, report);
        if let Some(v) = field(row, "date")
            && !is_date(v)
        {
            report.violation(KIND, format!("{file}: {at}: date「{v}」が年-月-日でない"));
        }
        if let Some(v) = field(row, "surface")
            && !SURFACE_ENUM.contains(&v)
        {
            report.violation(
                KIND,
                format!(
                    "{file}: {at}: surface「{v}」が一覧に無い（対話面は rules 行の id で指す・R-8）"
                ),
            );
        }
    }

    for key in ["supersedes", "superseded_by"] {
        if let Some(v) = field(meta, key)
            && !(note_ids.contains(v) && v != note.id)
        {
            report.violation(
                KIND,
                format!("{file}: meta.{key}「{v}」の設計ノートが実在しない"),
            );
        }
    }
    if status == Some(STATUS_RETIRED) && field(meta, "superseded_by").is_none() {
        report.violation(
            KIND,
            format!("{file}: meta: {STATUS_RETIRED} なのに superseded_by（後継）が無い（P-7.2）"),
        );
    }
}

/// 節 1 つ（番号・型・型ごとの行）。
#[allow(clippy::too_many_arguments)]
fn check_section(
    file: &str,
    section: &Node,
    previous: &mut Option<u64>,
    known: &HashSet<String>,
    requirements: &HashSet<String>,
    prose_ns: &HashSet<&str>,
    external: Option<&[Field]>,
    gate: Option<&prose::Gate>,
    report: &mut Report,
) {
    let shown = section.get("n").and_then(Node::as_str).unwrap_or("?");
    let at = format!("§{shown}");
    non_empty(file, &at, section, SECTION.required, report);
    // 節の欄は SECTION の和集合で見る（型ごとの required / forbid は下の by_type が別に数える）
    unknown_fields(file, &at, section, &SECTION, report);

    match shown.parse::<u64>() {
        Ok(n) if n >= 1 => {
            if previous.is_some_and(|p| n <= p) {
                report.violation(
                    KIND,
                    format!(
                        "{file}: {at}: n が前の節（{}）より大きくない（同じ・戻るは認めない）",
                        previous.unwrap_or(0)
                    ),
                );
            }
            *previous = Some(n);
        }
        _ => report.violation(
            KIND,
            format!("{file}: {at}: n「{shown}」が 1 以上の整数でない"),
        ),
    }

    let Some(ty) = field(section, "type") else {
        return; // 欄の非空が数えてある
    };
    if !TYPE_ENUM.contains(&ty) {
        report.violation(KIND, format!("{file}: {at}: 節の型「{ty}」が一覧に無い"));
        return;
    }
    for key in if ty == PROSE { NEEDS_BODY } else { NEEDS_ROWS } {
        if section.get(key).is_none() {
            report.violation(
                KIND,
                format!("{file}: {at}: 節の型 {ty} の欄「{key}」が無い"),
            );
        }
    }
    if ty == PROSE {
        for key in FORBIDS_ROWS {
            if section.get(key).is_some() {
                report.violation(
                    KIND,
                    format!("{file}: {at}: 節の型 {ty} に置けない欄「{key}」が在る"),
                );
            }
        }
        // 散文の門（FR12・R-16）。印を持つ文の参照 id は同じ母集団で解く
        if let (Some(gate), Some(body)) = (gate, field(section, "body")) {
            for m in prose::scan(body, gate) {
                if let Some(why) = m.reason {
                    report.violation(
                        PROSE_GATE,
                        format!("{file}: 節 {shown} 行 {}: {why} {}", m.line, m.head),
                    );
                }
                for id in m.pointers.iter().filter(|id| !known.contains(*id)) {
                    report.violation(
                        KIND,
                        format!("{file}: {at}: 散文の参照 id「{id}」が実在しない"),
                    );
                }
            }
        }
        return;
    }

    let rows = row_list(file, &format!("{at} の rows"), section, "rows", report);
    if ty == CONTRACT_TABLE {
        check_contract_rows(file, &at, &rows, requirements, prose_ns, external, report);
    } else {
        check_table_rows(file, &at, ty, &rows, known, report);
    }
    duplicate_ids(file, rows, report);
}

/// 部品・口・欄・歯の表の行（行の欄の集合と値域・参照 id）。
fn check_table_rows(
    file: &str,
    at: &str,
    ty: &str,
    rows: &[&Node],
    known: &HashSet<String>,
    report: &mut Report,
) {
    let keys = match ty {
        "parts-table" => &PARTS_ROW,
        "ports-table" => &PORTS_ROW,
        "fields-table" => &FIELDS_ROW,
        _ => &TEETH_ROW,
    };
    for row in rows {
        let rat = format!("{at} の行 {}", row_id(row));
        non_empty(file, &rat, row, keys.required, report);
        for (key, _) in row.as_map().unwrap_or_default() {
            if !keys.has(key) {
                report.violation(
                    KIND,
                    format!("{file}: {rat}: 行の欄「{key}」が欄の決まりに無い"),
                );
            }
        }
        if ty == "fields-table" {
            if let Some(v) = field(row, "need")
                && !NEED_ENUM.contains(&v)
            {
                report.violation(KIND, format!("{file}: {rat}: need「{v}」が一覧に無い"));
            }
            if let Some(v) = field(row, "shape")
                && !SHAPE_ENUM.contains(&v)
            {
                report.violation(KIND, format!("{file}: {rat}: shape「{v}」が一覧に無い"));
            }
        }
        resolve_ids(file, &rat, row, "ref", known, report);
    }
}

/// 契約表の行（欄は器の導出 file が決める・id と参照は folio2 が持つ）。
fn check_contract_rows(
    file: &str,
    at: &str,
    rows: &[&Node],
    requirements: &HashSet<String>,
    prose_ns: &HashSet<&str>,
    external: Option<&[Field]>,
    report: &mut Report,
) {
    let row_ids: HashSet<String> = rows
        .iter()
        .map(|row| row_id(row))
        .filter(|id| id != "?")
        .collect();
    for row in rows {
        let id = row_id(row);
        let rat = format!("{at} の行 {id}");
        if let Some(fields) = external {
            let required: Vec<&str> = fields
                .iter()
                .filter(|f| f.need == "required")
                .map(|f| f.name.as_str())
                .collect();
            non_empty(file, &rat, row, &required, report);
            for (key, _) in row.as_map().unwrap_or_default() {
                if !fields.iter().any(|f| &f.name == key) {
                    report.violation(
                        KIND,
                        format!("{file}: {rat}: 契約表の欄「{key}」が器の導出 file に無い"),
                    );
                }
            }
            for f in fields {
                let Some(value) = row.get(&f.name) else {
                    continue;
                };
                let ok = match f.shape.as_str() {
                    "text" => matches!(value, Node::Null | Node::Scalar(_)),
                    _ => match value {
                        Node::Null => true,
                        Node::Seq(items) => items.iter().all(|x| x.as_str().is_some()),
                        _ => false,
                    },
                };
                if !ok {
                    report.violation(
                        KIND,
                        format!(
                            "{file}: {rat}: 欄「{}」が {} の形でない",
                            f.name,
                            if f.shape == "text" {
                                "text（文字列）"
                            } else {
                                "list（文字列の一覧）"
                            }
                        ),
                    );
                }
            }
        }
        if id != "?" && !is_lower_id(&id) {
            report.violation(
                KIND,
                format!("{file}: {rat}: 行 id「{id}」が形 {ROW_ID_PATTERN} でない"),
            );
        }
        if let Some(v) = field(row, "section")
            && !prose_ns.contains(v)
        {
            report.violation(
                KIND,
                format!("{file}: {rat}: section「{v}」が同じ文書の {PROSE} の節の n でない"),
            );
        }
        resolve_ids(file, &rat, row, "req", requirements, report);
        resolve_ids(file, &rat, row, "depends", &row_ids, report);
    }
}

// ── (c) 図 ──

fn check_figures(file: &str, root: &Node, known: &HashSet<String>, report: &mut Report) {
    for entry in row_list(file, "figures", root, "figures", report) {
        let at = format!("figures の {}", row_id(entry));
        non_empty(file, &at, entry, FIGURE_ENTRY.required, report);
        unknown_fields(file, &at, entry, &FIGURE_ENTRY, report);
        if let Some(v) = field(entry, "type")
            && FigureType::from_name(v).is_none()
        {
            report.violation(
                KIND,
                format!("{file}: {at}: 図の型「{v}」が部品目録の一覧に無い"),
            );
        }
        if let Some(spec) = entry.get("spec")
            && !matches!(spec, Node::Map(_))
        {
            report.violation(KIND, format!("{file}: {at}: spec が表でない"));
        }
        resolve_ids(file, &at, entry, "refs", known, report);
    }
}

// ── 小さな読み手 ──

/// 行の一覧。無い・null は 0 行、表の一覧でなければ「まだ分からない」。
fn row_list<'a>(
    file: &str,
    place: &str,
    node: &'a Node,
    key: &str,
    report: &mut Report,
) -> Vec<&'a Node> {
    match node.get(key) {
        None | Some(Node::Null) => Vec::new(),
        Some(Node::Seq(items)) if items.iter().all(|n| n.as_map().is_some()) => {
            items.iter().collect()
        }
        Some(_) => {
            report.unknown(format!("{file}: {place} が表の一覧でない"));
            Vec::new()
        }
    }
}

/// 欄の表の未知の欄（欄の決まりの required と optional の和集合に無い鍵）を 1 件ずつ数える
/// （要件書 FR9 の正本の形・要件書の図の行の検査と同じ字面）。表でなければ 0 件（形の側が数えてある）。
fn unknown_fields(file: &str, at: &str, node: &Node, keys: &Keys, report: &mut Report) {
    for (key, _) in node.as_map().unwrap_or_default() {
        if !keys.has(key) {
            report.violation(UNKNOWN_FIELD, format!("{file}: {at} の未知の欄「{key}」"));
        }
    }
}

/// 値を持つ文字列の欄（無い・null・空白だけは None＝欄の非空の側が数える）。
fn field<'a>(node: &'a Node, key: &str) -> Option<&'a str> {
    node.get(key)
        .and_then(Node::as_str)
        .filter(|s| !s.trim().is_empty())
}

/// 一覧の欄の各要素が既知の id に解けるか。
fn resolve_ids(
    file: &str,
    at: &str,
    node: &Node,
    key: &str,
    known: &HashSet<String>,
    report: &mut Report,
) {
    match node.get(key) {
        None | Some(Node::Null) => {}
        Some(Node::Seq(items)) => {
            for (i, item) in items.iter().enumerate() {
                match item.as_str() {
                    Some(v) if known.contains(v) => {}
                    Some(v) => report.violation(
                        KIND,
                        format!("{file}: {at} の {key}[{i}]: id「{v}」が実在しない"),
                    ),
                    None => {
                        report.unknown(format!("{file}: {at} の {key}[{i}] が文字列でない"));
                    }
                }
            }
        }
        Some(_) => report.unknown(format!("{file}: {at} の {key} が一覧でない")),
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

/// 英小文字で始まり、英小文字・数字・「-」だけが続く（id_pattern / row_id.pattern）。
fn is_lower_id(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// 「v」+ 数字列 + 「.」+ 数字列（version_pattern）。
fn is_version(s: &str) -> bool {
    s.strip_prefix('v')
        .and_then(|rest| rest.split_once('.'))
        .is_some_and(|(major, minor)| digits(major) && digits(minor))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 床の木の導出は凍結 anchor（設計判断の席が独立の実装で組んだ・P-10.1）と byte 一致（便 57 §1 (c)3・
    /// `ceiling.rs` / `rules.rs` の同名の歯と同じ形で `schema.rs` の pub の `derive` を呼ぶ）。
    #[test]
    fn note_floor_derives_the_frozen_anchor_byte_for_byte() {
        let anchor = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/schema/note-region.txt"
        ))
        .unwrap();
        assert_eq!(crate::schema::derive(&FLOOR), anchor);
    }
}
