# 設計: 便 92 — 判断の記録に帰結の欄 produced を足し、後ろ向きの辺 15 対を書き写す（FR19 / NFR3）

- 要件: FR19（欄の決まりの file と天井の正本と規則の表の決まりの部分を床の定数から導出する）/ NFR3（参照は必ずつながる）
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.2（文書と台帳からは規則を id で参照する）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-6.2（生成物を手で直さない）/ N-2.1（散文にしか無い規則を規則として扱わない）/ N-3.1（規則の例外機構を足さない）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）
- 出所: 判断の記録 ADR-13 決定 (3-b)（ウ）。設計ノート docs/design/graph-and-incremental-ceiling.md §8 の便の列の 2 本目（G0-a2）の後半。後付けの設計ノート docs/design/edge-retrofit-2026-09-22.md §4.1 の 4 が「帰結の欄が要る」で落ちた 16 対を席へ返している。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 co が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規の file は無い・縮む file も無い）。
- 門: 本便は design-intent の下の正本（adr/）を書き換えるので天井の門の対象である。席が実測した `folio ceiling --gate` は 2（まだ分からない・印が古い）を返す。持ち主の裁定 D-12（2026-09-21 23:45 JST「推奨で進めて」）により門の外で受ける。
- 対の便: 規則の表の行の側（ADR-13 決定 (3-b)（イ）・条以外を指す欄 refs）は便 91（行 cn・docs/design/delivery-91.md）。**書き換える file が 1 つも重ならない**ので依存は無いが、判断の記録が定めた順（（イ）→（ウ））に合わせて便 91 の後に出す。

## 1. 目的と中身

判断の記録は、自分が依っている先を根拠の欄 basis で指せるが、**自分が発効で生んだもの**（要件・受入基準・規則の表の行・後続の判断）を指す型付きの欄を持たない。後付けの 1 回目はこれを basis に書こうとして外した＝判断の記録の面は basis を章 04「根拠と撤退条件」に描く（face_adr.rs の CHAPTERS・35 行）ので、公開される面で根拠と帰結の区別が失われるからである。本便は判断の記録に帰結の欄 produced を足し、その 15 対を書き写す。欄の決まりの正本は実装の型付きの定数（P-5.6）で、設計文書の側の生成区間へは `folio schema --write` が導出する。**帰結を basis には書かない。**

