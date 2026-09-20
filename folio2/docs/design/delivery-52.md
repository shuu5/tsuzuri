# 設計: 便 52 — 部品目録の上限 3 本と図の型の名札を組み立て時の導出に足し、手書きの写し 4 本と、値を自分と比べるだけになる歯 3 本を無くす（ADR-11 決定 (4)③・FR4 / NFR2）

- 要件: FR4（面は正本から生成する）/ NFR2（部品と型の閉じた一覧は部品目録から）
- 条: P-2.4 / P-5.1 / P-6.3・P-6.4 / P-4.2 / P-10.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(ア) と (4)③。rules 行 D-11 の作法 = 本便の根拠は ADR-11 決定 (4)③。値は 1 つも変えない。
- 位置: 便 49（憲法の値域の導出の土台）と便 50（面の表の鍵の列）の後。便 51（規則の表の欄の決まりの節）とは独立。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ba が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

部品目録 design-intent/preview/parts.json は、部品と図の型の閉じた一覧の正本で、crates/folio/build.rs が組み立て時に型 4 つ（Component・FigureType・ShelfType・StyleProp）を OUT_DIR の parts_catalog.rs へ導出している（便 13）。ところが同じ部品目録が持つ値のうち 4 つは、面の生成器に手書きの写しが残っている = crates/folio/src/face.rs の定数 MAX_RAIL_NODES（7・部品目録の components.pipeline-rail.max_nodes）・MAX_STATE_NODES（4・components.state-strip.max_nodes）・MAX_PER_BAND（4・components.context-band.max_per_band）・FIGURE_LABELS（図の型 5 つの名札の対・部品目録の figure_body_classes.type_ids と同じ字面と順）。一致を見ているのは src の unit test 3 本だけで、歯を回さないと食い違いが出ない。ADR-11 決定 (3)(ア) により部品目録は file が正本で、実装は手書きの写しを持たない。本便はこの 4 つを組み立て時の導出に足す。値・面の出力の byte・Err の文言・部品目録は変えない。

設計判断の席の実測（2026-09-20・main c4529fd = 便 49 の着地後・build.rs は正規化 332 行で、便 49 が憲法の値域の純粋な関数 constitution_enums と、tests/constitution_enums.rs から build.rs を path の属性で取り込む形を入れた）: 部品目録の components のうち、鍵が max_ で始まる欄を持つ部品はこの 3 つだけで、値は全部 整数。figure_body_classes.type_ids は 5 つの対の表で、鍵は 5 つとも figure_type_enum に在る（figure_type_enum は 8 値 = 面の部品の図 3 つと図の道具の 5 型）。上限の定数を使う所は face_constitution.rs 1 か所・face_srs.rs 3 か所（超えたら Err・文言に上限の数と部品目録の欄の名が入る）。FIGURE_LABELS を使う所は face.rs の 1 か所（型から名札を引く・無ければ Err「図の型「…」は図の道具の型でない」）。実行時の「部品目録と組み立て時の写しの一致」は crates/folio/src/parts.rs の関数 catalog_matches が見ていて（違えば「まだ分からない」で文言は「parts.json: 組み立て時の部品目録と違う（組み立て直す）」）、いまは型 4 つぶんだけを比べる。値を部品目録と比べている歯は 3 本 = face_constitution.rs の face_rail_node_limit_matches_the_parts_catalog・face_srs.rs の face_srs_limits_match_the_parts_catalog・face_note.rs の face_note_figure_labels_match_the_parts_catalog_type_ids。

(a) build.rs の部品目録の導出に足す。
1. 上限: components の各部品について、鍵が max_ で始まる欄を file の順に全部拾い、部品ごとに定数 1 つずつを書く。定数の名は部品の名と欄の名を大文字にして「_」で繋ぐ（pipeline-rail の max_nodes は PIPELINE_RAIL_MAX_NODES・state-strip の max_nodes は STATE_STRIP_MAX_NODES・context-band の max_per_band は CONTEXT_BAND_MAX_PER_BAND）。型は usize。どの部品のどの欄かを build.rs に書き並べない（一覧の写しを作らない）。値が 0 以上の整数でなければ組み立ての失敗。
2. 図の型の名札: figure_body_classes.type_ids の対を file の順に定数 1 つ（名は FIGURE_TYPE_LABELS・型の字面と名札の対の列）として書く。表でない・値が文字列でない・鍵が figure_type_enum に無い、は組み立ての失敗。
3. 便 49 と同じく、部品目録の文字列を受けて source の文字列か理由の文を返す純粋な関数の形にする（名は parts_catalog・今の関数 derive が file を読んでからこれを呼ぶ形でよい）。今の出力（型 4 つ）の字面は変えない。

