# 設計: 便 140 — 設計ノートの欄の決まりの生成区間の、導出物の置き場の字から器の名（scribe2）を外す（消費側の repo）

- 要件: FR19（判断の記録と設計ノートの欄の決まりの file などの生成区間へ、床の定数から決定的に導出した決まりの部分を書き込む・--check は差分を検出する）。本便は、設計ノートの欄の決まりの生成区間の 1 行（導出物の置き場 placement）の字を、正本である実装の型付きの定数の側で直し、`folio schema --write` で導出し直すだけで、FR19 の規範文も欄の集合も値域も変えない。契約表の行の req は FR19 の 1 つ（main に在る id）。
- 条: P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する＝直す先は定数で、写しは導出で直る）/ P-6.2（生成物を手で直さない＝生成区間の字を手で書き換えない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-10.1（検査は生成側からも検査側からも独立した凍結 anchor を持つ＝凍結 anchor は字面の置き換えで組み、歯の中の期待の字は手で写す）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）。
- 出所: 天井の 34 周目（2026-09-25・一括 20 の枝 docs/batch20 c8cdce8 の上）の文書どうしの整合の所見 **F-2**（重さ 直す・場所 design-note の schema.derived.placement）。一括 20 の仕分け（`docs/design/batch20-triage.md`）の便の候補 **B-5**。台帳の控え **f2-648.208**（規則の表の行 D-14 の、直す先が枝に書けない所見を控えに束ね、窓を閉じた後・次の周の前に便で運ぶ 1 件）。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `em` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 10 本（書き換える 7 本 + 本文が変わらない verify の scope 3 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/design-note/schema.yaml` の生成区間）を書き換えるので、天井の門の対象である（規則の表の行 D-12）。起草役が write-set 10 本を base の binary で `folio ceiling --gate --dir design-intent --write-set …` に渡すと **0（通す・断りの字 = 印が 4 観点とも合格・引き金の要約値が同じ・印の後に引き金の外の変更が在る（節点 6 個と節点の外の字・次の引き金の周が読む））**。便の全差分を当てた写しで撃っても同じ字で 0 だった（§1 (g)）。
- 前の便: 前提の着地は無い。**base = main 021b7a4（便 138 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 便 138（行 ek・台帳 f2-648.210）は着地済み（main 021b7a4＝本便の base）。便 138 の書き換えた file は面の生成器と面の歯と面の fixture だけで（`face_labels.rs`・`face_srs.rs`・`face_index_read.rs`・`tests/face_srs.rs`・`tests/face_index.rs`・`tests/fixtures/face/` の 2 本）、本便の write-set と 1 本も重ならない（`git diff --stat f82dba1 021b7a4` で本便の write-set の 10 本は 1 byte も動いていない）。重なりは 0。便 139（起草中・`face_index*.rs` と `tests/face_index.rs` の見込み）とも重なりは 0 である（起草役の実測の時点で枝 docs/d139 に commit は無い）。**共通の検証（workspace の nextest）の本数は受付の時点の main で数え直す**（base 021b7a4 で 897・本便の後 899）。共通の検証は同時に撃たない逐次を勧める（`crates/folio/tests/sheet.rs` の一時 dir の衝突を避けるため）。
- 改訂 b（2026-09-25・席の依頼）: base を main f82dba1 から 021b7a4（便 138 の着地）へ移し、枝を rebase した。数え直した値は nextest の本数（base 889 → 897・本便の後 891 → 899）だけが動き、write-set の 10 本の行数・生成区間と anchor の数と sha256・既存の歯の落ちる 18 本・RED・突然変異・門・床・`folio build` の差 0 は f82dba1 の値と同じだった（write-set の 10 本は f82dba1 と 021b7a4 の間で 1 byte も動いていない）。write-set・verify・size・done は変えない。 同じ改訂で、§0 の重なりの欄に便 139 との重なり 0 を書き、(f) の 3 に `tests/schema.rs` の残り 4 行の注記を足した（席の裁定 2026-09-25・問い 6 点は推奨どおり）。
- 改訂 c（2026-09-25・独立の検証 d140-verify.md〔支持・blocking 0・数値は全部一致〕の任意の文面 3 点）: (f) の floor_note.rs の 509 と +20 行に、起草役の歯の書き方による参考値であること（検証役の歯では 503 / +14）を添えた。(c) の 3 の tests/schema.rs の 16 行を、実装が守る基準（700 行以下・歯 f89 が落とす上限）として読める字にした。(e) の 1 に、実装だけの写しでは f99 は落ちず、土台の写しを直したときに動くことの 1 句を足した。数値・write-set・verify・size・done は変えない。

## 1. 設計

### (a) いま起きていること（実測・base main 021b7a4・数は参考値）

1. **生成区間の今の字。** 設計ノートの欄の決まり `design-intent/design-note/schema.yaml` の schema 節は生成区間で（`# folio:schema:begin` から `# folio:schema:end` まで）、正本は実装の型付きの定数 `crates/folio/src/floor_note.rs` の床の木 FLOOR である。`folio schema --write` がその木を生成区間へ導出する（FR19・判断の記録 ADR-9）。導出物の節 derived の置き場の欄 placement の行は、逐語で次のとおり（生成区間の 96 行目・file の 116 行目）。