### (a) 実測（2026-09-22・main bcaff51・便 90 の着地後）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = **adr.rs 1243・余地 257**／rules.rs 277・余地 1223／check.rs 822・余地 678／link.rs 657・余地 843／face_adr.rs 855・余地 645。歯 = tests/adr.rs 274・余地 **1226**／tests/schema.rs 624・余地 **876**。本便が触る src は adr.rs の 1 本だけで、**余地 257 は S の見積（100）を上回るが M の見積（300）には足りない**＝規則の表の行の側（rules.rs・check.rs）を別便 91 に割った理由がこれである。本便の実測の増分は **+42 行**で、着地後の adr.rs は 1285・余地 215 になる。
- 判断の記録の行の欄の閉じた一覧の正本は crates/folio/src/adr.rs の定数 RECORD（32 行）で、required 10 語（33〜36 行）・optional 8 語（37〜46 行）である。同じ file の FLOOR（99 行〜）がその 2 本を木として持ち、`folio schema` はこの木を adr/schema.yaml の生成区間へ導出する。
- 帰結の散文の欄 consequences は**既に在る**（RECORD の optional の 4 つ目）。値は字の一覧で、面の生成器（face_adr.rs 707 行）が章 05「改訂と帰結」の見出し「この判断で変わること」の下に描く。**produced はその散文と別の、id だけの欄である**（散文は何を変えるかを述べ、produced は生んだものの id を機械が辿れる形で持つ）。
- **id の形を解く口は既に在る。** 同じ file の is_basis_id（1093 行・pub(crate)）が 条と規範文・要件書の 5 接頭辞・規則の表の行・判断の記録 の全体一致を返す。ただし produced は**条と規範文を受けない**（(b)）ので、is_basis_id の中の条の判定だけを小さな関数 is_article_id に切り出して使う。切り出しは振る舞いを 1 つも変えない（is_basis_id はその関数を呼ぶ形になるだけ）。既存の単体の歯 id_shapes_are_scanned_without_regex（1126 行〜）が切り出しの後も同じ値を見る。
- **実在の突き合わせも既に在る。** crates/folio/src/link.rs の references（308 行〜）が判断の記録の各記録を全欄歩き（walk・351 行）、refs::scan_ids が拾った内部 3 空間の id と scan_adr_ids が拾った判断の記録の id を、実在しなければ種別 adr の違反として出す。**link.rs は 1 字も触らない。** 席は当てた木で実測した（ADR-99 を書くと 種別 adr の違反 1 件・終了コード 1）。
- **basis の床の枝と同じ形で足せる。** 同じ file の 683〜698 行が basis の各項を is_basis_id で数えており、欄の有無を見る口 present（1035 行）も在る。
- 生成区間の実測（席が当てて測り、戻した）。閉じた一覧 RECORD.optional に 1 語足し、注 produced_note を 1 つ足すと adr/schema.yaml の生成区間は **117 行 22263 byte から 127 行 22701 byte** になる。変わるのは 2 か所で、① optional の 1 行（今は 93 字の flow の 1 行）が 103 字になり導出の幅（schema.rs の WIDTH = 100）を超えるので block の 10 行に開く ② produced_note の 1 行が basis_note の次に増える。ほかの 7 file の生成区間は 1 byte も動かない。
- 生成区間の凍結の定数は tests/schema.rs に在る。REGION_LINES（38 行・117）・REGION_BYTES（39 行・22263）・REGION_SHA256（40 行）。凍結 anchor は tests/fixtures/schema/adr-region.txt（117 行・22263 byte）。**便 89 は残り 6 本の正本の側の歯を tests/schema_docs.rs へ移したが、判断の記録の側は tests/schema.rs に残っている**ので、本便は tests/schema_docs.rs を 1 字も触らない。
- adr.rs の中の単体の歯 floor_notes_are_outside_the_diff（1198 行〜）は FLOOR の `_note` で終わる欄の本数を 22 で凍結している（1203 行）。produced_note を足すので **23 に直す**。同じ file の adr_floor_derives_the_frozen_anchor_byte_for_byte（1215 行〜）は FLOOR の導出が凍結 anchor と byte 一致することを見るので、anchor を直せば緑になる。
- **欄の決まりの写しが tests/fixtures の下に 17 本ある。** `folio check` は adr/schema.yaml の schema の節を床の定数と突き合わせる（floor_diff・種類 adr の違反「schema.optional が床の定数と違う」）ので、閉じた一覧に 1 語足すと写しの側も直さないと 17 組の歯が落ちる。席は当てた木で実測し、直す前は workspace の nextest が 24 本落ち、17 本の写しの optional の 1 行を直すと **708 件すべて緑**になった。写しはどれも optional を flow の 1 行で持っているので、**1 行の置き換えで足りる**（生成区間の block の形に開く必要は無い＝床の突き合わせは字面でなく値を見る）。写しの `_note` の欄は突き合わせの外なので触らない（floor_base の写しの注は実の file より古い字を持つが、本便はそれを直さない）。
- 面の歯と面の凍結の写しは動かない。face_adr.rs は自分が読む欄だけを描くので、知らない欄が 1 つ増えても落ちない。席が当てた木で面の歯・parts の検査を含む nextest 708 件が全部緑だった。
- 土台の写しの正本 tests/floor_cases.yaml と id の一覧の凍結 anchor design-intent/anchors/ は **1 byte も動かない**（席が当てた木で floor_cases の歯が緑だった）。
- 歯の関数名の接頭辞。`grep -rn 'fn f92_' crates/folio/tests` は今 0 本（使われている最大は f90_）。

