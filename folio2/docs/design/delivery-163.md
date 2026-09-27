# 設計: 便 163 — 設計ノートの面が承認欄を床と同じく項の一覧として読み、項ごとに署名の行を出す（台帳 f2-648.243・S）

- 要件: FR9（設計ノートを 1 つの型で生成する・正本の形は構造の床〔FR5〕で数える）と FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は設計ノートの面の承認欄と日付の読み方を床に揃え、入口の設計ノートの棚のカードの日付も同じ口から取る（便 147 と同じ扱い）。規範文・確かめ方・受入基準は変えない。
- 条: P-15.2（止める判定と事後の確かめは同じ式＝床と面は同じ読み方）/ P-6.3（同じ内容を 2 つの面に持たない）/ P-4.2（読めない形は まだ分からない として出す）/ P-3.3（床の合格を面の用意と取り違えない）。
- 出所: 台帳 **f2-648.243**（席の tsuzuri の写しの通し・2026-09-27 09:58 JST・main 8472b5c・tsuzuri a92980e）。依頼は席から起草役へ（2026-09-27・最優先＝利用者 tsuzuri の面の生成を塞いでいる）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `fj` が指す §1 だけ。write-set は 5 本（書き換えるだけ）で、新しい file も dir も fixture も無い。
- 門: 対象外。作業ツリー planner-d163 の一番上で base の binary に write-set 5 本を渡すと **0（通す・設計文書の正本を書き換えない便）**。base と本便の写しでも 0。
- 前の便: **base = main 8472b5c（便 161 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 並行の便: 便 162（docs/d162）と一括 30（docs/batch29）とは write-set が重ならない（(g)）。
- 改訂 a: 独立の検証（d163-verify.md）の B1 と N1〜N5 を採った。歯の承認欄を 2 項とも 持ち主 × R-8 にし（便 162 の組の判定の前でも後でも床が通す）、2 項の段を日付の降順に、表でない項を持つ一覧の段と後の項が古い一覧の段を足した（(c)・(e) の V4〜V6）。冒頭の FR4 を見出しで引き、f146 の単体の歯の注を直した。write-set・verify・size は変わらない。

## 1. 設計

### (a) いま起きていること（参考値・base 8472b5c）

1. **床と面が同じ欄を違う形で読む。** 設計ノートの承認欄 meta.approval を、床（`crates/folio/src/note.rs` の check_meta）は**項の一覧**として読み（口 row_list・表 1 つは「表の一覧でない」の まだ分からない）、面は**表 1 つ**として読む。面の読み手は 3 か所: `crates/folio/src/face_note.rs` の status（発効の状態の行の日付）と approval_chapter（署名の行）、`crates/folio/src/face_labels.rs` の approval_date を経た note_dated（面の鮮度の札・版の札・足の行と、入口の設計ノートの棚のカード）。
2. **tsuzuri で面が 1 枚も出ない。** tsuzuri の design-intent と contracts を写しへ cp して撃った（tsuzuri の repo では何も書かない）。写しのままの素の check は不合格 2（天井の正本の生成区間が古い）で、`folio schema --write` → commit の後は**合格 0/0**・schema --check と derive --check も 0。その上で `folio build --write` は **rc 2「design-note/surface.yaml.meta.approval: 表でない」で 1 file も書かない**。surface.yaml（status effective）は承認欄を一覧 1 項で書く。
3. **folio2 の写しでも同じ（歯の形の実測）。** 実の design-intent の写しに発効の最小の設計ノート 1 本を足して commit し、承認欄の形だけを替えた。

| 承認欄 | 素の check（base も本便も同じ） | build --write（base） | build --write（本便） |
| --- | --- | --- | --- |
| 発効・一覧 1 項 / 2 項 | 合格 0/0 | rc 2「表でない」・書かない | rc 0・署名の行 1 / 2 |
| 草案・一覧 1 項 | 合格 0/0 | rc 2「表でない」・書かない | rc 0・署名の行 1 |
| 発効・表 1 つ | rc 2（違反 1〔承認欄が無い〕・まだ分からない 1〔表の一覧でない〕） | rc 2・**35 file を書く** | rc 2「一覧でない」・書かない |
| 草案・表 1 つ | rc 2（まだ分からない 1） | rc 2・**35 file を書く** | rc 2「一覧でない」・書かない |

   base は、床が通す形で面が止まり、床が通さない形で面が書く。本便の後は答えが揃う。

4. **承認欄の全数（同じ種類の食い違いは設計ノートだけ）。** 実の写しで形を 1 つずつ替えて撃った。