```
    placement: 消費側（器 scribe2）の repo に版管理で置く。path は消費側が宣言する（拡張子 .toml・全文を同じ parser に渡す）
```

2. **なぜ古い字か。** 判断の記録 ADR-21 決定 (1) は、M3 の出口の利用者を外部の利用者 1 つ（scribe2）から v3 の設計の置き場へ読み替え、判定点 ② を 導出物が v3 の repo に着地して差分 0 を数えられる とした。要件書の制約 CON8 も、導出物を作る形は M3 の出口の利用者（器の次の世代〔scribe v3〕の設計の置き場）が自分の repo で書く契約表に当てると書く。導出物を置く repo は、器（便を運ぶ道具・今は scribe2）の repo ではなく、導出物を読む消費側の repo である。今の字は消費側と器 scribe2 を同じものとして焼いており、ADR-21 の後では置き場の名が合わない。
3. **語彙の同じ向きの直し。** 語彙の項 導出物（derived-artifact）の定義は、33 周目の直しで括弧の中を 器〔便を運ぶ道具〕が読む契約表の形 とし、置き場や道具の名を焼かない字にした。本便は同じ向きで、置き場の欄から器の名を外す。
4. **生成区間なので正本の字では直せない。** 生成区間を手で直すと `folio schema --check` が差分で落ち（P-6.2）、次の `--write` が元の字へ戻す。直す先は定数の側である。
5. **写しの全数（`find` と `grep` で列挙）。** 句 （器 scribe2） を持つ file は base の repo に 5 本（本文書を除く）で、そのうち本便が直すのは 4 本である。

| file | 何か | 本便 |
| --- | --- | --- |
| `crates/folio/src/floor_note.rs` | 正本（FLOOR の derived.placement の Floor::Val の字） | 直す |
| `design-intent/design-note/schema.yaml` | 実の生成区間（導出） | `folio schema --write` で直す |
| `tests/fixtures/schema/note-region.txt` | 生成区間の凍結 anchor（歯 `note_floor_derives_the_frozen_anchor_byte_for_byte` と `tests/schema.rs` が読む） | 字面の置き換えで直す |
| `tests/fixtures/floor_base/design-intent/design-note/schema.yaml` | 床の凍結の土台の写し（floor_cases・freeze・ids などの歯が写して使う） | 1 行を手で直す |
| `docs/design/batch20-triage.md` | 仕分けの記帳（所見の証拠の字） | 直さない（来歴） |

   設計ノートの欄の決まりの file は、`find . -name schema.yaml -path '*design-note*'`（target と .git を除く）で実の 1 本と土台の 1 本の 2 本だけである。欄の名 placement を持つ file も、`grep -rl placement tests crates` で上の 3 本と `crates/folio/tests/figure.rs`（図の歯の関数名の一部で、別の意味）だけである。
6. **土台の写しも直す理由。** 土台の写しは、注（`_note` で終わる欄）が古い版の字のまま残っている（例 folio_check_note・derived_note・index_note は今の定数と字が違う）。注は床の突き合わせの外なので落ちない。ところが placement は `_note` で終わらない欄で、床（`folio check`）が値を床の定数と突き合わせる。起草役が定数だけを直した写しで土台を `folio check` に当てると、違反 1 件 `[note] design-note/schema.yaml: 床の定数と違う: schema.derived.placement` で合格しない。
7. **生成区間の数（便の前 → 後・sha256 は `sha256sum` と python の hashlib の 2 実装で一致）。**

