//! 設計ノートの床の定数（便 114・docs/design/delivery-114.md §1・ADR-15 決定 (2)(3)・責務の層 1 読む）。`note.rs` から
//! 欄の集合の型 `Keys` とその 2 つの method・欄の決まりの定数・床の木 `FLOOR` を字を変えずに降ろした。検査の本体と
//! 置き場と違反の種別は `note.rs` に残る。見え方は `note.rs` の検査が読む定数と欄の集合の型・必須の欄・2 つの method だけを広げた。
//! 読み手は `note.rs` の検査・`schema.rs`（欄の決まりの生成区間の導出）・`derive.rs`（導出物の命令・便 119）。

use crate::floor::{Floor, keys_floor};

// ── 床の定数（design-note/schema.yaml の schema 節の正本） ──

/// 欄の集合（required / optional）。
pub(crate) struct Keys {
    pub(crate) required: &'static [&'static str],
    optional: &'static [&'static str],
}

impl Keys {
    pub(crate) fn has(&self, key: &str) -> bool {
        self.required.contains(&key) || self.optional.contains(&key)
    }

    pub(crate) fn all(&self) -> Vec<&'static str> {
        self.required
            .iter()
            .chain(self.optional.iter())
            .copied()
            .collect()
    }
}

const PATH_BASE: &str = "repo-root";
const DATE_FORMAT: &str = r"^\d{4}-\d{2}-\d{2}$";
pub(crate) const DOC: Keys = Keys {
    required: &["meta", "sections"],
    optional: &["figures", "sources"],
};
pub(crate) const DOC_META: Keys = Keys {
    required: &["id", "title", "version", "status", "generated", "profile"],
    optional: &["approval", "supersedes", "superseded_by", "note"],
};
pub(crate) const ID_PATTERN: &str = "^[a-z][a-z0-9-]*$";
pub(crate) const VERSION_PATTERN: &str = r"^v[0-9]+\.[0-9]+$";
pub(crate) const STATUS_ENUM: &[&str] = &["draft", "effective", "retired", "example"];
pub(crate) const EFFECTIVE_STATUS: &[&str] = &["effective", "retired"];
pub(crate) const STATUS_EXAMPLE: &str = "example";
pub(crate) const STATUS_RETIRED: &str = "retired";
pub(crate) const APPROVAL_REQUIRED: &[&str] = &["who", "date", "ruling", "verbatim", "surface"];
pub(crate) const SURFACE_ENUM: &[&str] = &["R-8"];
pub(crate) const SECTION: Keys = Keys {
    required: &["n", "type", "title"],
    optional: &["body", "rows", "note"],
};
pub(crate) const TYPE_ENUM: &[&str] = &[
    "prose",
    "parts-table",
    "ports-table",
    "fields-table",
    "teeth-table",
    "contract-table",
];
pub(crate) const PROSE: &str = "prose";
pub(crate) const CONTRACT_TABLE: &str = "contract-table";
/// 節の型ごとの required / forbid（by_type）。
pub(crate) const NEEDS_BODY: &[&str] = &["body"];
pub(crate) const NEEDS_ROWS: &[&str] = &["rows"];
pub(crate) const FORBIDS_ROWS: &[&str] = &["rows"];
pub(crate) const PARTS_ROW: Keys = Keys {
    required: &["id", "name", "role"],
    optional: &["ref", "note"],
};
pub(crate) const PORTS_ROW: Keys = Keys {
    required: &["id", "name", "input", "output", "refuses"],
    optional: &["ref", "note"],
};
const REFUSES_NONE_MARKER: &str = "なし";
pub(crate) const FIELDS_ROW: Keys = Keys {
    required: &["id", "name", "need", "shape"],
    optional: &["enum", "note"],
};
pub(crate) const NEED_ENUM: &[&str] = &["required", "optional"];
pub(crate) const SHAPE_ENUM: &[&str] = &["text", "list", "number", "bool", "table"];
pub(crate) const TEETH_ROW: Keys = Keys {
    required: &["id", "name", "red_when", "fixture"],
    optional: &["ref", "note"],
};
/// 器（scribe2）の導出 file の置き場（repo の根からの相対）と読み手の期待する形。
pub(crate) const EXTERNAL_PATH: &str = "contracts/schema.toml";
pub(crate) const EXTERNAL_HEAD: &str = "schema = 1";
pub(crate) const EXTERNAL_ROWS_KEY: &str = "field";
pub(crate) const EXTERNAL_ROW_FIELDS: &[&str] = &["name", "need", "shape"];
/// 導出 file の need / shape の値域（読めない値は「まだ分からない」）。
pub(crate) const EXTERNAL_NEED: &[&str] = &["required", "optional"];
pub(crate) const EXTERNAL_SHAPE: &[&str] = &["text", "list"];
pub(crate) const ROW_ID_PATTERN: &str = "^[a-z][a-z0-9-]*$";
pub(crate) const FIGURE_ENTRY: Keys = Keys {
    required: &["id", "type", "caption", "spec"],
    optional: &["refs", "note"],
};
/// 導出物の拡張子と配列の名（derived の節・導出の命令 `derive.rs` と写しが同じ定数を引く・便 119）。
pub(crate) const DERIVED_EXTENSION: &str = ".toml";
pub(crate) const DERIVED_ARRAY: &str = "contract";