### (b) 足す欄と、書き写す 15 対

欄の名は **produced**。判断の記録の記録の**任意**の欄で、値は id の一覧である。受ける相手は **要件書の id（FR・NFR・AC・CON・GOAL）・規則の表の行（R-n・D-n）・判断の記録（ADR-n）**で、**条と規範文（P-n・A-n・N-n）は受けない**。条を受けないのは、判断が条に対して行うのは「生む」ではなく「改訂する」で、その記帳は既に amends の欄と憲法の側の amended_by が持つからである（欄を 2 つにして意味が重なると、機械で数えられる形に閉じるという向き〔条 N-2〕から遠ざかる）。さらに **自分の id と、自分の basis に既に在る id** も受けない＝同じ相手が根拠と帰結の両方になる形を床が断る。

**ADR-13 決定 (3-b)（ウ）が言う「要件・規則」と 12 という数について。** 決定の字は その判断が生んだ要件・規則を記す帰結の欄（12 対）である。その出所である後付けの設計ノート §4 の E8 と §4.1 の 4 は **16 対**を挙げており、その 16 のうち 5 対は指す先が判断の記録（後続の判断）である。E8 の定義の字は その判断が生んだ要件・受入基準・規則の行・**後続の判断** なので、後続の判断は初めから射程に入っている。決定の「要件・規則」は短い言い換えで、12 は起草時の実測の値である。**席は後続の判断も produced で受ける**と決めた＝受けない形にすると 5 対に受け皿が無いまま残り、同じ穴がもう 1 度開く。後続の判断は supersedes / superseded_by（後継）とも amends（改訂）とも別の関係なので、既存の欄では受けられない。席の裁定 2026-09-22。

**数は 15 対である。** 後付けの設計ノートが挙げた 16 対のうち **ADR-1 → ADR-7 は既に型付きの辺になっている**（後付けの便 PR #240 の着地で、相手の側の欄に入った）。席は `~/.local/share/folio2/handoff-2026-09-21/grill/retrofit_pairs.py` を main bcaff51 で走らせ、この対の 型付きの辺が在るか の欄が yes に変わっていることを実測した。残る 15 対が本便の確定値である。数の差は判断の記録の改訂を要しない（決定 (3-b) が定めるのは欄を足すことと帰結を basis に書かないことで、対の数はその根拠の実測値である）。

書き写す 15 対は次のとおり。**id の順は 要件 → 受入基準 → 規則の表の行 → 判断の記録**、同じ種類の中は番号の順とする。

| 判断の記録 | produced | 出所の欄 | 出所の字 |
| --- | --- | --- | --- |
| ADR-1 | ADR-2 | consequences | 条文を直す変更は…＋凍結 anchor（ADR-2）…の 3 点を伴わないと床が落とす |
| ADR-2 | ADR-11 | consequences | 現状は判断の記録 ADR-11 の帰結 2 つ目が引き取る |
| ADR-3 | FR12, R-16, ADR-9, ADR-11 | note | 決定 (7) の散文の門の引き取りの状態は要件書 FR12 の注と規則の表 R-16 の注が持つ／節の型の閉じた一覧の置き場は…（判断の記録 ADR-9・ADR-11 決定 (3)(イ)） |
| ADR-4 | FR15, R-15 | note | 実装（crates/folio/src/figure.rs）に入った（要件書 FR15 の注と同じ…）／規則の表 R-15 の値を rules.yaml から読み |
| ADR-8 | FR17, FR18, FR20, AC15, AC16, AC18, D-12 | consequences・note | 要件書の次の版（v1.6）= 天井の要件 2 つ（FR17…・FR18…）と受入基準 2 つ（AC15…・AC16…）／要件書 FR20 / AC18（第 1.19 版）／席の受付の手順の正本は規則の表の開発規律行 D-12 |