| 承認欄 | 床の読み方 | 面の読み方 | 形を替えたときの答え（base） |
| --- | --- | --- | --- |
| **設計ノート meta.approval** | 項の一覧 | 表 1 つ | 上の表＝**本便で揃える** |
| 判断の記録 approval | 表 1 つ（adr.rs の check_approval） | 表 1 つ（face_adr.rs・adr_dated・憲法の面の改訂の来歴） | 一覧 1 項: 床 不合格「型が違う」・build は書かない＝揃っている |
| 憲法 meta.approval | 表 1 つ（--freeze-start の写しの組み立て）・素の check は形を見ない | 表 1 つ | 凍結の前の骨格で一覧 1 項: 床は骨格のまま・build「表でない」 |
| 要件書・入口 meta.approval | 読まない | 項の一覧（役・承認者・日付の行・last_approval） | 表 1 つ: 床 合格 0/0・build「一覧でない」 |
| 天井・相談窓口 meta.approval | 読まない | 読まない | 表 1 つ: 床も build も合格 |

   「床は一覧・面は表 1 つ（またはその逆）」は設計ノートの 1 か所だけ。憲法・要件書・入口は**床が形を見ず面だけが形を決める**別の種類で、揃えるには床に確かめを足す（利用者と骨格の床の答えが変わる）ので S に収まらない＝(i) の 2。

5. **本便の後に tsuzuri の写しで止まる所（全部）。** 面を 1 枚ずつ `folio face` で撃ち、止まった所を写しの上でだけ手で避けて次を撃った（避け方は測るためで、tsuzuri の直し方の提案ではない）。どれも**床は合格 0/0 のまま面だけが止まる**。

| # | 面 | まだ分からない の字 |
| --- | --- | --- |
| 1 | 入口・要件書 | srs.yaml.meta: 欄 counts が無い |
| 2 | 要件書 | srs.yaml.meta: 欄 promise が無い |
| 3 | 憲法 | constitution.yaml.amendment: 表でない（tsuzuri は散文の block・面は declaration / steps / effective_step の表を読む） |
| 4 | 要件書 | srs.yaml.actors: role が「道具」の actor が 0 で 1 つでない |
| 5 | 要件書 | srs.yaml.actors: 入れる側の帯が 5 で上限 4（部品目録の context-band の max_per_band）を超える |
| 6 | 要件書 | srs.yaml.outputs: 出る側の帯が 5 で上限 4 を超える |
| 7 | 要件書 | srs.yaml: 欄 scope_m1 が無い |
| 8 | 要件書 | srs.yaml.requirements[0].basis[1]: 条 id「ADR-7」が無い（面は basis を憲法の条だけで引く） |
| 9 | 設計ノート surface | 章が 21 本ある＝章が多すぎる（上限 12） |

   9 か所を写しの上で避けると、素の check 合格 0/0・`folio build --write` rc 0（15 file）。判断の記録 8 枚と設計ノート bakeoff-surface の面は避けなくても出る。base の binary では、この 9 か所に加えて承認欄の 1 か所（入口の面と設計ノート 2 枚）でも止まる。

6. **base の歯。** workspace の nextest 985 / 985・clippy 0 警告・床 4 本 rc 0・`folio build --write` の出力 34 file。`--test site` 15 本・`--test face_note` 31 本・`--test face_index` 34 本。`f163_` と行 id `fj` は 0 件。

### (b) 直す先

1. **`crates/folio/src/face_labels.rs`。** 口を 2 つ足し、note_dated をそれに替える。
   - note_approvals: 設計ノートの承認欄の項。無いか null なら空・一覧でなければ Err「meta.approval: 一覧でない」（床の row_list と同じ形を受ける）。
   - note_approval_date: 状態が NOTE_UNREAD（draft・example）なら None、そうでなければ**最後の項**の日付（escape 済み）。承認欄が無いか空なら None。
   - approval_date（表 1 つ）は判断の記録だけの口として残し、注の「・設計ノート」を外す。