| もの | 行数 | byte | sha256 |
| --- | ---: | ---: | --- |
| 生成区間（= 凍結 anchor `note-region.txt` の全体）の前 | 137 | 16,806 | 21dc7f7cb7304f2da4890cb501aab731697d6b2a620899c23c5610791f994a20 |
| 同 後 | 137 | 16,789 | 558fc1742fe371f71868912369d716cbc693210a576175ffb3b83cad1b7f151e |
| `design-intent/design-note/schema.yaml` の全体の前 → 後 | 179 → 179 | 23,904 → 23,887 | 3b342f19… → 4f4c011e… |
| 土台の写しの全体の前 → 後 | 179 → 179 | 21,219 → 21,202 | f31ce2b4… → 2437d457… |

   差は placement の 1 行の句 （器 scribe2） の 17 byte だけで、行数は変わらない。ほかの 8 本の生成区間（判断の記録・天井の正本・規則の表・入口の正本・要件書・語彙・相談窓口・索引の欄の決まり）は 1 byte も動かない（`folio schema --check` が 9 file とも 一致）。
8. **生成区間の中のほかの scribe2 の字（直さない）。** 生成区間には placement のほかに scribe2 を名指す行が 13 在る（id_note・n_note・section_ref_note・rows_note・contract_table.owner・external_schema_note・reads_note・semantic_check_owner・semantic_check_note・derived_note・landing.source_of_truth・landing.trailer_name_source・guards_note）。どれも folio2 の便を運ぶ器（語彙の項 器 が folio2 では scribe2 を指すと定める）の型・受付・記録か、実測の出所を指す字で、消費側の repo の置き場を名指すのは placement だけである。34 周目の所見も placement だけを挙げた。
9. **base の歯（参考値）。** workspace の nextest 897 / 897・clippy 0 警告・床 4 本（check・inject --check・schema --check・derive --check）rc 0・`folio build` の出力 30 file。`git grep -n 'f140_' -- crates` は 0 件。歯の file の本数は schema 20・schema_docs 29・floor_cases 12。

### (b) 直す先 — 定数の字 1 か所と、導出・写し・凍結の値

1. **定数（`crates/folio/src/floor_note.rs`）。** FLOOR の derived の節の placement の Floor::Val の字を次にする。消費側 の後の括弧（器 scribe2）だけを外し、ほかの字は 1 字も変えない。

```
消費側の repo に版管理で置く。path は消費側が宣言する（拡張子 .toml・全文を同じ parser に渡す）
```

   消費側 は、導出物を読む側（M3 の出口では v3 の設計の置き場の repo・folio2 自身の置き場では folio2 の repo の `contracts/`）を指す既存の字で、同じ生成区間の derived_note の 導出物の置き場（--out）は消費側が宣言する と揃う。置き場の名を焼かないので、v3 の器が scribe2 から替わっても字は古くならない。
2. **生成区間（`design-intent/design-note/schema.yaml`）。** 手で書かず、`folio schema --dir design-intent --write` の出力をそのまま commit する（design-note/schema.yaml だけが 書いた・ほかの 8 本は 変わらない）。生成区間の外（頭の注・meta・mapping・plain）は変えない。
3. **凍結 anchor（`tests/fixtures/schema/note-region.txt`）。** folio の出力から写さず、今の anchor の placement の行の句 （器 scribe2） を外す字面の置き換え（OS の道具か python の標準 library）で組む。起草役は、定数を直した binary の `--write` が書いた生成区間と、字面の置き換えで組んだ anchor が **1 byte も違わない**ことを確かめた（(a) の 7 の後の値）。
4. **凍結の定数（`crates/folio/tests/schema.rs`）。** NOTE_REGION_BYTES を 16806 から 16789 に、NOTE_REGION_SHA256 を (a) の 7 の後の値に直す。NOTE_REGION_LINES（137）は変えない。定数の上の doc 注に 便 140 (b) の 1 行を足してよい。
5. **土台の写し（`tests/fixtures/floor_base/design-intent/design-note/schema.yaml`）。** placement の行の句 （器 scribe2） だけを外す（1 行・17 byte）。古い字のままの注は直さない（突き合わせの外で、直すと本便の範囲を広げる）。
6. **便 99 の凍結 anchor の巻き添え（`tests/fixtures/schema/node-digest-anchor.txt` と `crates/folio/tests/graph.rs`）。** 土台の写しは便 99 の残差（節点の外の byte）の母集団に入るので、残差の 2 行（191〜192 行）が動く。直し方は便 101・119・121・126・129 と同じで、土台を直した後に独立の script `tests/fixtures/schema/node-digest.py` を土台に当てて anchor を組み直し（folio の出力から写さない）、`sha256sum` で測ったその file の要約値を `graph.rs` の定数 F99_ANCHOR_SHA256 に写す（上の doc 注に 便 140 の 1 行を足してよい）。