**判断の記録は 5 本・id は 15 個。** どの相手も、その判断の発効で生まれたか、その判断の決定を後から引き取ったものである。

**外す対。** 判断の記録が出所で型付きの辺がまだ無い対のうち、下の 8 対は帰結ではないので書かない。どれも後付けの 1 回目が別の種別で外した形と同じで、片方だけ辺にしない。

| 外す対 | 出所の字 | 種別 |
| --- | --- | --- |
| ADR-1 → FR1 | 番号は folio2 の他の id（P-1・R-7・FR1）と同じくゼロ詰めしない | E5 id の形の例示 |
| ADR-8 → ADR-6 | 判断の記録 ADR-6 の intake.yaml と同じ運び方 | E6 先例の引き合い |
| ADR-11 → NFR1 | 要件書 FR1 が行の id で数値を呼ぶ形（非機能要件 NFR1 と同形）にする | E6 先例の引き合い |
| ADR-3 → ADR-4 | 承認の欄（1 つの承認で 2 本発効した記録） | E1 承認と裁定の来歴 |
| ADR-2 / ADR-9 / ADR-10 / ADR-11 → R-8 | 承認の欄の対話面の値そのもの | E1 承認と裁定の来歴 |

**規則の表の行の側の 3 対は便 91 が別に書く。** ADR-4 → R-15・ADR-3 → R-16・ADR-8 → D-12 は、便 91 が規則の表の行の側に同じ相手を refs として書く対でもある。走査は対を無向で数えるが、**規則の表の行 R-17 は行ごとに数える**ので、判断の記録の側にも規則の表の行の側にも欄が要る。両便が書いても内容は違う（判断の記録の側は 自分が生んだもの、行の側は 自分が依っている先）。

**書く場所は記録の basis の次の行**で、1 行の flow の一覧とする（basis と同じ形）。

### (c) 床の側の判定（adr.rs だけを触る）

1. 欄の字の定数 `PRODUCED` を 1 本置き（値は produced）、RECORD.optional を 8 語から 9 語へ増やす。置き場は **consequences の次・supersedes の前**（散文の帰結の隣に id の帰結を並べる）。
2. FLOOR に注 `produced_note` を 1 つ足し、置き場は **basis_note の次・prose_note の前**とする。注の字は (d) の anchor の逐語と 1 字も違わないこと。
3. is_basis_id（1093 行）の中の条の判定を関数 `is_article_id` に切り出す（振る舞いは変えない）。
4. 記録の検査に枝を 1 本足し、置き場は basis の枝（683〜698 行）の後・amends の枝（700 行）の前とする。中身は、欄が無いなら何もしない。一覧なら各項を見て、**is_basis_id が偽か is_article_id が真**なら種別 adr の違反を 1 件（字面に 記録の id と produced と その値 と id の形でない を含む）、それ以外でも**その記録自身の id かその記録の basis に在る id と同じ**なら種別 adr の違反を 1 件（字面に 自分の id か根拠（basis）に在る を含む）出す。一覧でなければ種別 adr の違反を 1 件（字面に produced が一覧でない を含む）出す。
5. adr.rs の単体の歯 floor_notes_are_outside_the_diff の注の本数を 22 から 23 に直す。
6. **実在は数えない。** (a) のとおり link.rs の references が判断の記録の全欄を歩いて数えるので、ここで重ねると 1 つの誤りが 2 件になる。

新しい module・新しい旗・例外の口（無効化の旗・今回だけの口）は持たない（N-3.1）。

### (d) 生成区間と凍結 anchor

`folio schema --write` が adr/schema.yaml の生成区間を 127 行・22701 byte に書き直す。schema.rs の TARGETS は既に adr/schema.yaml（adr::FLOOR）を持つので **schema.rs も main.rs も 1 行も触らない**。要件 FR19 の対象も増えない。

