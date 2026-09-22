# 設計: 便 91 — 規則の表の行に条以外を指す任意の欄 refs を足し、受け皿が無くて除外した 27 対を書き写す（FR19 / NFR3）

- 要件: FR19（欄の決まりの file と天井の正本と規則の表の決まりの部分を床の定数から導出する）/ NFR3（参照は必ずつながる）
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.2（文書と台帳からは規則を id で参照する）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-6.2（生成物を手で直さない）/ P-17.1（要件・条文・判断・受入基準の id を再利用・改番しない）/ N-2.1（散文にしか無い規則を規則として扱わない）/ N-3.1（規則の例外機構を足さない）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）
- 出所: 判断の記録 ADR-13 決定 (3-b)（イ）。設計ノート docs/design/graph-and-incremental-ceiling.md §8 の便の列の 2 本目（G0-a2）の前半。後付けの設計ノート docs/design/edge-retrofit-2026-09-22.md §4.1 の 3 が「受け皿の欄が無い（規則の表の行）」で落ちた 33 対を席へ返している。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cn が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規の file は無い・縮む file も無い）。
- 門: 本便は design-intent の下の正本（rules.yaml）を書き換えるので天井の門の対象である。席が実測した `folio ceiling --gate --dir design-intent --write-set design-intent/rules.yaml` は 2（まだ分からない・印が古い）を返す。持ち主の裁定 D-12（2026-09-21 23:45 JST「推奨で進めて」）により門の外で受ける。
- 対の便: 判断の記録の側（ADR-13 決定 (3-b)（ウ）・帰結の欄）は便 92（行 co・docs/design/delivery-92.md）。**書き換える file が 1 つも重ならない**ので依存は無いが、判断の記録が定めた順（（イ）→（ウ））に合わせて本便を先に出す。

## 1. 目的と中身

規則の表の行は、自分が依っている条を `article` の 1 つの欄でしか指せない。値は 1 つで、行 R-4 の双方向（条 → 行・行 → 条）でその条に固定されている。だから行の注が別の行・要件・受入基準・制約・判断の記録・別の条を名指しても、機械は「この行を直したら、その行が依っている先を読み直せ」と言えない。本便は規則の表の行に条以外を指す任意の欄 refs を足し、後付けの 1 回目が受け皿の無さで除外した 33 対のうち 27 対を書き写す（席の裁定で 6 対を外した・(b)）。欄の決まりの正本は実装の型付きの定数（P-5.6）で、設計文書の側の生成区間へは `folio schema --write` が導出する。

