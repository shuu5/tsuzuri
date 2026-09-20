# 設計: 便 50 — 面の生成器の表の鍵の列（憲法の値域 10 本ぶん）を消し、名札を導出した型への網羅の場合分けで持つ（ADR-11 決定 (4)② の 2 本目・FR4 / NFR2）

- 要件: FR4（面は正本から生成する）/ NFR2（部品と型の閉じた一覧）
- 条: P-5.1・P-5.6 / P-6.3・P-6.4 / P-4.1・P-4.2 / P-10.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(ア) と (4)②。rules 行 D-11 の作法 = 本便の根拠は ADR-11 決定 (4)②。向きの比較と決定は docs/design/adr-11-step2-enums.md（案 1）。面の出力の byte は 1 つも変えない。
- 位置（3 本の 2 本目）: 便 49（f2-648.79）= 憲法の値域の導出の土台と床の側 2 本 → 本便 = 面の側 → 便 51 = 規則の表の欄の決まりの節。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ay が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

便 49 で、憲法の正本 design-intent/constitution.yaml の schema.enums の値域が、組み立て時に型へ導出されるようになった（crates/folio/build.rs の純粋な関数 constitution_enums → OUT_DIR の constitution_enums.rs → crates/folio/src/constitution_enums.rs が取り込む・型ごとに 定数 ALL・定数 NAMES・関数 name・関数 from_name・順は file の順）。いま面の生成器は、値域ごとに「鍵（値域の値の字面）と名札の対」の表を手書きで持つ。鍵の列は憲法の値域の手書きの写しで、憲法の側で値が足されても消えても、表は黙って古いままになる（一致を見ているのは段の 1 本だけ）。本便は、鍵の列を消して名札を導出した型への網羅の場合分けで持ち、値域の食い違いを組み立てで止める。面の出力の byte・Err の文言・正本・fixture は変えない。

設計判断の席の実測（2026-09-20・便 49 の着地後の main）: 鍵の列を持つ表は crates/folio/src/face.rs に 12 枚 = TIERS（鍵と構造体 Tier の対・3 行）・STRENGTH（3）・PATTERN（5）・BINDS（3）・MECH_KIND（4）・LIVE（5）・STAGE（2）・POLARITY（2）・RATIONALE_KIND（3）・RETREAT_KIND（3）・STRENGTH_MEANING（3・強度の意味）・PRIO（3・強度の色）。もう 1 枚が crates/folio/src/face_adr.rs の RETREAT_KIND（3・判断の記録の面の名札で、face.rs の側と名札の字面が違う）。表の行の順は 13 枚とも憲法の値域の file の順と同じである（= 導出した型の ALL の順）。使う所は表引き（X の関数 lookup・関数 tier_of）が 19 か所と、表を順に回す所が 2 か所（crates/folio/src/face_srs.rs の関数 legend_line = STRENGTH と STRENGTH_MEANING を並べて回し、PATTERN を回す・crates/folio/src/face_constitution.rs の関数 context = 表の長さ TIERS.len を読む）。規則の表の側の表 RULE_KIND と RULE_STATUS は本便では触らない（便 51 の側）。正規化した行数は face.rs 1,046・face_constitution.rs 1,136・face_index.rs 1,327（余地 173）・face_srs.rs 1,386（余地 114）・face_adr.rs 764・tests/face.rs 1,241。

(a) 名札の持ち方。導出した型 1 つにつき、その型の値を受けて名札（TIERS は構造体 Tier・ほかは文字列）を返す関数を置く。関数の中は型の値の全部を並べた場合分けで、その他を受ける枝を置かない = 憲法の値域に値が足されれば名札が無いので組み立てが通らず、値が消えれば無い値を名指しているので組み立てが通らない。同じ型に名札が 2 種類以上在るもの（強度 = 規範の語・意味・色、撤退条件の種類 = 憲法の面と判断の記録の面）は関数を分ける。値域の値の字面（must-not・ask-first など）を鍵として書いた表は face.rs と face_adr.rs から無くす。

(a2) 判断の記録の床の木の注。crates/folio/src/adr.rs（正規化 1,223・余地 277）の床の木 FLOOR は、撤退条件の種類ごとの説明の注 retreat_kind_note を表で持ち、その表の鍵 3 つ（spike・measure・ruling）が値域の値の手書きの字面である。鍵を RetreatKind の各値の name（const fn）で書く形に替える。注の文と順は変えない = 判断の記録の欄の決まり adr/schema.yaml の生成区間は byte 不変（folio schema --check の 1 行目は 22182 byte のまま）。

(b) 表引きの口。正本の値を導出した型へ引く口を X に 1 つ足す（型の from_name を受ける形でよい）。引けないときの Err の文言は今の lookup と同じ = 道・名・「の表に無い値」・値。関数 tier_of は名と引数と Err の文言（「段 の表に無い値」で始まる）を変えない。値域に関わらない表（DOC_STATUS・METHOD・TONE・RULE_KIND・RULE_STATUS・棚の表・FIGURE_LABELS ほか）と関数 lookup はそのまま残す。

(c) 順に回す所。legend_line は Strength の ALL と Pattern の ALL を回す。表の順と file の順は同じなので凡例の 1 行の byte は変わらない。context の「段の表の 3 つ全部でない」の数は Tier の ALL の長さから取る。