2. **`crates/folio/src/face_note.rs`。** status は発効の状態の行の日付を note_approval_date から取る（無ければ今と同じ「effective に承認欄が無い」の Err）。approval_chapter は note_approvals の項ごとに署名の行を 1 つ出す（役 承認・承認者・日付・逐語・裁定 id・今と同じ字の並び）。項が無ければ今と同じ 2 つの文（見本 / 未）。
3. **表 1 つは床と面の両方で断る。** 欄の決まり（design-note/schema.yaml の生成区間の approval）は項の欄（required の 5 つと surface_enum）を書き、入れ物の形を書かない。入れ物の形は床の読み方（row_list・便 23 から一覧）で、tsuzuri の発効の設計ノートも一覧で書く。面を床に揃え、表 1 つは面も まだ分からない にする（build は書かない）。
4. **部品は増やさない。** 署名の行は部品目録の approval-block の中に置く（要件書の面が承認欄の行ごとに署名の行を出すのと同じ形）。
5. **変えないもの。** 床（note.rs）・判断の記録と憲法と要件書と入口の面・欄の決まりとその生成区間・init の雛形・folio2 自身の床と面の出力。

### (c) 歯（関数名 f163_）

1. **f163_note_approval_list_builds_one_sign_per_item（`crates/folio/tests/site.rs`）。** 口 f163_signed が、実の置き場の写しに最小の設計ノート signed.yaml（散文 1 節）を足して commit し、素の check と build --write を撃つ。承認欄の項は 2 つとも 持ち主 × R-8（base の床も、承認者と対話面の組を縛る便 162 の後の床も通す組）。発効・一覧 1 項、発効・一覧 2 項（**日付の降順** 2026-09-23 → 2026-09-21）、草案・一覧 1 項の 3 つで、check も build も rc 0、note-signed.html の署名の行が項の数と同じで**正本の項の順**に並び（字は歯の側で組む）、発効の状態の行が「発効・拘束力あり（承認 <最後の項の日付>）」、鮮度の札が 承認 <最後の項の日付>（2 項の段は最大の 09-23 でなく 09-21・草案は 生成 2026-09-20）。**base では build が「表でない」で rc 2＝RED。**
2. **f163_floor_and_face_refuse_a_single_table_alike（同じ file）。** 発効と草案の表 1 つで、check は rc 2 で「meta の approval が表の一覧でない」を持ち、build は rc 2 で「design-note/signed.yaml.meta.approval: 一覧でない」を持ち、面を書かない。表でない項を持つ一覧（草案・`- 承認する`）でも、check は rc 2 で同じ字、build は rc 2 で「design-note/signed.yaml.meta.approval[0]: 表でない」を持ち面を書かない（項を黙って飛ばさない）。**base では表 1 つで build が面を書く＝RED。**
3. **f163_note_dated_reads_the_last_item_of_the_list（`crates/folio/src/face_labels.rs` の既存の tests の区間・単体）。** 口 note_dated で、発効と廃止の 2 項の一覧 → 承認・最後の項の日付、後の項が前の項より古い一覧 → 最後の項（古い方）の日付、draft と見本 → 生成日、承認欄が無い・null・空の一覧 → 生成日、発効の表 1 つ → Err「meta.approval: 一覧でない」。**base では一覧が「表でない」の Err＝RED。**

**既存の歯の書き換え 4 本**（表 1 つの承認欄の字を一覧 1 項に替えるだけ・確かめる字は変えない）。これらは面の側の読み方を写して、床が通さない形で書かれていた。
- `crates/folio/tests/face_note.rs` の face_note_shows_the_effective_and_retired_states と f146_note_stamp_cover_and_foot_follow_the_approval（変異の字の 1 か所ずつ）。
- `crates/folio/tests/face_index.rs` の口 note_state（f147_note_card_updated_is_the_latest_note_face_date が使う・注の 1 行も）。
- `crates/folio/src/face_labels.rs` の f147_type_dates_and_shelf_updated の note_dated の段（判断の記録の段は表 1 つのまま）。
- あわせて同じ file の f146_named_and_approval_date_pick_the_approval の注 1 行（「承認欄が 1 つの表（判断の記録・設計ノート）」を、設計ノートは note_dated が項の一覧で読む字に）。確かめる字は変えない。

### (d) 採らなかった形

1. **面が表 1 つと一覧の両方を受ける。** 床が まだ分からない とする形から面を書き、build が床の答えと違うものを配信先に置く（(e) の M3）。P-15.2 の向きの食い違いが残る。
2. **床を表 1 つに揃える。** tsuzuri の発効の設計ノートと床の歯（tests/note.rs・便 161 の f161_ を含む）が一覧で、利用者の床の答えが変わる。版を上げるたびの承認を 1 つの表に積めない（要件書・入口は承認の行を積む）。
3. **日付を項の中の最大の日付で取る。** 要件書の面の日付（最後の 承認 の行・便 145）と向きが割れる。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **実装だけを当てると、(c) の書き換えの 4 本が落ちる**（控え sim0-nextest.log）。書き換えを合わせた本便の全部で、workspace の nextest 988 / 988・clippy 0 警告・床 4 本 rc 0。main + 便 162（改訂 b の 2 の模擬）+ 本便の写しでも workspace の nextest 992 / 992・clippy 0 警告・verify 6 行とも rc 0（便 162 の前でも後でも緑）。
2. **RED。** 歯だけを base に当てると、site・face_note・face_index と単体の 230 本のうち f163_ の 3 本と書き換えた 4 本の 7 本だけが落ちる（控え red.log・落ちた本文も）。
3. **突然変異。** 写しの src 2 本だけを 1 通りずつ変え、site・face_note・face_index と単体の全部（230 本）を撃った。**M1〜M6 と検証役の V1〜V8 は全部落ちる。**