### (a) 実測（2026-09-22・main bcaff51・便 90 の着地後）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = rules.rs 277・余地 **1223**／check.rs 822・余地 **678**／adr.rs 1243・余地 257／refs.rs 431・余地 1069／link.rs 657・余地 843／face_constitution.rs 1294・余地 206。歯 = tests/check.rs 920・余地 **580**／tests/schema_docs.rs 1029・余地 **471**。本便が触る src は rules.rs と check.rs の 2 本で、どちらも余地が S の見積（100）を大きく上回る。**adr.rs（余地 257）は本便では 1 字も触らない**＝判断の記録の側を別便に割った理由がこれである。
- 規則の表の行の欄の閉じた一覧の正本は crates/folio/src/rules.rs の定数 4 本。THRESHOLD_REQUIRED 9 語（20 行）・THRESHOLD_OPTIONAL 5 語（25 行）・DISCIPLINE_REQUIRED 7 語（29 行）・DISCIPLINE_OPTIONAL 1 語（34 行）。同じ file の FLOOR（134 行〜）がその 4 本を木として持ち、`folio schema` はこの木を rules.yaml の生成区間へ導出する。
- **床は今のところ行の欄を数えていない。** crates/folio/src/check.rs の check_rules（575 行〜）は最上位の節の閉じた一覧（rules::RULES_TOP_LEVEL）と、各行の id / article / what の非空と、id の重複だけを見る。行の欄の集合を閉じる枝は無く、THRESHOLD_OPTIONAL の読み手は FLOOR と歯だけである（rules.rs の file の頭の注がそう書いている）。**本便もその形は変えない**＝欄を 1 つ足し、その欄の中身を数える枝を 1 本足すだけで、行の欄の集合を閉じる新しい振る舞いは持ち込まない。
- **id の形を解く口は既に在る。** crates/folio/src/adr.rs の is_basis_id（1093 行・pub(crate)）が 条と規範文・要件書の 5 接頭辞・規則の表の行・判断の記録 の全体一致を返し、要件書の図の行の refs（check.rs 744 行）と要件の行の adrs（便 90・check.rs 699 行〜）がすでにこの口を呼んでいる。**adr.rs は 1 字も触らない。**
- **実在の突き合わせも既に在る。** crates/folio/src/refs.rs の check_refs（31 行〜）が rules.yaml を population_without_schema（153 行〜）で読む＝schema の節を除く全欄を歩くので、新しい欄の要件書の id・条 id・規範文 id・規則の表の行の id は初めから行 R-4 の 1 つ目の数えの内側に入る。判断の記録の id は crates/folio/src/link.rs の references（308 行〜）が同じ母集団で拾い、実在しなければ種別 A-2 の違反を出す。**refs.rs も link.rs も 1 字も触らない。** 席は当てた木で実測した（AC99 を書くと 参照 id の違反 1 件・ADR-99 を書くと A-2 の違反 1 件・どちらも終了コード 1）。
- **行 R-4 は 1 字も変えない。** R-4 の what は 参照 id の未解決の数 ／ どの条からも参照されない rules 行の数、value は 0 ／ 0、population は 正本 4 file の全欄、note は 双方向（条 → 行・行 → 条）。1 つ目の数えの母集団は実装では schema の節を除く全欄なので、欄を足しても式も値も動かない。2 つ目の数え（refs.rs の reverse・67 行）は各行の article だけを読み、その条の relations.rules に行 id が載るかを見るので、新しい欄はこの数えに入らない。**したがって規則の表の行の追加も変更も起きず、裁定 id は要らない（P-17.1）。**
- 生成区間の実測（席が当てて測り、戻した）。閉じた一覧の 2 本に 1 語ずつ足し、注 refs_note を 1 つ足すと rules.yaml の生成区間は **27 行 1833 byte から 30 行 2269 byte** になる。変わるのは 3 か所で、①  threshold_row の optional の 1 行に refs が付く ② discipline_row の 1 行（今は 99 字の flow の 1 行）が 105 字になり導出の幅（schema.rs の WIDTH = 100）を超えるので block の 3 行に開く ③ refs_note の 1 行が増える。ほかの 7 file の生成区間は 1 byte も動かない（`folio schema --write` が 変わらない を 7 行、書いた を 1 行出す）。
- 生成区間の凍結の定数は 便 89 で tests/schema_docs.rs へ移っており、rules.yaml の側の値は 2 組に在る。RULES_REGION_LINES（60 行）・RULES_REGION_BYTES（61 行）・RULES_REGION_SHA256（62 行）と、歯 f85_rules_region_matches_the_new_anchor（846 行〜）が本文に直に書いている 4 つの字（849 行の rules.yaml・1833 byte・853 行の 27・854 行の 1833・858 行の要約値）。凍結 anchor は tests/fixtures/schema/rules-region.txt（27 行・1833 byte）。
- tests/schema_docs.rs の RULES_DRIFT_FROM / RULES_DRIFT_TO（469・470 行）は値域 stage の行を当て先にしており、その行は本便で 1 byte も動かないので **この 2 つの定数は変えない**。
- 土台の写し tests/fixtures/floor_base/design-intent/ と tests/floor_cases.yaml は **1 byte も動かない**（席が当てた木で floor_cases の歯が緑だった）。id の一覧の凍結 anchor design-intent/anchors/ も動かない。
- 面の歯と面の凍結の写しも動かない。席が当てた木で workspace 全体の nextest が **708 件すべて緑**（本便の歯 f91_ を足す前の本数）で、`folio build --write` が 20 file を書き、床は 違反 0・まだ分からない 0 だった。落ちたのは (d) が名指す 5 本だけで、どれも凍結の値と anchor の字である。
- 歯の関数名の接頭辞。`grep -rn 'fn f91_' crates/folio/tests` は今 0 本（使われている最大は f90_）。

### (b) 足す欄と、書き写す 27 対

