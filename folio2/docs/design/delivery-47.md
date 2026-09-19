# 設計: 便 47 — 天井の床の定数を ceiling.rs の 1 枚に寄せ、床の木 FLOOR と生成区間の受け皿（最上位の節 schema）を作る（ADR-11 決定 (4)① の 1 本目・FR19 / FR18 / FR17）

- 要件: FR19（決まりの部分を床の定数から導出する）/ FR18（所見の形と天井の 3 値を床で数える）/ FR17（天井の材料の束を観点ごとに組む）
- 条: P-5.1・P-5.6（実装の型付きの定数を規則の正本とするあいだ、写しを設計文書の置き場へ導出する）/ P-6.3・P-6.4 / P-10.1（凍結 anchor は生成側から独立）/ N-3.1 / N-5.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(イ)(ウ)(オ) と (4)①。rules 行 D-11 の作法（P-5.6 が掛かる定数を足す便・値を変える便は根拠の判断の記録を名指す）= 本便の根拠は ADR-11 決定 (4)①。値は 1 つも変えない（置き場を寄せ、木にするだけ）。
- 割り方（3 段）: 本便 = 受け皿（実装だけ・天井の正本は 1 字も触らない）→ 設計判断の席の PR = 天井の正本 第 3 版（生成区間を足し、人が書いた一覧 4 節を外す・持ち主の承認）と要件書の次の版 → 便 48 = 締め（folio schema の対象に足す・旧 4 節の検査を消す・最小の写しの fixture を直す）。3 段に割る理由は行数ではなく、途中の main を常に 床 合格・CI 緑に保つ順序である（新しい最上位の節は今の床が落とし、一覧を外すと今の床が「空」で落とす）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 av が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

天井の正本 design-intent/ceiling.yaml は、同じ一覧を人が書く欄と実装の定数の両方に持ち、床が 1 字の一致で突き合わせている（憲法 P-6.4 と両立しない形・天井の 2 周目の止める所見）。判断の記録 ADR-11 決定 (3)(イ) は、この形の決まりを実装の型付きの定数を正本にして file の中の生成区間へ写す、と決めた。本便はその 1 本目で、実装の側だけを片付ける。天井の正本・ほかの設計文書・既存の fixture は 1 字も触らない。値（一覧の中身・順・規則の文）は 1 つも変えない。

設計判断の席の実測（2026-09-20・main 4929978）: crates/folio/src/ceiling.rs（正規化 438・余地 1,062）が定数 9 本（CEILING_TOP_LEVEL 8 語・VERDICT_VALUES・VIEWPOINT_IDS・DOCUMENT_IDS・FINDING_REQUIRED・PLACE_REQUIRED・REFUTE_VALUES・RECORD_REQUIRED・BUNDLE_CONTENTS）を持ち、関数 same_list / same_set / contains_all で天井の正本の欄と突き合わせる。crates/folio/src/bundle.rs（正規化 548・余地 952）は同じ束の中身の一覧 CONTENTS と要約値の規則の名 DIGEST をもう 1 枚持ち、関数 load の中で天井の正本の bundle 節と順つきの等値で突き合わせ（違えば「床が組める形でない」）、関数 load_rules で所見の欄の決まり 8 本（finding の required / optional・place の required・refute の values・weights の values / refute・verdicts の values・record の required）を天井の正本から読んで型 Rules に入れる。crates/folio/src/findings.rs（正規化 1,145・余地 355）は反証の束の中身 REFUTE_CONTENTS（5 語）・結果の欄 RESULT_REQUIRED（6 語）・反証の規則の文 REFUTE_RULE（1 文）を持ち、判定は全部 Rules 経由の値を使う。床の木の機械（型 Floor の 5 変種・マクロ keys_floor・関数 floor_diff・導出 derive）は crates/folio/src/schema.rs に在り、判断の記録の側 adr.rs と設計ノートの側 note.rs が使っている。