/// 導出物の命令の名の字（命令の口 `main.rs` の clap の name と写しの command が同じ字を引く・便 119）。
macro_rules! derived_subcommand {
    () => {
        "derive"
    };
}
pub(crate) const DERIVED_SUBCOMMAND: &str = derived_subcommand!();
/// 導出物の差分を数える命令（derived の節の check の command）。
pub(crate) const DERIVED_CHECK_COMMAND: &str = concat!("folio ", derived_subcommand!(), " --check");

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
            ("profile_enum", Floor::Strs(crate::catalog::PROFILES)),
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
            ("folio_check_note", Floor::Val("契約表について folio2 が持つ検査は 3 つだけ = 正本の形（重複キー・未知の欄・欄の非空・要件書 FR5 の構造の床）/ 導出物の差分 0（FR11・事後の検出・P-18.2）/ folio2 が所有する文書の id 空間の解決（R-4・母集団は広げない）。このうち導出物の差分 0（derived-diff-zero）は、配信の組み立て（folio build）から切り離した独立の命令 folio derive --check が数える（便 119・判断の記録 ADR-16 決定 (5)）。folio check と folio build はこの検査を回さない")),
        ]),
    ),
    (
        "derived",
        Floor::Map(&[
            ("format", Floor::Val("toml-subset")),
            ("extension", Floor::Val(DERIVED_EXTENSION)),
            ("granularity", Floor::Val("one-file-per-doc")),
            ("head", Floor::Val(EXTERNAL_HEAD)),
            ("array", Floor::Val(DERIVED_ARRAY)),
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
                    ("command", Floor::Val(DERIVED_CHECK_COMMAND)),
                    ("verdict_on_diff", Floor::Val("nonzero")),
                    ("stage", Floor::Val("post")),
                ]),
            ),
            ("derived_note", Floor::Val("判断の記録 ADR-3 決定 (4)・要件書 FR11。器は統合先（main）へ着地した後の受付からしか新しい表を読まない（正本の改訂 → 取り込みの要求 → 着地 → 再受付）。契約 file の読み手は共有の scalar の読み手で escape を解かず複数行の値も扱わない（scribe2 contract-source.md §2・実測 2026-09-16）＝goal の単一行化と section を文字列で出す（section_value_shape）のはそのため。導出物を組む口（folio derive --write）と差分を数える口（folio derive --check）は便 119 で入った。どちらも面の生成器も様式の file も呼ばず、導出物の置き場（--out）は消費側が宣言する（既定なし・判断の記録 ADR-16 決定 (5)）。値に二重引用符・逆斜線・改行が在る行は、escape しない形では書けないので導出せず「まだ分からない」とする")),
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
            ("guards_note", Floor::Val("設計ノートの編集を編集の時点で止める仕掛け（in-loop）は folio2 側に 1 本も無い（器 scribe2 の受付は別 repo の guard で、folio2 の設計ノートの編集を止めない）。この節は極性一覧（P-18.3）へ寄せる材料であり、P-18.4 の判定は folio2 全体を数える rules 行 R-13 の 1 面に委ねる（判定面を 2 つにしない・P-6.3）。post の検査は編集時に止めることの代わりにしない（P-18.2）。post のうち derived-diff-zero は folio derive --check が数える（便 119・事後の検出で、編集の時点で止める仕掛けの代わりにしない）。prose-mentions は規則の表の行 R-17 の床の歯（2026-09-22 着地・実装 crates/folio/src/mentions.rs・台帳 f2-648.131）で、対象の file の閉じた一覧に design-note/ が在る＝設計ノートの散文の欄に現れた id が、その行の型付きの欄にも相手の行の型付きの欄にも無ければ事後に数える")),
        ]),
    ),
]);