欄の名は **refs**。閾値の行（R-n）と作法の行（D-n）の**任意**の欄で、値は id の一覧である。名を refs としたのは、要件書の図の行（check.rs 744 行）と設計ノートの図の行がすでに同じ名で同じ判定（is_basis_id）の一覧を持っており、**新しい名を作らずに済むから**である。

**ADR-13 決定 (3-b)（イ）の「条以外」の読み。** 決定の字は 条以外（要件・制約・判断の記録・ほかの行）を指す欄 で、同じ文が 33 対を書き写すと言う。ところが席が実測すると、その 33 対のうち **20 対は指す先が条か規範文**である（(b) の表の P- / A- / N- の行）。指す先から条と規範文を落とすと決定が約束した 33 対は達成できない。したがって決定の「条以外」は **article の欄が持つその 1 つの条以外**の意味であり、**別の条と規範文は書ける**と読む。この読みを機械に持たせるため、床は **その行の article の値と同じ id** と **その行自身の id** を refs に書くことを違反にする（(c) 2）。これで欄が article の写しにも自己参照にもならず、決定の「条以外」が機械で数えられる形に閉じる。席の裁定 2026-09-22。

書き写す 27 対は次のとおり。出所はどれも、その行の note か what か population の中の言及である。**id の順は 条と規範文 → 要件書の id → 規則の表の行 → 判断の記録**、同じ種類の中は番号の順とする。

| 行 | refs | 出所の欄 | 出所の字 |
| --- | --- | --- | --- |
| R-1 | P-4.2, P-6.3, D-3 | note | この行の判定は「まだ分からない」（P-4.2）／ここには写さない（…P-6.3）／行 D-3 の注はこの行を指す |
| R-3 | P-4.2, AC2 | note | M0 受入条件（AC2）／その判定は「まだ分からない」（P-4.2） |
| R-4 | AC6 | note | M0 受入条件（AC6） |
| R-5 | P-2.4 | note | 一覧への追加は判断の記録（P-2.4） |
| R-12 | P-4.2, R-2 | note | この行の判定は「まだ分からない」（P-4.2）／P-14 が定める写しの範囲（…行 R-2 の projection の欄）とは違い |
| R-13 | P-4.2 | note | この行の判定は「まだ分からない」（P-4.2） |
| R-14 | P-4.1, P-11.1, N-3.1, R-7 | note | 緩める口を持たない（N-3.1）／前の生成物も上書きしない（P-4.1）／往復の上限は R-7（P-11.1） |
| R-15 | P-10.3, A-3.1, CON2, ADR-4 | note | 依存を足す便 30 の受け皿（…A-3.1 の確認・要件書 CON2）／凍結 anchor（ADR-4 帰結…）／「まだ分からない」に落とす（P-10.3） |
| R-16 | P-6.3, P-10.1, ADR-3 | note | ここはその読み方の言い換えである（P-6.3）／独立した凍結 anchor を持つ（P-10.1…）／同じ式（ADR-3 決定 (7)） |
| R-17 | P-4.2 | note | それまでこの行の判定は「まだ分からない」（P-4.2） |
| D-3 | R-1 | note | 事実と日付は行 R-1 の注が持つ（ここには写さない） |
| D-10 | P-5.2, R-8 | what・note | 対話面（R-8）の散文で出し（what）／要件書 FR1 への行参照は v1.1 で足す（P-5.2） |
| D-11 | P-5.6 | what | 実装の型付きの定数のうち憲法 P-5.6（写しの導出）が掛かるのは |
| D-12 | ADR-8 | note | 判断の記録 ADR-8 決定 (4)(6) |

**行は 14 本・id は 27 個。** 数え直しの口は `~/.local/share/folio2/handoff-2026-09-21/grill/retrofit_pairs.py`（REPO を folio2 の根にして走らせ、出所の種類が 規則行 で 型付きの辺が無い 行を拾う）。この口を main bcaff51 で走らせると 規則行 の対は **42** で、うち 出所が ruling の欄だけの 9 対を外すと **33**（後付けの設計ノート §4 の E3 と同じ値）、そこから下の 6 対を外すと **27** になる。

**除外 6 対。** どれも後付けの 1 回目が別の種別で外した形と同じで、片方だけ辺にしない。