| 値 | 前 | 後 |
| --- | --- | --- |
| 残差 sha256（191 行） | c2c6fe61c3218d82e48aeefbddc11ce54bcbbbad502e3446f657a4c298a6a2ec | 59b39e9c41046d7322a34a4e6cb536b81c7b653818c06c7aae357ae7805e8c92 |
| 残差の byte・合計（192 行） | 142997・391169 | 142980・391152 |
| anchor の行数・byte | 192・3,007 | 192・3,007（不変） |
| F99_ANCHOR_SHA256 | c76bd9acddcfccb3c0caaa7d396ca4cbfe474937891ae3cd7b8a3e9e85016946 | 10bd11d5526fc203249b2550ebe1cdd9f3ab0ffe89a57dea24b8eb81add95d74 |

   節点の要約値の行（1〜190 行）と `graph-digest-anchor.txt` は 1 byte も動かない。残差の 2 行のほかが 1 byte でも変わったら、本便の土台の直しが間違っている。
7. **変えないもの。** 生成区間のほかの行（(a) の 8 の 13 行を含む）・ほかの 8 本の生成区間・`crates/folio/src/note.rs`（床の本体と単体の歯）・`schema.rs`（src）・`main.rs`・語彙・要件書・判断の記録・規則の表・契約表の導出物（`contracts/`）・面の生成器・`tests/floor_cases.yaml`・土台のほかの file。

### (c) 歯（関数名 f140_・base で 0 件）

1. **f140_derived_placement_names_no_vessel（単体の歯・`crates/folio/src/floor_note.rs` の末尾に tests の区間を新しく置く）。** 床の木 FLOOR を共有の導出 `crate::floor::derive` に通した字の中で、字下げ 4 の placement の行がちょうど 1 行在り、その字が (b) の 1 の字（歯の中で手で写した凍結の針）と等しく、導出の全体に句 （器 scribe2） が無いこと。**base では字が違う＝RED。** 定数を直接見るので、生成区間を書き直し忘れた便でなく、定数そのものの字の誤りを落とす。
2. **f140_the_note_region_placement_names_no_vessel（`crates/folio/tests/schema.rs` の末尾）。** 実の `design-intent/design-note/schema.yaml` と凍結 anchor `tests/fixtures/schema/note-region.txt` のそれぞれで、字下げ 4 の placement の行がちょうど 1 行で、(b) の 1 の字の行（改行つき）を含み、句 （器 scribe2） を含まないこと。**base では古い字＝RED。**
3. **歯の置き場の上限。** `tests/schema.rs` は歯 f89（`crates/folio/tests/schema_docs.rs` の f89_schema_teeth_are_split_and_under_the_cap）が器の式（幅 120 正規化・空行 1）で 700 行の上限を持ち、base で 684 行である。**実装は `tests/schema.rs` を 700 行以下に保つ（歯 f89 が落とす上限＝実装が守る基準）。base から足せるのは 16 行までで、歯と凍結の定数の doc 注を合わせてこの内に収める（収まらなければ歯を単体の歯か別の歯の file へ移す）。** 起草役の模擬は 12 行（歯 11 行と凍結の定数の doc 注 1 行）で 696 行だった。最初の模擬は歯 2 本を `tests/schema.rs` に置いて 736 行になり f89 が落ちたので、定数を見る歯を単体の歯へ移し、`tests/schema.rs` の歯を 1 本にした。`floor_note.rs` の tests の区間は新設（file の中に tests の区間が今は無い）で、新しい file ではない。
4. fixture は新しい file を足さない。歯は実の file と凍結 anchor を読むだけで、写しの一時 dir も作らない。

### (d) 採らなかった形