| 変異 | 落ちる歯 |
| --- | --- |
| M1 承認の章が最初の項だけを出す / M4 署名の行を逆の順に出す / V1 状態の行だけ最初の項の日付 / V2 署名の行を 1 行に潰す / V3 署名を 1 つに束ねる / V5 署名の行を日付順に並べ替える | 1 |
| M2 日付を最初の項から取る / V4 日付を項の中の最大の日付で取る（(d) の 3） | 1・3 |
| M3 表 1 つを 1 項の一覧と見る | 2・3 |
| M5 draft と見本でも承認欄の日付を読む | 1・3・書き換えた 3 本 |
| M6 草案では署名の行を出さない | 1・2 |
| V6 一覧の中の表でない項を黙って飛ばす | 2 |
| V7 見本だけ承認欄の日付を読む | 3・単体の f147 |
| V8 廃止も承認欄を読まない | 3 |

### (f) 大きさ・verify と done の対応

1. **write-set（5 本・印なし）。** `crates/folio/src/face_note.rs`・`crates/folio/src/face_labels.rs`・`crates/folio/tests/site.rs`・`crates/folio/tests/face_note.rs`・`crates/folio/tests/face_index.rs`。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_note.rs` | 1033 | 467 | 1035（+2） | 465 |
| `crates/folio/src/face_labels.rs` | 658 | 842 | 732（+74・単体の歯を含む） | 768 |

3. **size は S。**
4. **verify は 6 行**で、done の 6 の塊と 1 対 1。
   1. `cargo nextest run -p folio --test site f163_` = (c) の 1・2（base は 0 件で rc 4）。
   2. `cargo nextest run -p folio --bin folio f163_ f147_type_dates_and_shelf_updated` = (c) の 3 と書き換えた単体の 1 本。
   3. `cargo nextest run -p folio --test site` = build の歯の全部（参考値 17 本）。
   4. `cargo nextest run -p folio --test face_note` = 設計ノートの面の歯の全部（書き換えた 2 本を含む・31 本）。
   5. `cargo nextest run -p folio --test face_index` = 入口の面の歯の全部（書き換えた口を使う f147 を含む・34 本）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` の 3 本は write-set に在る。`--bin folio` の 2 つの絞り込みの語を関数名に持つ src は face_labels.rs だけで write-set に在る。

### (g) 門・受付・並行の便

門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0。便 162 の改訂 b は承認者と対話面の組を床で縛る（R-8 は持ち主だけ・席は D-17）ので、(c) の承認欄は 2 項とも 持ち主 × R-8 で書く。main + 便 162（改訂 b の 2 の模擬）+ 本便の写しでも (c) の歯と workspace の全部が緑（(e)）。一括 30 は design-intent を刈り込むので、着地の順によって folio2 の build の出力が一括 30 の分だけ変わる（本便の差ではない・撤退条件 (2) は着地の直前の main と比べる）。

### (h) 今の置き場の床と面（撤退条件 (2) の実測）

1. **folio2 自身。** 設計ノート 2 本は見本で承認欄を持たない。床 4 本・凍結の 4 つの旗の出力（標準出力・標準エラー・rc・書いた file）・build（34 file・2146076 byte）が base と本便で 1 byte も違わない。
2. **tsuzuri。** schema --write の後の写しで、素の check・schema --check・derive --check は base と本便で同じ（合格・0・0）。build は承認欄で止まらなくなり、(a) の 5 の 1 で止まる。
3. **骨格。** init の骨格は base と本便で同じ（素の check rc 2・まだ分からない 2・build は 6 file を書き byte も同じ）。骨格は設計ノートを書かない。

### (i) 運ばないもの・言えないこと・撤退条件