| 外す対 | 出所の字 | 種別 |
| --- | --- | --- |
| R-12 → P-14 | 指す条は 2026-09-21 に P-14（…）から P-5（…）へ移した | E6 版の来歴（N から移した） |
| R-12 → R-9 | 同じ出所の R-9・R-10・R-11 と揃う | E6 先例の引き合い（N と同じ語に揃えた） |
| R-12 → R-10 | 同上 | E6 |
| R-12 → R-11 | 同上 | E6 |
| R-16 → R-9 | 憲法・rules・要件書の規範文には掛けない（R-9 / R-12 の領分） | E7 否定・対象外 |
| R-16 → R-12 | 同上 | E7 |

**ruling の欄だけが出所の 9 対も書かない**（R-1・R-4・R-6・R-7・R-13・D-5・D-6・D-9 → P-17.3 と R-14 → ADR-4）。承認と裁定の来歴は「誰がいつ決めたか」の記録で、その行が何に依っているかではない（後付けの設計ノート §4 の E1 と同じ扱い・規則の表の行 R-17 の population の 数えない言及 にも 承認と版の来歴 が挙がっている）。R-15 → A-3.1・R-15 → ADR-4・R-16 → ADR-3 は ruling と note の両方に出るので、note の側を根拠として **書く**。

**判断の記録の側の 3 対は本便でも書く。** R-15 → ADR-4・R-16 → ADR-3・D-12 → ADR-8 は、便 92 が判断の記録の側に同じ相手を帰結として書く対でもある。走査は対を無向で数えるが、**規則の表の行 R-17 は行ごとに数える**（its what = その行以外の id のうち、その行の型付きの欄に無いものの数）ので、規則の表の行の側にも判断の記録の側にも欄が要る。両便が書いても内容は違う（行の側は 自分が依っている先、判断の記録の側は 自分が生んだもの）。

**書く場所は行の末尾**（flow の表の閉じ括弧の直前）で、1 行 1 欄の形は取らない（rules.yaml の行は 1 行 1 行の flow の表である）。

### (c) 床の側の判定（rules.rs と check.rs だけを触る）

1. crates/folio/src/rules.rs に欄の字の定数 `ROW_REFS` を 1 本置き（値は refs）、THRESHOLD_OPTIONAL を 5 語から 6 語へ（末尾に ROW_REFS）、DISCIPLINE_OPTIONAL を 1 語から 2 語へ（note の次に ROW_REFS）増やす。**どちらも末尾に足す**ので生成区間の差は最小になる。同じ file の FLOOR に注 `refs_note` を 1 つ足し、置き場は discipline_row の次・enums の前とする。注の字は (d) の anchor の逐語と 1 字も違わないこと。
2. crates/folio/src/check.rs に関数 check_rule_refs を 1 本足し、check_rules（575 行〜）の行の繰り返しの中から呼ぶ（非空の検査の直後）。中身は、欄が無い・null なら何もしない。一覧なら各項を見て、**adr::is_basis_id が偽**なら種別 schema の違反を 1 件（字面に 行 id と refs と その値 と id の形でない を含む）、真でも**その行自身の id かその行の article の値と同じ**なら種別 schema の違反を 1 件（字面に 自分の id か article の条である を含む）出す。一覧でなければ種別 schema の違反を 1 件（字面に refs が一覧でない を含む）出す。
3. **実在は数えない。** (a) のとおり refs.rs の check_refs と link.rs の references が rules.yaml の schema の節を除く全欄を歩いて数えるので、ここで重ねると 1 つの誤りが 2 件になる。

新しい module・新しい旗・例外の口（無効化の旗・今回だけの口）は持たない（N-3.1）。行の欄の集合を閉じる枝も足さない（(a)）。

### (d) 生成区間と凍結 anchor

`folio schema --write` が rules.yaml の生成区間を 30 行・2269 byte に書き直す。schema.rs の TARGETS は既に rules.yaml（rules::FLOOR）を持つので **schema.rs も main.rs も 1 行も触らない**。要件 FR19 の対象も増えない。

凍結 anchor tests/fixtures/schema/rules-region.txt は、生成器にも検査側にも依らずに組む（P-10.2）。組み方は、いまの anchor の 2 行を次の 4 行に替え、その後ろに注の 1 行を置くことで、OS の道具だけで足りる。席はその手順で組んだ写しが `folio schema --write` の出力と **1 byte も違わない**ことを実測し、byte 数と要約値を OS の道具 sha256sum で独立に出した（2026-09-22）。