1. **生成区間を手で直す。** P-6.2 に反し、`folio schema --check` が差分で落ち、次の `--write` が戻す。
2. **置き場の名を v3 に替える（例 消費側（器の次の世代 scribe v3）の repo）。** 名を焼く形が同じで、v3 の器や置き場の名が変わればまた古くなる。語彙の導出物の項の 33 周目の直し（名を焼かない）とも向きが逆になる。
3. **役の名を入れる（例 消費側（M3 の出口の利用者）の repo）。** M3 の出口は ADR-21 が読み替えた一時の役で、folio2 自身の置き場の導出物（`folio derive --dir design-intent --out ../contracts --write` が folio2 の repo の `contracts/example.toml` に書く）にも同じ欄が当たるので、役の名で狭めると folio2 自身の置き場が字から外れる。
4. **生成区間のほかの scribe2 の 13 行も直す。** (a) の 8 のとおり、それらは folio2 の便を運ぶ器（今は scribe2）の型・受付・記録か出所を指し、ADR-21 の後も正しい。直すと所見の外へ範囲を広げ、size も変わる。器が替わるときは語彙の項 器 の定義を先に直す一括が要る（§5 に控える）。
5. **土台の写しを今の実の file の全体で置き換える。** 土台は床の凍結の土台で、古い注を持ったまま多くの歯が数えている。全体を置き換えると便 99 の残差のほかに土台を数える歯が広く動く。placement の 1 行だけにする。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **定数を直して `--write` しただけの写し（凍結 anchor・凍結の定数・土台・便 99 の anchor は base のまま）では、既存の歯が 18 本落ちる（起草役の実測・897 本中）。** どれも本便の write-set の中の直しで緑に戻る。この写しでは graph の f99 は落ちない（f99 は土台の写しを直したときに動き、便 99 の anchor を組み直すと緑に戻る）。

| 歯（file・本数） | 落ちる理由 | 直し方 |
| --- | --- | --- |
| schema の schema_design_note_region_matches_the_frozen_anchor_and_names_the_derive_command・schema_check_matches_the_real_design_note_file_and_its_frozen_digest・schema_check_fails_on_one_byte_drift_inside_the_design_note_region・schema_write_restores_the_design_note_region_and_is_idempotent（4） | 凍結 anchor と NOTE_REGION_BYTES・SHA256 が古い | (b) の 3・4 |
| bin/folio の note::tests::note_floor_derives_the_frozen_anchor_byte_for_byte（1） | 凍結 anchor が古い | (b) の 3 |
| freeze（3）・freeze_root（4）・ids（3）・mechanism_live（2）・floor_cases の floor_cases_all_pass_with_folio（1） | 土台の placement が床の定数と違い、土台を写した床が合格しない | (b) の 5 |

   (b) の 3〜5 まで当てた写し（便 99 の anchor は base のまま）では、残るのは graph の f99_the_independent_script_matches_the_anchor の 1 本だけで、(b) の 6 で緑になる。本便の差分の全部を base に当てた写しで、workspace の nextest は **899 / 899**（base 897 + 単体 1 + schema 1）・clippy 0 警告・床 4 本 rc 0・schema 21 / 21・schema_docs 29 / 29・floor_cases 12 / 12。`tests/schema.rs` は 696 行（上限 700）。
2. **動く凍結 anchor は 2 本。** `tests/fixtures/schema/note-region.txt` の 1 行（placement・17 byte）と、便 99 の `tests/fixtures/schema/node-digest-anchor.txt` の残差の 2 行（行数・byte 数は不変）。ほかの凍結 anchor（ほかの 8 本の生成区間の anchor・`graph-digest-anchor.txt`・面の凍結 anchor・id の一覧の anchor・天井の束の anchor）は 1 byte も変えない。
3. **RED（起草役の実測）。** 歯だけを base に当てた写しで、f140_ の 2 本とも落ちる（単体の歯は左が （器 scribe2） の字・右が新しい字で不一致）。同じ写しで歯 f89 は緑（歯を足しても `tests/schema.rs` は上限の内）。
4. **突然変異（起草役の実測・本便を当てた写しの file を 1 通りずつ変える）。** 5 通りとも、f140_ か既存の歯の 1 本以上が落ちる。