凍結 anchor tests/fixtures/schema/adr-region.txt は、生成器にも検査側にも依らずに組む（P-10.2）。組み方は、いまの anchor の 12 行目 1 本を下の 10 行に開き、その後ろ（basis_note の行の次）に注の 1 行を置くことで、OS の道具だけで足りる。席はその手順で組んだ写しが `folio schema --write` の出力と **1 byte も違わない**ことを実測し、byte 数と要約値を OS の道具 sha256sum で独立に出した（2026-09-22）。

開く前の 1 行（12 行目）:

```
  optional: [amends, grill, approval, consequences, supersedes, superseded_by, note, figures]
```

開いた後の 10 行:

```
  optional:
    - amends
    - grill
    - approval
    - consequences
    - produced
    - supersedes
    - superseded_by
    - note
    - figures
```

basis_note の行の次に置く 1 行:

```
  produced_note: その判断が発効で生んだもの（要件・受入基準・rules 行・後続の判断）の id の一覧。根拠（basis）には書かない＝面の章 04 は根拠だけを描く。各項は id の形（P-5.2）で、条と規範文は書かない（条の改訂は amends と amended_by が持つ）。自分の id と basis に在る id は書かない
```

- 新しい anchor の行数 = **127**・byte 数 = **22701**・要約値（sha256）= **40396925dd036902ccc3b70e540718e9f3e3ab10b5eb341b9251f252d98558a8**
- **本便の前の main ではこの要約値はどこにも無いので、これを見る歯は赤い歯である。**

凍結の定数は tests/schema.rs の 3 つを直す。REGION_LINES を 117 から 127・REGION_BYTES を 22263 から 22701・REGION_SHA256 を上の要約値へ。

欄の決まりの写し 17 本は、それぞれの optional の 1 行を次のとおり置き換える（flow の 1 行のまま・block に開かない）。

```
  optional: [amends, grill, approval, consequences, produced, supersedes, superseded_by, note, figures]
```

置き換える 17 本: tests/fixtures/adr/effective-no-approval/adr/schema.yaml・tests/fixtures/adr/schema-drift/adr/schema.yaml・tests/fixtures/adr/two-adopted/adr/schema.yaml・tests/fixtures/anchor/no-anchor/adr/schema.yaml・tests/fixtures/anchor/root-digest-drift/adr/schema.yaml・tests/fixtures/check/dup-key/adr/schema.yaml・tests/fixtures/check/empty-field/adr/schema.yaml・tests/fixtures/check/unknown-section/adr/schema.yaml・tests/fixtures/floor_base/design-intent/adr/schema.yaml・tests/fixtures/link/adr-id-missing/adr/schema.yaml・tests/fixtures/link/amended-by-orphan/adr/schema.yaml・tests/fixtures/link/retreat-kind-drift/adr/schema.yaml・tests/fixtures/refs/bad-counts/adr/schema.yaml・tests/fixtures/refs/dangling-id/adr/schema.yaml・tests/fixtures/refs/orphan-rule/adr/schema.yaml・tests/fixtures/vocab/exemptions/adr/schema.yaml・tests/fixtures/vocab/unknown-word/adr/schema.yaml。**17 本とも直す前の字は同じ 1 行で、置き換えは 1 本につき 1 か所である**（席が実測した）。

### (e) 面の側 — 本便では出さない

判断の記録の面に produced の行き先を出すのは **本便では運ばない**。面の章は 5 つ（問題・決定・案・根拠と撤退条件・改訂と帰結）で閉じており（face_adr.rs の CHAPTERS・35 行）、章 05 には既に散文の帰結が出ている。id の帰結を面へ出すかは見た目の裁定を伴うので、便 91 と同じく面の便へ回す。face_adr.rs は自分が読む欄だけを描くので、知らない欄が 1 つ増えても落ちない（席が当てた木で nextest 708 件が全部緑）。