替える前の 2 行（7 行目と 8 行目）:

```
    optional: [basis, projection, same_failure, population, note]
  discipline_row: {required: [id, article, what, kind, status, ruling, ruled_at], optional: [note]}
```

替えた後の 5 行:

```
    optional: [basis, projection, same_failure, population, note, refs]
  discipline_row:
    required: [id, article, what, kind, status, ruling, ruled_at]
    optional: [note, refs]
  refs_note: その行の散文が依っている、article の条以外の id の一覧（ほかの行・要件書の id・判断の記録・別の条と規範文）。各項は id の形（P-5.2）で、その行自身の id と article の値は書かない。未解決は行 R-4 の 1 つ目の数えが拾い、判断の記録の未実在は A-2 の網が拾う（R-4 の what と値と母集団は変えない）
```

- 新しい anchor の行数 = **30**・byte 数 = **2269**・要約値（sha256）= **d3f7f85d910a08c767cfe908219b1a89ceb906982a49dc2fb45313cbe9f6cf7f**
- **本便の前の main ではこの要約値はどこにも無いので、これを見る歯は赤い歯である。**

凍結の定数と歯の中の字は 6 か所を直す。tests/schema_docs.rs の RULES_REGION_LINES を 27 から 30・RULES_REGION_BYTES を 1833 から 2269・RULES_REGION_SHA256 を上の要約値へ。歯 f85_rules_region_matches_the_new_anchor（846 行〜）の 849 行の rules.yaml・1833 byte を rules.yaml・2269 byte へ・853 行の 27 を 30 へ・854 行の 1833 を 2269 へ・858 行の要約値を上の要約値へ（この歯は 1 本で 4 つの字を持つ）。

### (e) 面の側 — 本便では出さない

規則の表の面（憲法の面の §5 の数値の表）に refs の行き先を出すのは **本便では運ばない**。face_constitution.rs の余地は **206** で、M の便（300）は初めから入らない。面の生成器は自分が読む欄だけを描く閉じた表（788〜794 行の note / population / projection / basis / same_failure の 5 組と、作法の行の note）を持つので、知らない欄が 1 つ増えても落ちない。席が当てた木で `folio build --write` が 20 file を書き、面の歯・面の凍結の写し・parts の検査を含む nextest 708 件が全部緑だった。

### (f) 要件書と規則の表の行は触らない・次の一括の 🔴 が 1 つ出る

- **要件書（design-intent/srs.yaml）は 1 字も触らない。** 欄を足す根拠は FR19（欄の決まりを床の定数から導出する）と NFR3（参照は必ずつながる）で既に在り、新しい要件は要らない。生成区間の対象 file も増えない。**版は上げない**（便 90 が起草した v1.27 のままで、承認待ちの起草を 2 つ重ねない）。
- **規則の表の行（R-n・D-n）の what・value・population・note・裁定の欄は 1 字も触らない。** 本便が rules.yaml に足すのは 14 行の refs の欄と生成区間だけである。(a) のとおり R-4 の 2 つの数えはどちらも動かないので、行の変更は起きず、裁定 id は要らない（P-17.1）。
- **🔴 次の一括へ回す 1 点。** 行 R-17 の population の末尾は 書き写す先は、その行の種類が既に持つ型付きの欄だけ（判断の記録 = basis・要件書の項 = basis〔条の id だけ〕と rules・条 = relations）＝この行のために欄を増やさない と書いている。便 90（要件の項の adrs）・本便（規則の表の行の refs）・便 92（判断の記録の produced）で欄が 3 つ増えるので、この括弧の列挙は事実と食い違う。**行の population を直すのは規則の表の行の変更で、裁定 id が要る（P-17.1）ので本便からは外す。** 直しの中身は括弧の列挙に 3 つの欄を足し、末尾の一文を「この行のために欄を増やすときは判断の記録で定める」の向きに改めることで、裁定は判断の記録 ADR-13 の発効の承認（2026-09-22 11:42 JST・逐語「全部承認する」）が既に在る。次の一括の承認要求に 🔴 として載せる。

### (g) 歯（関数名は f91_ で始める・置き場は crates/folio/tests/check.rs）