| 変異 | f140_ 単体 | f140_ schema | schema 全体 | note_floor の anchor | floor_cases | f99 |
| --- | --- | --- | --- | --- | --- | --- |
| M1 定数を古い字へ戻す | 落ちる | 緑 | 落ちる | 落ちる | 落ちる | 緑 |
| M2 定数に別の器の名（器 scribe3）を焼く | 落ちる | 緑 | 落ちる | 落ちる | 落ちる | 緑 |
| M3 凍結 anchor を直さない | 緑 | 落ちる | 落ちる | 落ちる | 緑 | 緑 |
| M4 生成区間を書き直さない | 緑 | 落ちる | 落ちる | 緑 | 緑 | 緑 |
| M5 土台を直さない | 緑 | 緑 | 緑 | 緑 | 落ちる | 落ちる |

5. **面と索引の変化。** 便の前後で `folio build --dir design-intent --out <置き場> --write` の出力は 30 file のまま `diff -r` で差 0（設計ノートの欄の決まりの生成区間は面に出ない）。`folio graph --dir design-intent --print` の出力も byte 一致（節点 224・辺 1116・型 15・端が節点でない参照 28）。`folio check --dir design-intent`・`folio inject --check`・`folio derive --dir design-intent --out ../contracts --check` は便の後も rc 0。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 10 本とも印なし。書き換える 7 本 = `crates/folio/src/floor_note.rs`・`design-intent/design-note/schema.yaml`・`tests/fixtures/schema/note-region.txt`・`tests/fixtures/floor_base/design-intent/design-note/schema.yaml`・`crates/folio/tests/schema.rs`・`tests/fixtures/schema/node-digest-anchor.txt`・`crates/folio/tests/graph.rs`。本文不変の 3 本 = `crates/folio/src/note.rs`（verify の絞り込みの語 note_floor_derives_the_frozen_anchor_byte_for_byte を関数名に持つ単体の歯の file）・`crates/folio/tests/schema_docs.rs`・`crates/folio/tests/floor_cases.rs`（verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 2 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/floor_note.rs` | 489 | 1,011 | 509（+20・起草役の歯の書き方による参考値・検証役の歯では 503 / +14） | 991 |
| `crates/folio/src/note.rs` | 916 | 584 | 916（不変） | 584 |

   余地はどちらも S の見積 100 を超える。src の外の参考値は、`tests/schema.rs` 684 → 696（歯 f89 の上限 700）・`tests/graph.rs` 612 → 613・`tests/schema_docs.rs` 1,189・`tests/floor_cases.rs` 761・`design-intent/design-note/schema.yaml` 234 → 234・土台の写し 225 → 225・`note-region.txt` 171 → 171・`node-digest-anchor.txt` 192 → 192。
3. **size は S。** 触る src は `floor_note.rs` の 1 本（字 1 か所 + 単体の歯 1 本）で、増分は +20 行（起草役の歯の書き方による参考値・検証役の歯では +14）。ただし `tests/schema.rs` は便の後に 696 行で、歯 f89 の上限 700 まで残り 4 行である。次に `tests/schema.rs` へ歯を足す便は、歯の file の分割が要る（`tests/schema_docs.rs` も上限 1,200 に対し 1,189 行）。
4. **verify は 8 行**で、done の 8 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f140_` = (c) の 1。
   2. `cargo nextest run -p folio --test schema f140_` = (c) の 2。
   3. `cargo nextest run -p folio --test schema` = 欄の決まりの歯の全部（設計ノートの生成区間と凍結 anchor の byte 一致・NOTE_REGION の 3 つの凍結の値・`--write` の書き直しを含む・参考値 21 本）。
   4. `cargo nextest run -p folio --test schema_docs f89_schema_teeth_are_split_and_under_the_cap` = `tests/schema.rs` が上限 700 行の内。
   5. `cargo nextest run -p folio --bin folio note_floor_derives_the_frozen_anchor_byte_for_byte` = 床の木の導出が凍結 anchor と byte 一致。
   6. `cargo nextest run -p folio --test graph f99_the_independent_script_matches_the_anchor` = 独立の script で組み直した便 99 の anchor と要約値の一致。
   7. `cargo nextest run -p folio --test floor_cases` = 土台を写した床の case の全部（`tests/floor_cases.yaml` の expected_cases は base で 146・歯は参考値 12 本）。
   8. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は schema・schema_docs・graph・floor_cases の 4 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f140_ を関数名に持つ src は `floor_note.rs`、note_floor_derives_the_frozen_anchor_byte_for_byte を持つ src は `note.rs` で、どちらも write-set に在る。