(a) ceiling.rs に床の木 FLOOR を置く（crate の中へ公開・schema.rs の型 Floor を使う）。木の欄と順と字面は、設計判断の席が独立の実装で組んで main に置いた凍結 anchor tests/fixtures/schema/ceiling-region.txt（24 行・2,915 byte・sha256 = 5ad2f19b7d8c4a197865c5280f4c82653f97ef6dc0b9b45186a2678539429fa2）のとおりにする。1 行目は schema の見出しで、子は順に top_level（5 語 = meta・weights・documents・viewpoints・schema）・top_level_note・verdicts（values）・verdicts_note・viewpoints（観点の id 4 つ）・viewpoints_note・documents（読む文書の id 9 つ）・documents_note・finding（required・optional・place の required・refute の values）・finding_note・record（required）・record_note・bundle（contents・digest）・bundle_note・refute（contents・result_required・rule）・refute_note。名が _note で終わる欄 8 本は説明の注で、字面は anchor の逐語。作業者は、schema.rs の関数 derive に FLOOR を渡した結果が anchor の file と byte 一致するまで FLOOR の字面と順を合わせる。anchor の file を書き換えて合わせてはいけない（変えるなら設計判断の席へ問う）。体裁の規則（schema.rs の実装）には手を入れない。

(b) 定数を 1 枚に寄せる。bundle.rs の CONTENTS と DIGEST、findings.rs の REFUTE_CONTENTS・RESULT_REQUIRED・REFUTE_RULE を ceiling.rs へ移し、元の file は ceiling.rs のものを use する（名は変えてよいが、値と順は変えない）。所見の任意の欄の定数 FINDING_OPTIONAL（2 語 = refute・note）を ceiling.rs に新設する（今は天井の正本の finding の optional を bundle.rs が読むだけで、実装に定数が無い）。FLOOR の葉は、これらの定数と同じ配列を指す形にして、同じ一覧を ceiling.rs の中で 2 回書かない。最上位の節の許す一覧は、FLOOR の top_level の 5 語に、移行のあいだだけ許す旧 4 節（verdicts・finding・record・bundle）を足したものにする（旧 4 節の名は別の定数 1 本に置き、便 48 で消す）。

(c) 床（関数 check_ceiling）の振る舞い。最上位の節 schema が在るとき = schema.rs の関数 floor_diff で FLOOR と突き合わせ、食い違いの道 1 本につき違反 1 件を種別 ceiling で出す（文言は判断の記録の側と同じ型 =「ceiling.yaml: 床の定数と違う: schema.」に道を続ける）。このとき旧 4 節は無くてもよく、在れば今と同じ検査を掛ける。最上位の節 schema が無いとき = 今と 1 字も変わらない（旧 4 節は必須で、今と同じ文言の違反を出す）。つまり「旧 4 節も schema も無い」天井の正本は今と同じく落ちる（緩む窓を作らない）。weights・documents・viewpoints・meta の検査（観点の id の列と順・読む文書の id の集合・行の非空・reads の解決・重複）は変えない。

(d) 束を組む命令と所見の検査の読み手。bundle.rs の関数 load_rules は、weights の 2 本（values・refute）だけを天井の正本から読み、残り 6 本と FINDING_OPTIONAL は ceiling.rs の定数から Rules を埋める。型 Rules の形は変えない（findings.rs の判定の本文は変えない）。関数 load の中の bundle 節の突き合わせ（「床が組める形でない」）は消す（突き合わせる相手が定数そのものになるため。file の側から束の形を変える経路は (c) の床の検査が持つ）。関数 finding_text の出す byte は値が同じなので不変 = 束の凍結 anchor の要約値 4 本と反証の束の凍結値は変わらない。文書の id から面の名を引く表 FACE_NAMES は本便では触らない。

(e) 歯。既存の歯の期待は 1 本を除いて不変（実の正本と fixture は旧 4 節を持つままなので、今の文言がそのまま出る）。
1. crates/folio/src/ceiling.rs の中の単体の歯（関数名は ceiling_floor で始める）: derive に FLOOR を渡した結果が tests/fixtures/schema/ceiling-region.txt と byte 一致。共通の検証（workspace 全部の歯）が回す。
2. crates/folio/tests/ceiling.rs に足す（関数名は ceiling で始める）: 第 3 版の形 = 実の正本の写しから旧 4 節（と、その直前の注の行）を外し、末尾に anchor の file の中身を足したもの → folio check が 0・違反 0。歯は実の正本の版に依らない形で書く = 旧 4 節は在るものだけ外し、schema の節が既に在れば足さない（第 3 版が main に入った後も同じ歯が緑のまま）。
3. 同じ file: 歯 2 の写しの生成区間の側の 3 値の順を入れ替える → 終了 1・種別 ceiling の違反がちょうど 1 件・文言に schema.verdicts を含む。
4. 同じ file: 歯 2 の写しから schema の節も外す（旧 4 節も schema も無い）→ 終了 1（緩む窓が無いこと）。
5. crates/folio/tests/bundle.rs: 既存の歯 bundle_unknown_when_the_bundle_contents_differ_from_the_floor は、突き合わせが消えるので差し替える = fixture の天井の正本の写しから旧 4 節を外し末尾に anchor の中身を足しても、folio ceiling の書き出しが凍結 anchor の要約値 4 本と同じ束を組む（読み手が file の一覧に依らなくなったこと）。関数名は bundle で始める。
6. 回帰（期待不変）: tests/ceiling.rs の既存の歯・tests/bundle.rs の残りの歯・tests/findings.rs の全部（本文は変えない・verify の置き場として write-set に載せる）。