tests/check.rs の Work（82 行〜）は実の design-intent を一時 dir へ写し、器の導出 file を写しの根に置き、git init と 1 commit を行う。rules.yaml を字面で変異させる口 mutate_rules（133 行）も既に在る。本便の歯はその写しを使う（新しい歯の file も新しい dir も作らない）。**変異の当て先は行 R-4 の refs の 1 か所**で、字は `, refs: [AC6]}` とする。席は実の rules.yaml でこの字がちょうど 1 回だけ現れることを実測した（mutate_rules は当て先が 1 か所でなければ自分で落ちる）。

1. f91_the_real_rules_carry_the_refs_field — 実の規則の表の写しで素の床が 終了コード 0・違反 0。写しの rules.yaml に `refs: [` の字がちょうど **14 回**現れ、その一覧の項の合計が **27 個**で、各項が (b) の表の id のどれかであること。本便の前の main には refs の欄が 1 つも無いので **赤い歯**。
2. f91_a_value_that_is_not_an_id_is_a_violation — 当て先を `, refs: [xyz]}` に替えて素の床 → 終了コード 1・違反 1 件で、行に R-4 と refs と xyz と id の形でない が出る。変異の当て先が無いので **赤い歯**。
3. f91_the_article_of_the_row_is_a_violation — 当て先を `, refs: [P-5]}` に替えて素の床 → 終了コード 1・違反 1 件で、行に R-4 と refs と P-5 と 自分の id か article の条である が出る（R-4 の article は P-5）。同じ理由で **赤い歯**。
4. f91_refs_that_is_not_a_list_is_a_violation — 当て先を `, refs: AC6}` に替えて素の床 → 終了コード 1・違反 1 件で、行に R-4 と refs が一覧でない が出る。同じ理由で **赤い歯**。
5. f91_an_id_that_does_not_exist_is_a_violation — 当て先を `, refs: [AC99]}` に替えて素の床 → 終了コード 1・違反 1 件で、行に thresholds[3].refs[0] と AC99 と 実在しない が出る（(a) のとおり refs.rs の網が受け持つことを歯で押さえる）。同じ理由で **赤い歯**。

5 本とも席が当てた木で実測し、5 件とも違反はちょうど 1 件だった。

回帰は verify の 2 行目で見る。tests/check.rs（本便の前の 41 本 ＋ f91_ の 5 本 = 46 本）と tests/schema_docs.rs（本数は変わらない）。器の受付は verify の旗 --bin の次の語を filter 語と読むので、verify の行に --bin は書かない（rules.rs の中の単体の歯 rules_floor_derives_the_frozen_anchor_byte_for_byte は .vessel.toml の common-verify の workspace の nextest が見る）。

### (h) 大きさ

src は crates/folio/src/rules.rs（277・余地 **1223**・+16 行の実測）と crates/folio/src/check.rs（822・余地 **678**・+32 行の実測）の 2 本で、合わせて **+48 行**。歯は crates/folio/tests/check.rs（920・余地 580・+70 行の見込み）と crates/folio/tests/schema_docs.rs（1029・余地 471・**行数は不変**で凍結の 3 つの値と歯の中の 4 つの字だけが変わる）。正本 design-intent/rules.yaml は 14 行の末尾が伸び、生成区間が 27 行から 30 行になる。凍結 anchor tests/fixtures/schema/rules-region.txt は 27 行から 30 行になる。size **S**（触る src の余地は 1223 と 678 で、どちらも S の見積 100 を大きく上回る）。新しい file も新しい dir も無く、縮む file も無い。外部 crate は増やさない。

### (i) 本便が運ばないもの・撤退条件