### (g) 門と受付

1. **門。** 本便は設計文書の正本の生成区間を書き換えるので門の対象である。起草役の実測は base の設計文書でも本便を当てた写しの設計文書でも **0（通す）** で、断りの字は §0 の 門 のとおり。
2. **周の引き金の外であること。** 周の引き金（判断の記録 ADR-18 決定 (1)・天井の正本 `design-intent/ceiling.yaml` の trigger の閉じた一覧）は、憲法・判断の記録・要件書・規則の表・天井の正本の規範の欄だけを持ち、設計ノートの欄の決まり（design-note）を 1 つも持たない。生成区間の導出は規範の欄の変更ではなく、引き金の要約値は変わらない。門が便の後の写しでも 引き金の要約値が同じ で 0 を返したことが、その実測である。したがって本便の後に天井の周を回し直す要は無い（次の引き金の周が読む）。
3. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測・§1 (h) の 6）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d140-draft.md`、模擬の差分と script は同じ dir の d140-scripts（repo には入れない）。

1. 模擬: base の写しの根で apply-140.sh（実装 → `folio schema --write` → 土台 → anchor-140.py → 凍結の定数 → 便 99 の anchor → 歯の順）を撃つ。その差分が c140.patch。workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との `diff -r`・`folio graph --print` の比較・verify の 8 行を撃つ。
2. RED: apply-140-teeth.py だけを base に当てると (e) の 3 のとおり（r140-teeth.patch）。定数だけを直して `--write` すると (e) の 1 の 18 本が落ちる。
3. 突然変異: mut-140.sh（(e) の 4・戻した file の mtime を新しくして組み直させる）。
4. 余地: lines-140.py と lines-140.awk（同じ式の 2 実装）を repo の根で write-set の src の file に当てる。
5. 生成区間と anchor の要約値: python の hashlib と `sha256sum` の 2 実装。
6. 受付の先撃ち: 契約を commit した後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-140.md#em`。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 生成区間のほかの scribe2 の 13 行（(a) の 8・(d) の 4）。土台の古い注。語彙・要件書・判断の記録・規則の表の字。面・索引・契約表の導出物。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 次の周の整合の観点が F-2 を解けたと読むかは周の結果でしか分からない。消費側 の字が v3 の置き場の読み手にとって十分かは、v3 の置き場の導出物が着地する（ADR-21 の判定点 ②）まで確かめられない。
3. **撤退条件。** (1) 本便の後に、(e) の 1 の 18 本と f99 の 1 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の設計文書で組んだ `folio build` の出力が、着地の直前の main の binary の出力と file 数か byte で 1 つでも違うか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、定数の placement の字・生成区間の数（137 行・16,806 byte）・土台の placement の行・便 99 の anchor の残差の 2 行・歯 f89 の上限 700 と `tests/schema.rs` の行数のどれかが base と違えば、(a)(b)(e) の値を (h) の手順で数え直してから運ぶ（placement の字が既に直っていたら止めて席へ返す）。

## 2. 範囲