### (f) 要件書と規則の表の行は触らない・🔴 は便 91 が挙げた 1 点だけ

- **要件書（design-intent/srs.yaml）は 1 字も触らない。** 欄を足す根拠は FR19 と NFR3 で既に在り、新しい要件は要らない。生成区間の対象 file も増えない。**版は上げない**（便 90 が起草した v1.27 のままで、承認待ちの起草を重ねない）。
- **規則の表（design-intent/rules.yaml）と憲法と語彙は 1 字も触らない。** 行 R-4 の母集団は正本 4 file で、判断の記録はその外である（判断の記録の中の id の解決は link.rs が別に持ち、その扱いは adr/schema.yaml の basis_note が「rules 行 R-4 の行と母集団は変えない」と既に書いている）。行の追加も変更も起きないので裁定 id は要らない（P-17.1）。
- **🔴 は本便では新しく出ない。** 行 R-17 の population の末尾の一文（書き写す先は、その行の種類が既に持つ型付きの欄だけ…この行のために欄を増やさない）が便 90・91・92 の 3 つの欄で事実と食い違う件は、便 91 の §1 (f) が次の一括の 🔴 として挙げている。本便はそれと同じ 1 点を指すだけで、行は触らない。

### (g) 歯（関数名は f92_ で始める・置き場は crates/folio/tests/adr.rs）

tests/adr.rs の Work（79 行〜）は実の design-intent を一時 dir へ写し、器の導出 file を写しの根に置き、git init と 1 commit を行う。写しの ADR-1.yaml を字面で変異させる口 mutate（116 行）も既に在る。本便の歯はその写しを使う（新しい歯の file も新しい dir も作らない）。**変異の当て先は ADR-1 の produced の 1 か所**で、字は `produced: [ADR-2]` とする。席は実の写しでこの字がちょうど 1 回だけ現れることを実測した（mutate は当て先が 1 か所でなければ自分で落ちる）。

1. f92_the_real_records_carry_the_produced_field — 実の判断の記録の写しで素の床が 終了コード 0・違反 0。写しの adr/ の下で `produced: [` で始まる行を持つ file がちょうど **5 本**（ADR-1・ADR-2・ADR-3・ADR-4・ADR-8）で、その一覧の項の合計が **15 個**、どの項も P- / A- / N- で始まらないこと。本便の前の main には produced の欄が 1 つも無いので **赤い歯**。
2. f92_an_article_id_is_a_violation — 当て先を `produced: [P-6]` に替えて素の床 → 終了コード 1・違反 1 件で、行に ADR-1 と produced と P-6 と id の形でない が出る。変異の当て先が無いので **赤い歯**。
3. f92_an_id_in_the_basis_is_a_violation — 当て先を `produced: [FR16]` に替えて素の床 → 終了コード 1・違反 1 件で、行に ADR-1 と produced と FR16 と 自分の id か根拠 が出る（FR16 は ADR-1 の basis に在る）。同じ理由で **赤い歯**。
4. f92_produced_that_is_not_a_list_is_a_violation — 当て先を `produced: ADR-2` に替えて素の床 → 終了コード 1・違反 1 件で、行に ADR-1 と produced が一覧でない が出る。同じ理由で **赤い歯**。
5. f92_an_adr_that_does_not_exist_is_a_violation — 当て先を `produced: [ADR-99]` に替えて素の床 → 終了コード 1・違反 1 件で、行に ADR-1.produced[0] と ADR-99 と 実在しない が出る（(a) のとおり link.rs の網が受け持つことを歯で押さえる）。同じ理由で **赤い歯**。

5 本とも席が当てた木で実測し、5 件とも違反はちょうど 1 件だった。