(f) 大きさと接続。新規 file は無い（anchor の file は設計判断の席が先に main へ置く）。既存 = ceiling.rs（FLOOR と注 8 本・寄せた定数・schema の節の検査で約 +90 行）・bundle.rs（ほぼ増減なし）・findings.rs（定数 3 本が出て縮む）・tests/ceiling.rs（約 +70 行）・tests/bundle.rs（差し替えで約 +10 行）・tests/findings.rs（本文不変）。size S = 既存 file 1 本あたりの増分は 100 行に収まる見積。schema.rs・main.rs・check.rs・天井の正本・要件書・fixture・tests/floor_cases.yaml・tests/fixtures/floor_base/・CI の yml は触らない。folio schema の対象の一覧（schema.rs の TARGETS）にはまだ足さない（実の天井の正本に印が無いので、足すと実の正本を見る歯が落ちる・便 48 の領分）。外部 crate は増やさない。

## 2. 範囲

- 入れる: ceiling.rs の床の木 FLOOR（注 8 本つき）・定数の 1 枚化・schema の節の受け皿・読み手 2 つ（load_rules・bundle 節の突き合わせ）の切り替え・歯。
- 入れない: 天井の正本と要件書の変更（設計判断の席の PR・持ち主の承認）・folio schema の対象の追加・旧 4 節の検査の削除・最小の写しの fixture 18 本と凍結した土台の直し（便 48）・面の名の表 FACE_NAMES・重さの値域 weights（file が正本のまま = ADR-11 決定 (3)(ア)）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| floor | 床の木 | ceiling.rs の FLOOR（欄 20・うち注 8）= 凍結 anchor と byte 一致 |
| consts | 定数の 1 枚化 | bundle.rs の 2 本と findings.rs の 3 本を ceiling.rs へ・FINDING_OPTIONAL を新設 |
| tray | 受け皿 | 最上位の節 schema を許し、在れば floor_diff・無ければ今のまま |
| readers | 読み手 | load_rules の 6 本 + 任意の欄を定数から・bundle 節の突き合わせを消す |
| anchor | 凍結 | tests/fixtures/schema/ceiling-region.txt（2,915 byte・sha256 5ad2f19b…） |
| teeth | 歯 | 単体 1・ceiling +3・bundle 差し替え 1・既存は期待不変 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "av"
title = "天井の床の定数を ceiling.rs の 1 枚に寄せ、床の木 FLOOR（凍結 anchor と byte 一致）と最上位の節 schema の受け皿を作る。束を組む命令の読み手は一覧を定数から取る（値と床の今の文言は不変・天井の正本は触らない・ADR-11 決定 (4)① の 1 本目）"
req = ["FR19", "FR18", "FR17"]
section = "1"
write-set = ["crates/folio/src/ceiling.rs", "crates/folio/src/bundle.rs", "crates/folio/src/findings.rs", "crates/folio/tests/ceiling.rs", "crates/folio/tests/bundle.rs", "crates/folio/tests/findings.rs"]
verify = ["cargo nextest run -p folio --test ceiling ceiling", "cargo nextest run -p folio --test bundle bundle", "cargo nextest run -p folio --test findings findings", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "ceiling の歯（第 3 版の形が通る・生成区間の側のずれが違反 1 件・旧 4 節も schema も無ければ落ちる・既存は期待不変）が緑、bundle の歯（一覧を file から外しても凍結 anchor と同じ束・残りは期待不変）が緑、findings の歯が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