1. **(a) の 5 の 9 か所は運ばない。** 面の側を直すか（folio2 の便）、tsuzuri の正本を直すか（利用者の手番）は席が決める。3（憲法の amendment の形）・8（要件書の basis に判断の記録の id）・9（設計ノートの章の上限 12）は folio2 と tsuzuri の書き方が割れている所で、面の決まりを変えるなら欄の決まりか部品目録の変更になる。
2. **床が形を見ない承認欄（(a) の 4 の憲法・要件書・入口）は運ばない。** 揃えるには床に確かめを足し、利用者と骨格の床の答えが変わる（FR22）。
3. **借り（行 D-11）。** 入れ物の形（項の一覧）は欄の決まりの生成区間に字が無い。書く先の候補は floor_note.rs の FLOOR の注 approval_note（生成区間 design-note/schema.yaml）で、便 162 が同じ注を書き換えるので、その後の一括で席が足す（本便は design-intent を書かない）。
4. **撤退条件。**
   - (1) (c) の書き換えの 4 本のほかに既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。
   - (2) 着地の後の main で、folio2 自身の床 4 本の結果か `folio build` の出力が着地の直前の main と 1 byte でも違ったら止めて席へ返す。
   - (3) 受付の時点の main で、face_note.rs の status と approval_chapter、face_labels.rs の approval_date と note_dated、tests/site.rs の口 real_copy・folio_check・git が base と違えば、数え直してから運ぶ（手順は控え d163-draft.md）。

## 2. 範囲

- 入れる: §1 (b) の 1・2、(c) の歯 3 本と口 2 つ（f163_signed・f163_sign）と既存の歯の書き換え 4 本。
- 入れない: 床（note.rs）・ほかの面・欄の決まりと生成区間・init の雛形・部品目録・fixture と新しい dir・(i) の 1〜3・tsuzuri の repo・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| read | 設計ノートの承認欄の口 | face_labels.rs の note_approvals・note_approval_date（項の一覧・最後の項の日付） |
| sign | 署名の行 | face_note.rs の approval_chapter で項ごとに 1 行（approval-block の中） |
| teeth | 歯 | tests/site.rs の f163_ 2 本・face_labels.rs の f163_ 1 本・書き換え 4 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 .243 を閉じ、(i) の 1〜3 を台帳に起こすか一括に束ねる。tsuzuri の設計席へ、承認欄の止まりが解けたことと (a) の 5 の 9 か所を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fj"
title = "台帳 f2-648.243: 設計ノートの承認欄 meta.approval を床（note.rs）は項の一覧、面（face_note.rs と face_labels.rs の note_dated）は表 1 つとして読むので、一覧で書いた発効の設計ノート（tsuzuri）で床は合格なのに folio build が 表でない で止まる。face_labels.rs に口 note_approvals（項の一覧・一覧でなければ Err）と note_approval_date（draft と見本は読まない・最後の項の日付）を足して note_dated をそれに替え、face_note.rs の状態の行と承認の章を同じ口で読み項ごとに署名の行を出す（部品は approval-block のまま）。表 1 つは床と面の両方が まだ分からない。床・ほかの面・欄の決まり・folio2 自身の出力は変えない。歯は f163_ 3 本と、表 1 つを写していた既存の歯 4 本の書き換え。門の対象外。base = main 8472b5c"
req = ["FR9", "FR4"]
section = "1"
write-set = ["crates/folio/src/face_note.rs", "crates/folio/src/face_labels.rs", "crates/folio/tests/site.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/face_index.rs"]
verify = ["cargo nextest run -p folio --test site f163_", "cargo nextest run -p folio --bin folio f163_ f147_type_dates_and_shelf_updated", "cargo nextest run -p folio --test site", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --test face_index", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/site.rs の f163_ の 2 本（実の置き場の写しに最小の設計ノートを足し、承認欄は 持ち主 × R-8。発効・一覧 1 項と日付の降順の 2 項・草案・一覧 1 項では素の check と build --write が rc 0 で署名の行が項の数と正本の順のとおり・発効の状態の行と鮮度の札が最後の項の日付〔草案は生成日〕/ 発効と草案の表 1 つでは check が rc 2 で 表の一覧でない・build が rc 2 で meta.approval: 一覧でない を持ち面を書かない・表でない項を持つ一覧も check と build が rc 2 で面を書かない）が緑、単体の f163_（note_dated が一覧の最後の項の日付〔後の項が古くても〕・draft と見本と空は生成日・表 1 つは Err）と書き換えた f147_type_dates_and_shelf_updated が緑、tests/site.rs と tests/face_note.rs と tests/face_index.rs の歯の全部（書き換えた 3 本を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