回帰は verify の 2 行目で見る。tests/adr.rs（本便の前の本数 ＋ f92_ の 5 本）と tests/schema.rs（本数は変わらず、凍結の 3 つの値だけが変わる）。器の受付は verify の旗 --bin の次の語を filter 語と読むので、verify の行に --bin は書かない（adr.rs の中の単体の歯 floor_notes_are_outside_the_diff と adr_floor_derives_the_frozen_anchor_byte_for_byte と id_shapes_are_scanned_without_regex は .vessel.toml の common-verify の workspace の nextest が見る）。

### (h) 大きさ

src は crates/folio/src/adr.rs の 1 本だけ（1243・余地 **257**・**+42 行の実測**・着地後は 1285・余地 215）。歯は crates/folio/tests/adr.rs（274・余地 1226・+70 行の見込み）と crates/folio/tests/schema.rs（624・余地 876・**行数は不変**で凍結の 3 つの値だけが変わる）。正本は design-intent/adr/schema.yaml の生成区間が 117 行から 127 行になり、5 本の判断の記録に 1 行ずつ増える。凍結 anchor tests/fixtures/schema/adr-region.txt は 117 行から 127 行になり、欄の決まりの写し 17 本はそれぞれ 1 行が変わる。size **S**（触る src は adr.rs 1 本で、余地 257 は S の見積 100 を上回る。**M の見積 300 には足りないので本便は S に収める**）。新しい file も新しい dir も無く、縮む file も無い。外部 crate は増やさない。

### (i) 本便が運ばないもの・撤退条件

- 面の側の表示（(e)）。判断の記録の面の章の見た目の裁定を伴うので、面の便に回す。
- 規則の表の行に条以外を指す欄（refs）を足して 27 対を書き写すこと。ADR-13 決定 (3-b)（イ）で、rules.rs と check.rs だけを触る別便 91（行 cn）である。
- 要件の項がほかの要件・制約を指す欄（12 対）。ADR-13 決定 (3-b) が便 90 の後の数えを見てから決めると保留している。
- 規則の表の行 R-17 を機械が数える歯（G0'）。本便の着地で受け皿の無い対が無くなるので、その次の便である。
- 行 R-17 の population の直し（便 91 の §1 (f) の 🔴）。次の一括の承認要求に載せる。
- 判断 → 判断 の**改訂**の欄（設計ノート §8 の G7a）。本便が足すのは改訂ではなく帰結で、別の欄である。
- 憲法の条文・規則の表・語彙・要件書・判断の記録の本文（決定・案・撤退条件・平易文・帰結の散文）・ほかの 7 file の生成区間・face_adr.rs・link.rs・refs.rs・check.rs・rules.rs・schema.rs・main.rs・tests/floor_cases.yaml・id の一覧の凍結 anchor・tests/schema_docs.rs・CI の yml。
- 撤退条件: produced の欄が機械で数えられる形に閉じていられなくなったとき（条や規範文を produced に書きたい対が出て、amends との使い分けが人の判断に戻るとき）は、(c) 4 の枝を adr.rs から外し、欄を RECORD.optional から落として 5 本の記録の行を消す。便 1 本で戻せる。欄を残したまま受ける相手を条まで広げる形は取らない（広げた瞬間に amends と produced のどちらに書くのかが人の判断に戻るため）。

## 2. 範囲