(b) face.rs の 4 つを導出した定数に替える。MAX_RAIL_NODES・MAX_STATE_NODES・MAX_PER_BAND は名を残し、右辺を導出した定数にする（使う所と Err の文言は変わらない）。FIGURE_LABELS は手書きの 5 対をやめ、導出した FIGURE_TYPE_LABELS を指す。節の見出しの注「歯が parts.json と同値を確かめる」は「組み立て時に部品目録から導出する」に直す。

(c) 実行時の一致を広げる。parts.rs の catalog_matches は、型 4 つに加えて、上限（部品ごとの max_ で始まる欄の名と値）と図の型の名札（type_ids の対と順）も、読んでいる置き場の部品目録と組み立て時の写しで比べる。違えば今と同じ「まだ分からない」と同じ文言。比べるために要る「部品の名・欄の名・値」の対の列は (a) の導出に定数として足してよい（名は LIMITS）。

(d) 歯。新しい歯の関数名は parts で始める。
1. 消す歯 3 本（根拠つき）: 上に挙げた 3 本は、導出の後は「部品目録から導出した値」を「部品目録の値」と比べるだけになり、何も落とせなくなるので消す（face_srs.rs の歯のうち band_limit の 2 行は上限の関数の歯なので、別の歯として残す）。
2. src/parts.rs の unit test（凍結の針・P-10.1）: 導出した上限 3 つが 7・4・4 で、図の型の名札が今の 5 対（構成図（architecture）・手順図（workflow）・順序図（sequence）・流れ図（dataflow）・状態図（lifecycle））と順まで同じ（字面を歯に直に書く）。
3. crates/folio/tests/parts.rs（build.rs を path の属性で取り込み、(a) の純粋な関数を直に呼ぶ）: 実の部品目録 → Ok ∧ 出力に PIPELINE_RAIL_MAX_NODES と FIGURE_TYPE_LABELS が在る。変異 3 つがそれぞれ Err = max_nodes を文字列にする・type_ids の鍵を figure_type_enum に無い名にする・type_ids を一覧にする。
4. 同じ file: 実の設計文書の置き場の写しで、部品目録の pipeline-rail の max_nodes を 8 にして folio parts --check → 「まだ分からない」（終了 2）∧「組み立て時の部品目録と違う」。type_ids の名札の 1 字を変えても同じ。
5. 回帰（期待不変・共通の検証が回す）: tests/parts.rs の既存の歯・面の凍結の fixture と byte 一致の歯の全部・tests/figure.rs・tests/floor_cases.rs（凍結の場合 134 件）・tests/constitution_enums.rs。

(e) 大きさと接続。新規 file は無い。既存 = build.rs（約 +70 行）・src/parts.rs（約 +50 行）・src/face.rs（減る）・src/face_constitution.rs と src/face_srs.rs と src/face_note.rs（歯を消すので減る）・tests/parts.rs（約 +90 行）。size S = 既存 file 1 本あたりの増分は 100 行に収まる見積。部品目録・設計文書・fixture・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: build.rs の上限と図の型の名札の導出・face.rs の 4 つの置き換え・実行時の一致の拡張・歯。
- 入れない: 部品目録の値の変更・図の型の一覧 FIGURE_TYPES（src/figure.rs・図の道具の側の表は別の話 = 下調べ A′-2）・入口の棚の id（決定 (4)④）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| derive | 導出 | build.rs の上限の定数・図の型の名札・LIMITS・純粋な関数 parts_catalog |
| swap | 置き換え | face.rs の 4 つ |
| match | 一致 | parts.rs の catalog_matches を上限と名札へ |
| teeth | 歯 | 消す 3 本・凍結の針・変異 3 つ・実行時のずれ 2 つ |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ba"
title = "部品目録の上限 3 本と図の型の名札を build.rs の組み立て時の導出に足し（どの欄かを書き並べない・純粋な関数 parts_catalog）、face.rs の手書きの写し 4 本を導出した定数に替え、実行時の部品目録との一致を上限と名札へ広げる（値・面の出力・Err の文言は不変・値を自分と比べるだけになる歯 3 本は消す・ADR-11 決定 (4)③）"
req = ["FR4", "NFR2"]
section = "1"
write-set = ["crates/folio/build.rs", "crates/folio/src/parts.rs", "crates/folio/src/face.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_note.rs", "crates/folio/tests/parts.rs"]
verify = ["cargo nextest run -p folio parts", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "parts の歯（導出の Ok と変異 3 つの Err・凍結の針 = 上限 7・4・4 と名札 5 対・部品目録の上限か名札を変えた写しで parts --check が「まだ分からない」・既存は期待不変）が緑、face.rs に上限の数と図の型の名札の手書きが残らず、共通の検証（面の凍結の fixture と byte 一致・figure・floor_cases 134 件）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
