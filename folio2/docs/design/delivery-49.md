# 設計: 便 49 — 憲法の値域 10 本を組み立て時に型へ導出し、床の側の手書きの写し 2 本（判断の記録の床の撤退条件の種類・注入の強度）を導出した名の列に替える（ADR-11 決定 (4)② の 1 本目・FR5 / FR6 / FR19）

- 要件: FR5（床）/ FR6（注入）/ FR19（判断の記録の欄の決まりの生成区間は byte 不変）
- 条: P-5.1・P-5.6 / P-6.3・P-6.4 / P-4.1 / P-10.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(ア) と (4)②。rules 行 D-11 の作法 = 本便の根拠は ADR-11 決定 (4)②。向きの比較と決定は docs/design/adr-11-step2-enums.md（案 1 = 組み立て時の導出）。値は 1 つも変えない。
- 位置（3 本の 1 本目）: 本便 = 導出の土台と床の側 2 本 → 便 50 = 面の生成器の表の鍵の列 → 便 51 = 規則の表の欄の決まりの節。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ax が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + を付けた 2 本）。

## 1. 目的と中身

憲法の正本 design-intent/constitution.yaml は schema.enums に値域 10 本（tier・binds・pattern・strength・rationale_kind・mechanism_kind・mechanism_live・retreat_kind・stage・polarity）を持つ。ADR-11 決定 (3)(ア) により、この一覧は憲法の file が正本で、実装は手書きの写しを持たない。いま床の側に手書きの写しが 2 本在る = crates/folio/src/adr.rs の定数 RETREAT_KIND（spike・measure・ruling・床の木 FLOOR に入って判断の記録の欄の決まり adr/schema.yaml の生成区間へ出る）と、crates/folio/src/inject.rs の定数 STRENGTHS（must・must-not・should・注入が規範文かどうかを決める）。本便は、部品目録で既に動いている組み立て時の導出（crates/folio/build.rs が design-intent/preview/parts.json から型 4 つを OUT_DIR へ書く・便 13）と同じ型で、憲法の値域を型へ導出し、この 2 本を導出した名の列に替える。値・検査の結果・文言・生成区間の byte は 1 つも変えない。面の生成器の表（crates/folio/src/face.rs ほか）は本便では触らない（便 50）。

設計判断の席の実測（2026-09-20・main ac2fee9）: build.rs は正規化 207 行。関数 derive が部品目録を読み、関数 write_enum が閉じた一覧 1 つにつき型・定数 ALL・関数 name・関数 from_name を書く。名を型の名にする関数 variant は英小文字と数字と「-」だけを受ける = 憲法の値 M0 と M1 を受けない。src/adr.rs は正規化 1,221 行（余地 279）・src/inject.rs は 264 行・src/main.rs は 549 行。実行の crate は binary だけで、tests/ のどの file も src/adr.rs と src/inject.rs を取り込んでいない（取り込むのは src/sha256.rs と src/yaml.rs だけ）。crates/folio/src/link.rs の unit test floor_constants_are_read_through_the_floor は、床の木から読んだ撤退条件の種類が spike・measure・ruling の 3 値でこの順であることを固定している（本便の後も期待不変 = 導出の側から独立した凍結の針の 1 本）。

(a) build.rs に憲法の値域の導出を足す。入力の道は crate から見て ../../design-intent/constitution.yaml。純粋な関数を 1 つ設ける = 憲法の正本の文字列を受けて、導出した Rust の source の文字列か、理由の文を返す（名は constitution_enums・引数は文字列 1 つ・戻りは Result）。main はこの関数の結果を OUT_DIR の constitution_enums.rs へ書き、失敗なら理由を標準エラーへ出して 0 でない値で終える（部品目録と同じ）。cargo:rerun-if-changed に憲法の正本の道を足す。
導出の決まり:
1. schema.enums の表に在る鍵を file の順に全部導出する（鍵の名を build.rs に書き並べない = 鍵の一覧の写しを作らない）。
2. 鍵の名から型の名を作る = 「_」で割り、各片の先頭を大文字にして繋ぐ（retreat_kind は RetreatKind・mechanism_live は MechanismLive・tier は Tier）。
3. 値の名から型の中の名を作るのは既存の関数 variant。ただし英大文字も受けるように広げる（M0 は M0・delivery-0 は Delivery0・v1-incident は V1Incident・must-not は MustNot）。部品目録の側の結果は変わらない（部品目録に英大文字の名は無い）。
4. 型ごとに書くもの = 型（Debug・Clone・Copy・PartialEq・Eq）・定数 ALL（file の順）・定数 NAMES（値の字面の列・file の順・長さは値の数）・関数 name（const fn）・関数 from_name。
5. 組み立ての失敗にするもの = 文書が 1 つでない・schema.enums が表でない・表が空・ある鍵の値が一覧でない・一覧が空・値が文字列でない・同じ値が 2 度在る・2 つの値が同じ型の中の名に潰れる・2 つの鍵が同じ型の名に潰れる・型の名にならない字。黙って空の一覧にしない（P-4.1）。

(b) 新しい file crates/folio/src/constitution_enums.rs = 導出した source を取り込む置き場（include と OUT_DIR）。本便で使うのは RetreatKind と Strength の 2 つだけで、残り 8 つは便 50 が使う = 使われない型の警告はこの file の中で抑える（部品目録の mod catalog と同じ）。src/main.rs に mod の 1 行を足す。