- 入れる: 欄の字の定数 1 本・閉じた一覧 1 本に 1 語・注 produced_note 1 つ・is_article_id の切り出し・床の枝 1 本・注の本数の凍結 22 → 23・判断の記録 5 本の produced（id は 15 個）・生成区間 117 行 → 127 行・凍結 anchor 1 本・欄の決まりの写し 17 本の 1 行ずつ・tests/schema.rs の凍結の定数 3 つ・f92_ の歯 5 本。
- 入れない: 面の側の表示・規則の表の行の refs・要件どうしの欄・行 R-17 の歯と population の直し・判断 → 判断 の改訂の欄・憲法と規則の表と語彙と要件書・判断の記録の本文・link.rs と refs.rs と check.rs と rules.rs と face_adr.rs と schema.rs と main.rs・tests/schema_docs.rs・tests/floor_cases.yaml・id の一覧の凍結 anchor・新しい file と新しい dir。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| field | 新しい欄 | 判断の記録の produced（要件・受入基準・規則の表の行・判断の記録の id を受ける任意の一覧） |
| floor | 床の枝 | crates/folio/src/adr.rs の定数 1 本と閉じた一覧 1 語と注 1 つ・is_article_id の切り出し・記録の枝 1 本 |
| rows | 書き写し | design-intent/adr/ の 5 本の produced（id は 15 個） |
| region | 生成区間 | adr/schema.yaml の生成区間 127 行（folio schema --write が書く） |
| anchor | 凍結 anchor | tests/fixtures/schema/adr-region.txt（127 行・22701 byte） |
| copies | 欄の決まりの写し | tests/fixtures/ の下の 17 本の adr/schema.yaml の optional の 1 行 |
| pins | 凍結の定数 | tests/schema.rs の 3 つの値と adr.rs の注の本数 |
| teeth | 歯 | crates/folio/tests/adr.rs の f92_ 5 本 |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は .vessel.toml の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。便 91（行 cn）とは書き換える file が 1 つも重ならない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "co"
title = "判断の記録に、その判断が発効で生んだもの（要件・受入基準・規則の表の行・後続の判断）の id だけを受ける任意の帰結の欄 produced を足し、後付けの 1 回目が受け皿の無さで除外した後ろ向きの辺 15 対を 5 本の記録へ書き写す。帰結は根拠の欄 basis には書かない。欄の決まりの正本は実装の型付きの定数（adr.rs の閉じた一覧と注）で、adr/schema.yaml の生成区間へ folio schema --write で導出する。床は id の形と、条でも自分の id でも basis に在る id でもないことと、一覧であることを数え、実在は既に在る link.rs の網に任せる。面には当面出さない（判断の記録の面の章は 5 つで閉じている）"
req = ["FR19", "NFR3"]
section = "1"
write-set = ["crates/folio/src/adr.rs", "design-intent/adr/schema.yaml", "design-intent/adr/ADR-1.yaml", "design-intent/adr/ADR-2.yaml", "design-intent/adr/ADR-3.yaml", "design-intent/adr/ADR-4.yaml", "design-intent/adr/ADR-8.yaml", "tests/fixtures/schema/adr-region.txt", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "tests/fixtures/check/dup-key/adr/schema.yaml", "tests/fixtures/check/empty-field/adr/schema.yaml", "tests/fixtures/check/unknown-section/adr/schema.yaml", "tests/fixtures/floor_base/design-intent/adr/schema.yaml", "tests/fixtures/link/adr-id-missing/adr/schema.yaml", "tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "tests/fixtures/refs/bad-counts/adr/schema.yaml", "tests/fixtures/refs/dangling-id/adr/schema.yaml", "tests/fixtures/refs/orphan-rule/adr/schema.yaml", "tests/fixtures/vocab/exemptions/adr/schema.yaml", "tests/fixtures/vocab/unknown-word/adr/schema.yaml", "crates/folio/tests/adr.rs", "crates/folio/tests/schema.rs"]
verify = ["cargo nextest run -p folio --test adr f92_", "cargo nextest run -p folio --test adr --test schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f92_ の歯 5 本（実の判断の記録 5 本に produced の行が在って id が 15 個で床が終了コード 0・条の id で違反 1 件・根拠に在る id で違反 1 件・一覧でない値で違反 1 件・実在しない判断の記録で違反 1 件）が全部緑、tests/adr.rs と tests/schema.rs の既存の歯が全部緑（生成区間 127 行 22701 byte と凍結 anchor が byte 一致し、凍結の要約値が sha256sum の測り直しと一致する）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