- 入れる: `crates/folio/src/floor_note.rs` の placement の字 1 か所と単体の歯 1 本（tests の区間の新設）。`folio schema --write` による `design-intent/design-note/schema.yaml` の生成区間の 1 行。凍結 anchor `note-region.txt` の 1 行。`tests/schema.rs` の凍結の定数 2 つと歯 1 本。土台の写しの placement の 1 行。便 99 の anchor の残差の 2 行と `tests/graph.rs` の定数 1 つ。
- 入れない: 生成区間のほかの行・ほかの 8 本の生成区間・`note.rs` と `schema.rs`（src）と `main.rs` の本文・土台のほかの字・語彙と要件書と判断の記録と規則の表・面と索引・`contracts/`・`tests/floor_cases.yaml`・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| constant | 床の定数の字 | `crates/folio/src/floor_note.rs` の FLOOR の derived.placement |
| region | 生成区間 | `design-intent/design-note/schema.yaml` の schema 節（`folio schema --write` が書く・137 行・16,789 byte） |
| anchor | 凍結 anchor | `tests/fixtures/schema/note-region.txt` と `tests/schema.rs` の NOTE_REGION_BYTES・NOTE_REGION_SHA256 |
| base | 床の凍結の土台 | `tests/fixtures/floor_base/design-intent/design-note/schema.yaml` の placement の 1 行 |
| residual | 便 99 の残差 | `tests/fixtures/schema/node-digest-anchor.txt` の 2 行と `tests/graph.rs` の F99_ANCHOR_SHA256 |
| teeth | 歯 | 単体の f140_ 1 本と schema の f140_ 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main 021b7a4・便 138 は着地済みで write-set の重なり 0・§0）。
- 並行の便: 便 139（起草中）とは重なり 0（§0）。受付は逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.208）を閉じる。次の周の整合の観点で F-2 と同じ所見が立たないかを見る。生成区間のほかの scribe2 の 13 行は器（今は scribe2）を指す字なので残したが、v3 の器が folio2 の便を運ぶようになるときは、語彙の項 器 の定義を直す一括と合わせてその 13 行を見直すかを、台帳の控えに残すかを決める。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "em"
title = "一括 20 の B-5（天井の 34 周目 文書どうしの整合 F-2・台帳 f2-648.208）: 設計ノートの欄の決まり design-intent/design-note/schema.yaml の生成区間が、導出物の置き場 derived.placement を 消費側（器 scribe2）の repo と書き、判断の記録 ADR-21 決定 (1) と制約 CON8 の後（導出物は消費側＝v3 の設計の置き場の repo に着地する）では置き場の名が合わない問題を直す。正本である実装の型付きの定数（crates/folio/src/floor_note.rs の FLOOR の derived.placement）の字から括弧（器 scribe2）だけを外して 消費側の repo に版管理で置く。…（以下は今のまま）とし、folio schema --write で生成区間を導出し直す（語彙の導出物の項を 33 周目で 器〔便を運ぶ道具〕 に直したのと同じ、置き場の名を焼かない向き）。凍結 anchor tests/fixtures/schema/note-region.txt を字面の置き換えで直して tests/schema.rs の NOTE_REGION_BYTES と NOTE_REGION_SHA256 を写し、床が値を突き合わせる床の凍結の土台の写しの placement の 1 行を直し、その巻き添えで便 99 の node-digest-anchor.txt の残差の 2 行を独立の script で組み直して tests/graph.rs の F99_ANCHOR_SHA256 を写す。生成区間のほかの行（器 scribe2 を指す 13 行を含む）・ほかの 8 本の生成区間・面・索引・語彙・要件書は変えない。歯は単体の f140_ 1 本と schema の f140_ 1 本。周の引き金（ADR-18）の外で、門は通す"
req = ["FR19"]
section = "1"
write-set = ["crates/folio/src/floor_note.rs", "design-intent/design-note/schema.yaml", "tests/fixtures/schema/note-region.txt", "tests/fixtures/floor_base/design-intent/design-note/schema.yaml", "crates/folio/tests/schema.rs", "tests/fixtures/schema/node-digest-anchor.txt", "crates/folio/tests/graph.rs", "crates/folio/src/note.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/floor_cases.rs"]
verify = ["cargo nextest run -p folio --bin folio f140_", "cargo nextest run -p folio --test schema f140_", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test schema_docs f89_schema_teeth_are_split_and_under_the_cap", "cargo nextest run -p folio --bin folio note_floor_derives_the_frozen_anchor_byte_for_byte", "cargo nextest run -p folio --test graph f99_the_independent_script_matches_the_anchor", "cargo nextest run -p folio --test floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f140_（床の木 FLOOR の導出の placement の行がちょうど 1 行で §1 (b) の 1 の字と等しく、導出の全体に（器 scribe2）の句が無い）が緑、schema の f140_（実の設計ノートの欄の決まりと凍結 anchor note-region.txt のそれぞれで placement の行がちょうど 1 行で新しい字を含み（器 scribe2）の句を含まない）が緑、schema の歯の全部（設計ノートの生成区間 137 行 16789 byte と凍結 anchor の byte 一致と sha256sum で測り直した要約値の一致と --write の書き直しを含む）が緑、schema_docs の f89 の tests/schema.rs の上限 700 行が緑、床の木の導出と凍結 anchor の byte 一致の単体の歯が緑、独立の script で組み直した便 99 の anchor（残差の 2 行だけが変わる）と F99_ANCHOR_SHA256 の一致が緑、土台を写した床の floor_cases の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が 9 file とも一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数も byte も変わらない"
<!-- contracts:end -->