(c) 写し 2 本を替える。src/adr.rs の定数 RETREAT_KIND は、手書きの 3 値をやめて RetreatKind の NAMES を指す（床の木 FLOOR は定数のままで、生成区間へ出る byte は不変）。src/inject.rs の定数 STRENGTHS は消し、規範文かどうかの判定は Strength の from_name で行う（値域の外の強度を断る文言「strength が規範の値でない」は不変）。ほかの関数は変えない。src/link.rs は触らない（床の撤退条件の種類の突き合わせは、本便の後は「この folio を組み立てた版の憲法と、いま読んでいる置き場の憲法のずれ」を落とす検査として、文言も種別もそのまま残る）。

(d) 歯。新しい歯の関数名は全部 constitution_enums で始める。
1. 新しい file crates/folio/tests/constitution_enums.rs（build.rs を path の属性で取り込み、(a) の純粋な関数を直に呼ぶ・使わない関数の警告はその取り込みの 1 行で抑える）:
   - 実の憲法の正本を渡す → Ok ∧ 出力に RetreatKind と MechanismLive の型が在る ∧ M0 と MustNot と V1Incident の名が在る。
   - 実の憲法の正本の schema.enums に在る鍵の数だけ型が出る（鍵の数は file から数える・10 と書かない）。
   - 変異 5 つがそれぞれ Err = schema.enums を消す・ある鍵の一覧を空にする・同じ値を 2 度書く・値を数にする・2 つの値が同じ名に潰れる組（例 = a-b と a--b は variant が同じ名を返す組を作業者が実測して選ぶ）。Err の文に鍵の名が入る。
2. src/constitution_enums.rs の unit test:
   - 実の憲法の正本を src/yaml.rs で読み、schema.enums の全部の鍵について、導出した型の NAMES が file の一覧と長さ・字面・順まで一致する（鍵と型の対は歯の中に 10 行で書いてよい・file の鍵がその 10 行と過不足なく一致することも見る）。
   - 導出の側から独立の凍結の針（P-10.1）= RetreatKind の NAMES は spike・measure・ruling、Strength の NAMES は must・must-not・should（字面を歯に直に書く）。
   - from_name は値域の外の字面に None、name は from_name の逆。
3. 回帰（期待不変・共通の検証が回す）: src/link.rs の unit test・tests/link.rs（凍結の組 retreat-kind-drift は違反 1 件のまま）・tests/inject.rs・tests/schema.rs（判断の記録の側 22182 byte・設計ノートの側 14618 byte・天井の正本 2915 byte の 3 行）・tests/adr.rs・tests/floor_cases.rs（凍結の場合 134 件）・tests/parts.rs（部品目録の導出は不変）。

(e) 大きさと接続。新規 file は 2 本（src/constitution_enums.rs・tests/constitution_enums.rs）。既存 = build.rs（約 +80 行）・src/adr.rs（1〜2 行）・src/inject.rs（数行）・src/main.rs（+1 行）。size S = 既存 file 1 本あたりの増分は 100 行に収まる見積。設計文書・fixture・src/link.rs・src/face.rs ほか面の生成器・src/render.rs・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない（build の依存 yaml-rust2 は既に在る）。

## 2. 範囲

- 入れる: build.rs の憲法の値域の導出と純粋な関数・導出した型の置き場・adr.rs と inject.rs の写し 2 本の置き換え・歯。
- 入れない: 面の生成器の表の鍵の列（便 50）・規則の表の欄の決まりの節（便 51）・改訂の範囲の下限と対話面の行 id（写しではない = 動かさない・docs/design/adr-11-step2-enums.md §3）・床が各欄の値を値域で数えること（別の bead）・値と文言の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| derive | 導出 | build.rs の純粋な関数 constitution_enums と OUT_DIR への書き出し |
| home | 置き場 | src/constitution_enums.rs（取り込みと unit test） |
| swap | 置き換え | adr.rs の RETREAT_KIND と inject.rs の STRENGTHS |
| teeth | 歯 | tests/constitution_enums.rs・unit test・ほかは期待不変 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ax"
title = "憲法の値域を組み立て時に型へ導出し（build.rs・部品目録と同じ型・鍵の一覧は file から）、判断の記録の床の撤退条件の種類と注入の強度の手書きの写し 2 本を導出した名の列に替える（値・文言・生成区間の byte は不変・ADR-11 決定 (4)② の 1 本目）"
req = ["FR5", "FR6", "FR19"]
section = "1"
write-set = ["crates/folio/build.rs", "+crates/folio/src/constitution_enums.rs", "+crates/folio/tests/constitution_enums.rs", "crates/folio/src/main.rs", "crates/folio/src/adr.rs", "crates/folio/src/inject.rs"]
verify = ["cargo nextest run -p folio constitution_enums", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "constitution_enums の歯（実の憲法からの導出・鍵の数は file から・変異 5 つが Err・導出した名の列が file と順まで一致・凍結の針 2 本・from_name と name）が緑、adr.rs と inject.rs に値域の手書きの 3 値が残らず、共通の検証（link・inject・schema の 3 行・adr・floor_cases 134 件・parts は期待不変）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