- 面の側の表示（(e)）。face_constitution.rs の余地 206 では M の便が入らないので、file を割る便の後に回す。
- 判断の記録に帰結の欄（produced）を足して 15 対を書き写すこと。ADR-13 決定 (3-b)（ウ）で、adr.rs（余地 257）だけを触る別便 92（行 co）である。
- 要件の項がほかの要件・制約を指す欄（12 対）。ADR-13 決定 (3-b) が便 90 の後の数えを見てから決めると保留している。
- 規則の表の行 R-17 を機械が数える歯（G0'）。受け皿の無い対が残っているあいだに歯を入れると床が落ち続けるので、便 92 の後である。
- 行 R-17 の population の直し（(f) の 🔴）。次の一括の承認要求に載せる。
- 憲法の条文・規則の表の行の値と母集団と裁定の欄・語彙・要件書・判断の記録の本文と欄の決まり・ほかの 7 file の生成区間・face_constitution.rs・adr.rs・refs.rs・link.rs・schema.rs・main.rs・土台の写し・tests/floor_cases.yaml・id の一覧の凍結 anchor・CI の yml。
- 撤退条件: refs の欄が機械で数えられる形に閉じていられなくなったとき（article の条そのものを refs にも書きたい対が出て、欄と article の使い分けが人の判断に戻るとき）は、(c) 2 の枝を check.rs から外し、欄を THRESHOLD_OPTIONAL と DISCIPLINE_OPTIONAL から落として 14 行の欄を消す。便 1 本で戻せる。欄を残したまま article との重なりを許す形は取らない（許した瞬間に R-4 の双方向がどちらの欄を見るのかが人の判断に戻るため）。

## 2. 範囲

- 入れる: 欄の字の定数 1 本・閉じた一覧 2 本に 1 語ずつ・注 refs_note 1 つ・床の枝 1 本（check_rule_refs）とその呼び出し 1 行・規則の表の 14 行の refs（id は 27 個）・生成区間 27 行 → 30 行・凍結 anchor 1 本・凍結の定数 3 つと歯の中の字 4 つの直し・f91_ の歯 5 本。
- 入れない: 面の側の表示・判断の記録の帰結の欄・要件どうしの欄・行 R-17 の歯・行 R-17 の population の直し・憲法と語彙と要件書・規則の表の行の値と母集団と裁定の欄・adr.rs と refs.rs と link.rs と face_constitution.rs と schema.rs と main.rs・土台の写しと tests/floor_cases.yaml・id の一覧の凍結 anchor・新しい file と新しい dir。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| field | 新しい欄 | 規則の表の行の refs（条以外の id を受ける任意の一覧） |
| floor | 床の枝 | crates/folio/src/rules.rs の定数 1 本と閉じた一覧 2 本と注 1 つ・crates/folio/src/check.rs の check_rule_refs と呼び出し |
| rows | 書き写し | design-intent/rules.yaml の 14 行の refs（id は 27 個） |
| region | 生成区間 | rules.yaml の生成区間 30 行（folio schema --write が書く） |
| anchor | 凍結 anchor | tests/fixtures/schema/rules-region.txt（30 行・2269 byte） |
| pins | 凍結の定数 | tests/schema_docs.rs の 3 つの定数と歯 f85_ の中の 4 つの字 |
| teeth | 歯 | crates/folio/tests/check.rs の f91_ 5 本 |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は .vessel.toml の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。便 92（行 co）とは書き換える file が 1 つも重ならない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cn"
title = "規則の表の行（thresholds と discipline）に条以外の id を受ける任意の欄 refs を足し、後付けの 1 回目が受け皿の無さで除外した 33 対のうち 27 対を 14 行へ書き写す。欄の決まりの正本は実装の型付きの定数（rules.rs の閉じた一覧と注）で、rules.yaml の生成区間へ folio schema --write で導出する。床は id の形（adr::is_basis_id）と、自分の id でも article の条でもないことと、一覧であることを数え、実在は既に在る refs.rs と link.rs の網に任せる。行 R-4 の what と値と母集団は変えないので裁定 id は要らない。面には当面出さない（face_constitution.rs の余地 206）"
req = ["FR19", "NFR3"]
section = "1"
write-set = ["crates/folio/src/rules.rs", "crates/folio/src/check.rs", "design-intent/rules.yaml", "tests/fixtures/schema/rules-region.txt", "crates/folio/tests/check.rs", "crates/folio/tests/schema_docs.rs"]
verify = ["cargo nextest run -p folio --test check f91_", "cargo nextest run -p folio --test check --test schema_docs", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f91_ の歯 5 本（実の規則の表に refs の行が 14 本・id が 27 個在って床が終了コード 0・id の形でない値で違反 1 件・自分の article の条で違反 1 件・一覧でない値で違反 1 件・実在しない id で違反 1 件）が全部緑、tests/check.rs と tests/schema_docs.rs の既存の歯が全部緑（生成区間 30 行 2269 byte と凍結 anchor が byte 一致し、凍結の要約値が sha256sum の測り直しと一致する）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