(d) 組み立てた版と読んでいる版のずれ。context は今、読んでいる置き場の憲法の schema.enums.tier の各値が段の表に在ること・2 度無いこと・数が同じことを見ている（文言は「段「…」が 2 度ある」と「段の表の 3 つ全部でない」）。導出の後、これは「この folio を組み立てた版の憲法」と「いま読んでいる置き場の憲法」のずれの検査になる。本便はこれを、読んでいる置き場の schema.enums に **在る鍵の全部** へ広げる = 鍵ごとに、各値が導出した型に在る・2 度無い・数が同じ、を見る（集合の一致・順は問わない = 今の段と同じ強さ）。組み立てた版に無い鍵が置き場に在るときも Err。段の 2 つの文言は変えない。ほかの鍵の Err は「constitution.yaml.schema.enums.」に鍵の名を続け、「組み立て時の憲法の値域と違う（組み立て直す）」で終える。置き場の schema.enums に無い鍵は見ない（最小の写しの fixture は段だけ・撤退条件の種類だけを持つので、直さずに通る）。鍵から名の列を引くために、build.rs の導出に「鍵の名と NAMES の対の列」の定数を 1 つ足す（名は ENUMS・file の順）。

(e) 歯。新しい歯の関数名は face_labels で始める（src の unit test）か face_unknown_when_an_enum で始める（tests/face.rs）。
1. crates/folio/src/face.rs の unit test（凍結の針・P-10.1）: 12 枚ぶんの名札を、値の字面と名札の対を歯の中に直に書いて固定する = 導出した型の ALL を回し、name で引いた対の列が、今の表の中身（本便の前の main の face.rs の字面）と順まで同じ。face_adr.rs の撤退条件の種類の名札も同じ形で face_adr.rs の unit test に 1 本。
2. 消す歯 1 つ（根拠つき）: face_srs.rs の unit test face_srs_label_tables_are_aligned のうち、STRENGTH と STRENGTH_MEANING と PRIO の鍵の列が揃っていることを見る 2 行は、鍵の列が無くなるので消す（3 つの名札が同じ型への網羅の場合分けになり、揃いは組み立てが保つ）。同じ歯の TONE の部分は残す。face.rs の unit test のうち表 TIERS を lookup に渡している 1 行は、(b) の口で同じ期待（表に無い値は Err）に書き換える。
3. tests/face.rs: 実の設計文書の置き場の写しで、憲法の schema.enums.pattern の一覧に値を 1 つ足す → 憲法の面の生成が「まだ分からない」（終了 2）∧ 標準エラーに schema.enums.pattern と「組み立て時の憲法の値域と違う」。同じ形で schema.enums に知らない鍵を 1 つ足す → 終了 2 ∧ 鍵の名。既存の face_unknown_when_a_tier_is_outside_the_table は期待不変。
4. crates/folio/tests/constitution_enums.rs に 1 本足す: 導出した source に定数 ENUMS が在り、実の憲法の鍵の数だけ対を持つ（関数名は constitution_enums で始める）。
5. 回帰（期待不変・共通の検証が回す）: 面の凍結の fixture と byte 一致の歯の全部（tests/face.rs・tests/face_index.rs・tests/face_adr.rs・tests/face_note.rs・tests/site.rs）・束の要約値の歯（tests/bundle.rs = 束に面が入る）・tests/parts.rs・tests/floor_cases.rs（凍結の場合 134 件）。

(f) 大きさと接続。新規 file は無い。既存 = face.rs（表 12 枚が関数に替わる・増分は 100〜200 行の見積）・face_constitution.rs（(d) で約 +40 行）・face_index.rs と face_srs.rs と face_adr.rs（表引きの書き換え = 増減ほぼ 0・face_srs.rs は余地 114 なので増やさない）・build.rs（正規化 332・ENUMS で約 +15 行）・adr.rs（(a2) で数行）・tests/face.rs（約 +60 行）・tests/constitution_enums.rs（約 +15 行）。size M = 既存 file 1 本あたりの増分は 300 行に収まる見積。設計文書・fixture・src/render.rs（退役待ち）・src/link.rs・src/inject.rs・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: 表 13 枚の鍵の列の削除と名札の関数・表引きの口・ずれの検査を在る鍵の全部へ・build.rs の ENUMS・歯。
- 入れない: 規則の表の側の表（便 51）・退役待ちの render.rs の 10 枚・床が各欄の値を値域で数えること（f2-648.82）・名札の字面の変更・面の出力の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| labels | 名札 | 導出した型への網羅の場合分け（その他の枝なし）・adr.rs の注の表の鍵 |
| parse | 表引きの口 | 正本の値を導出した型へ・Err の文言は不変 |
| skew | ずれ | 置き場の schema.enums に在る鍵の全部を組み立て時の写しと比べる |
| teeth | 歯 | 名札の凍結の針 13 枚ぶん・ずれ 2 本・ENUMS 1 本・ほかは期待不変 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ay"
title = "面の生成器の表 13 枚から憲法の値域の鍵の列を消し、名札を導出した型への網羅の場合分けで持つ（値が足されても消えても組み立てが通らない）・組み立てた版と読んでいる版のずれの検査を置き場の値域の在る鍵の全部へ（面の出力の byte と Err の文言は不変・ADR-11 決定 (4)② の 2 本目）"
req = ["FR4", "NFR2"]
section = "1"
write-set = ["crates/folio/build.rs", "crates/folio/src/face.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_index.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/adr.rs", "crates/folio/tests/face.rs", "crates/folio/tests/constitution_enums.rs"]
verify = ["cargo nextest run -p folio face_labels", "cargo nextest run -p folio --test face face", "cargo nextest run -p folio constitution_enums", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "face_labels の歯（名札の凍結の針 13 枚ぶん）が緑、face の歯（値域に値を足す・知らない鍵を足すと「まだ分からない」・既存は期待不変で面の凍結の fixture と byte 一致）が緑、constitution_enums の歯（ENUMS が鍵の数だけ）が緑、face.rs と face_adr.rs と adr.rs に値域の値の字面を鍵にした表が残らず、folio schema --check の 3 行が不変、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
