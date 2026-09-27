# 設計: 便 166 — 要件書の面が要件の行の adrs（判断の記録）を根拠の札に判断の記録の面へのリンクで出す（台帳 f2-648.248・f2-648.245 の便 A2・S）

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器で出す）。規範文・確かめ方・受入基準は変えない。
- 条: P-4.1 / P-4.2（面の無い判断の記録をリンクにせず「まだ分からない」と出す）/ P-15.2（形の判定は床と同じ式）/ P-6.3（判断の記録の番号のリンクは本文と同じ 1 つの形）/ P-2.4（部品を足さない）。
- 出所: 調べ `.local/share/folio2/handoff-2026-09-27/facestops-study.md`（表の 8・§5・§6 の A2）と台帳 **f2-648.248**（席の裁定 2026-09-27 10:57 JST: 推奨を全部採る）。便 B1（f2-648.249）が根拠の欄 basis を条だけに縛る前に要る。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `fm` が指す §1 だけ。write-set は 3 本（新しい歯の file 1 本を含む・新しい dir と fixture は無い）。
- 門: 対象外。作業ツリー planner-d166 の一番上で base の binary に write-set 3 本を渡すと **0（通す・設計文書の正本を書き換えない便）**。base と本便の写しでも 0。
- 前の便: **base = main 8472b5c（便 161 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 並行の便: 便 165（f2-648.247・A1）とは `crates/folio/src/face_srs.rs` が重なる（(g)）。
- 改訂 a: 独立の検証（d166-verify.md・通してよい・blocking 0）の非 blocking 1 と 5 を採った。歯 1 に非機能要件の行と rules の欄を、歯 2 の adrs を記録の無い id が先の並びにし、検証役の変異 V1〜V3・V6 を落とす（(c)(e)）。(d) の 4 と (g) の 3 の字を直した。src・write-set・verify・size は変えない。

## 1. 設計

### (a) いま起きていること（参考値・base 8472b5c）

1. **面は adrs を 1 つも描かない。** 要件の行の adrs は判断の記録だけを受ける欄で、判断の記録 ADR-13 決定 (3-b)（ア）が足し、床は便 90 から形（check.rs）と実在（link.rs の A-2）を数える。ADR-13 は面について「（イ）（ウ）を当面出さない」とだけ書き、（ア）を出さない判断は無い。ところが `crates/folio/src/face_srs_items.rs` の item_row は根拠の札（部品 hint「根拠」）を goals・basis（条）・rules からだけ組み、adrs を読まない。
2. **観測（base の binary）。** `folio init` の骨格に要件 2 本（FR1 は basis 空・adrs [ADR-1]、FR2 は basis [P-1]・adrs [ADR-1]）を足して commit し、build --write を撃った。素の check は骨格と同じ まだ分からない（違反 0・まだ分からない 2）、build は rc 2 で 6 file を書く。srs.html に ADR-1 は **0 回**で、FR1 には根拠の札が無く、FR2 の札は条だけ。
3. **folio2 自身。** adrs を持つ行は 18 本（FR 17・NFR 1）・id は 34（相異なる 16）。面の根拠の札 28 個のどれにも判断の記録は出ない（注の散文の番号は便 135 の後処理でリンクになる）。
4. **tsuzuri。** 要件の行 20 本が ADR-7 を basis と adrs の両方に書く。面は basis の ADR-7 で止まる（調べの 8）。便 B1 の後に利用者が ADR-n を basis から外すと、本便が無ければ判断の記録の参照が面から消える。
5. **面の無い判断の記録。** adrs の id の記録が adr/ に無ければ床は A-2 の違反（「判断の記録 ADR-9 が実在しない」）を出し、build --write は 1 file も書かない（rc 1）。床を回さないのは `folio face` と `build --check` だけ。
6. **base の歯。** workspace の nextest 985 / 985・clippy 0 警告・床 4 本 rc 0・build 34 file。`f166_` と行 id `fm` は 0 件で、歯の file face_srs_adrs は base に無い。

### (b) 直す先

1. **`crates/folio/src/face_srs_items.rs` の item_row。** rules の後に adrs を読む（無い・null は何もしない）。各項は次の順。
   1. 形: 床の check.rs の adrs の確かめと同じ式（`ADR-` で始まり adr.rs の is_basis_id）。外れたら Err「<場所>: adrs「<id>」が判断の記録の id の形でない」＝面は まだ分からない で書かない（床も [schema] で落とす＝答えが揃う）。is_basis_id は face_srs_items.rs から adr.rs への下向きの辺（面 4 → 検査 2）で、区切りの歯 tests/modules.rs は緑のまま。
   2. 面が在る（face.rs の adr_face が Some）なら、字「判断の記録 」の後に番号だけを包むリンク（class xref・行き先 adr-<数>.html・リンクの字は ADR-<数>）。本文の番号の後処理 link_ids と同じ形で、便 135 の歯が数える形（リンクの字 = 番号）。
   3. 面が無い（None）なら `判断の記録 ADR-<数>（まだ分からない）`（リンクにしない）。
   4. 同じ根拠の札の中で、目標・条・rules 行の後に「・」で続ける。根拠が判断の記録だけの行にも札が 1 つ出る。
2. **P-4.1 の判断: 面の無い id は「（まだ分からない）」。** 同じ面の図の根拠（face_srs.rs の ref_link・便 135）と判断の記録の面の根拠（face_adr.rs の resolve）が型付きの欄で既にこの形を採る。リンクにすると行き先の無いリンクを「在る」と見せ（P-4.1）、黙って落とすか字のままだと、面は分からないことを表に出さない（P-4.2）。
3. **dir を渡す。** adr_face は置き場の path を要るので、fr_chapter(o, ctx, dir, m)・nfr_chapter(o, ctx, dir)・item_row(o, ctx, dir, it, nfr) に足す（scope_chapter(o, ctx, dir, s, m) と同じ並び）。`crates/folio/src/face_srs.rs` は derive の 2 行（呼び出し）だけ変わる。
4. **部品は増やさない。** 根拠の札は部品 hint のまま、リンクは class xref のまま。新しい札の名札・class・部品目録の行は無い（P-2.4）。
5. **変えないもの。** 床（check.rs・link.rs）・rail と制約の basis（条だけ）・本文の番号の後処理・ほかの面・init の雛形・設計文書の正本。

### (c) 歯（関数名 f166_・新しい file `crates/folio/tests/face_srs_adrs.rs`・binary 経由）

口 skeleton が、一時 dir の根で git の init をし、`folio init` の骨格の `requirements: []` を要件の行に替えて commit する（fixture dir は足さない・d163 の f163_signed と同じ口）。骨格が counts を持つ間は行の数に揃え、持たなければ当たらない（便 165 の前でも後でも同じ歯）。期待する字（hint の札・リンク）は歯の側で組む。

1. **f166_adrs_link_to_the_adr_face_in_the_basis_chip。** (a) の 2 の FR1・FR2 に、FR2 の rules [R-2] と非機能要件の行 NFR1（basis 空・adrs [ADR-1]）を足す。check は rc 2 で違反 0、build --write は rc 2 で adr-1.html を書く。srs.html の FR1 と NFR1 の根拠の札の中身がちょうど「判断の記録 」+ 行き先 adr-1.html・字 ADR-1 のリンク（札ごとの一致は 2 回）。FR2 の札は条のリンクで始まり、rules R-2 のリンク・「・」・同じ字で終わる（札の末尾）。面の中の行き先 adr-1.html のリンクは 3 つ・根拠の札は 3 つ。**base では FR1 に札が無い＝RED。**
2. **f166_adrs_without_a_record_are_not_yet_known。** FR1 の adrs [ADR-9, ADR-1]（ADR-9 の記録は無い・正本の順は記録の無い id が先）。check は rc 1 で「[A-2] srs.yaml: requirements[0].adrs[0]: 判断の記録 ADR-9 が実在しない」、build --write は rc 1 で配信先を作らない。`folio face --face srs --write` は rc 0 で、札の中身がちょうど「判断の記録 ADR-9（まだ分からない）」・「判断の記録 」+ ADR-1 のリンク（正本の順のまま）、adr-9.html の字が面に無い。**base では札が無い＝RED。**
3. **f166_adrs_outside_the_adr_form_stop_the_face_like_the_floor。** adrs [ADR-1, <id>] の <id> を P-1・ADR-01（0 詰め）・R-2 の 3 通り。check は rc 1 で「の FR1 の adrs「<id>」が判断の記録の id の形でない」、face は rc 2 で「requirements[0].adrs[1]: adrs「<id>」が判断の記録の id の形でない」を持ち、面を書かない。**base では face が rc 0 で書く＝RED。**

「今の面を壊さない」ことは既存の歯が持つ: face_srs.rs の f135_every_adr_mention_on_the_real_srs_is_a_link と face_srs_body.rs の face_srs_census_on_the_real_sources_counts_and_verbatims（判断の記録の番号は全部、字が番号そのもののリンク）、face_srs_body.rs の face_srs_write_matches_the_frozen_fixture（adrs を持たない凍結 fixture と byte 一致）。

### (d) 採らなかった形

1. **番号を字のまま札に入れ、本文の後処理 link_ids にリンクを任せる。** face_srs.rs を書かずに済み便 165 と重ならないが、面の無い id が黙って字のまま残る（P-4.2・(e) の M3）。
2. **面の無い id で面全体を止める。** 床が既に違反で落とし build --write は書かない。1 面の生成まで止めると、どの行のどの id かを面で示せない。
3. **名札ごとリンクで包む（リンクの字が「判断の記録 ADR-1」）。** 本文の番号のリンクの形（字 = 番号・便 135）と割れ、既存の歯 2 本が落ちる（M9）。
4. **判断の記録の題を添える（`ADR-1（<題>）`）・別の札「判断の記録」を立てる。** 題は 1 id ごとに別の正本を読み、面の止まる所を増やす（題はリンクの先に在る・rules 行の札も題を持たない）。別の札は新しい名札で、ADR-13 決定 (3-b)（ア）が根拠の欄との使い分けとして足した、要件の根拠を示す欄という位置づけから離れる（M5）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は 0 本。** 本便を当てた写しで workspace の nextest 988 / 988・clippy 0 警告・床 4 本 rc 0。rustfmt --check の差の塊は base と同じ（face_srs.rs 4・face_srs_items.rs 1・新しい file 0）。名札ごと包む最初の模擬では上の 2 本が落ちた（控え first-draft/）ので、番号だけを包む形に直した。
2. **RED。** 歯だけを base に当てると f166_ の 3 本とも落ちる（どれも組み立ての口ではなく札・rc の比べで・控え red.log に本文）。
3. **突然変異。** 写しの face_srs_items.rs だけを 1 通りずつ変え、face_srs_adrs・face_srs・face_srs_body を撃った。**M1〜M9 と、検証役の V1〜V3・V6（改訂の前の歯をすり抜けた 4 つ）は全部落ちる。**

| 変異 | 落ちる歯 |
| --- | --- |
| M1 adrs を読まない | 1・2・3 |
| M2 面の無い id もリンクにする / M3 面の無い id を字のまま出す / M4 面の無い id を黙って落とす | 2 |
| M5 判断の記録を別の札「判断の記録」に出す / M6 判断の記録を札の頭に積む | 1・2 |
| M7 形を面の行き先の式（ADR- + 数字・0 詰めを受ける）で見る / M8 形でない id も札に出す | 3 |
| M9 名札ごとリンクで包む | 1・2・f135_every_adr_mention_on_the_real_srs_is_a_link・face_srs_census_on_the_real_sources_counts_and_verbatims |
| V1 非機能要件の行では adrs を読まない / V3 判断の記録を rules 行の前に置く | 1 |
| V2 adrs を id の字の順に並べ替える / V6 面の在る id を先に寄せる | 2 |

   表の 1・2・3 は (c) の 1・2・3。

### (f) 大きさ・verify と done の対応

1. **write-set（3 本）。** `crates/folio/src/face_srs_items.rs`・`crates/folio/src/face_srs.rs`（derive の 2 行）・`+crates/folio/tests/face_srs_adrs.rs`（新規）。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_srs_items.rs` | 390 | 1110 | 410（+20） | 1090 |
| `crates/folio/src/face_srs.rs` | 1068 | 432 | 1068（±0・2 行を書き換え） | 432 |

   src の外は `tests/face_srs_adrs.rs` 0 → 250。
3. **size は S。**
4. **verify は 2 行**で、done の 2 の塊と 1 対 1: `--test face_srs_adrs f166_`（(c) の 1〜3・base は歯の file が無く rc 101）・clippy（base も本便も 0 警告）。(c) の末尾の既存の歯と workspace の全部は `.vessel.toml` の common-verify が撃つ。`--bin` の行は無い。

### (g) 門・受付・並行の便

1. 門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0。
2. **便 165（A1・起草中）と face_srs.rs が重なる。** 165 は derive の check_counts の 1 行を消し、本便は同じ関数の 6 行下の 2 行（fr_chapter・nfr_chapter の呼び出し）を変える。165 の起草中の差分（控え c165-at-draft.patch）と本便の差分は base にどちらの順でも当たり、両方を当てた写しで本便の歯・165 の歯 outside_faces・face_srs・face_srs_body の 50 本が緑（165 の骨格は counts を書かない＝(c) の口が当たらない側）。改訂 a の歯と 165 の後の写し（控え c165-latest.patch）でも、どちらの順でも当たり 52 本が緑。受付は重なりに従って 1 本ずつでよい（順は問わない）。
3. 便 162（41 本）・便 163（face_note.rs・face_labels.rs・tests の site / face_note / face_index）・一括 30（design-intent と tests の check / face_constitution / face_srs）とは重ならない。一括 30 は srs.yaml の FR22 と FR25 の adrs に ADR-28 を足す（行は 18 のまま・id は 34 → 36）。一括 30 が先に着けば (h) の 1 の 34 個は 36 個になるが、撤退条件 (2) は着地の直前の main と比べるので判定は変わらない。便 164 は起草の時点で契約が無い。

### (h) 今の置き場の床と面（撤退条件 (2) の実測）

1. **folio2 自身。** 床 4 本と凍結の 4 つの旗の出力（標準出力・標準エラー・rc）は base と本便で同じ（build の行の byte 数だけが違う）。build の 34 file のうち **srs.html だけが違い、差は根拠の札の中だけ**: 18 本の行の札に判断の記録が 34 個（全部リンク・まだ分からない 0）・札の数は 28 のまま・札の中の判断の記録を除くと base と byte 一致・+2146 byte。
2. **tsuzuri。** 写しで調べの 9 か所を避けた（8 は basis から ADR-n を外し adrs は残す・tsuzuri の repo では何も書かない）要件書の面で、20 本の行の札に判断の記録が 21 個（全部リンク）、ほかは base と byte 一致。
3. **骨格。** init の骨格のままの素の check と build は base と本便で同じ（骨格は要件を持たない）。

### (i) 要件との関係・運ばないもの・撤退条件

1. **要件との関係（正本は書き換えない）。** FR4 の規範文と確かめ方は変えない。ADR-13 決定 (3-b)（ア）の欄を面が出していなかった穴を埋めるだけで、行 D-17 の持ち主の承認は要らない。
2. **運ばないもの。** basis に書いた判断の記録を面が止めずに通す寛容さ（調べの 8 は便 B1 が床で落とす）・判断の記録の題・rail と制約の根拠・判断の記録の面から要件への逆の辺。
3. **撤退条件。**
   - (1) 既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。
   - (2) 着地の後の main で、次のどれかが着地の直前と違ったら止めて席へ返す: folio2 自身の床 4 本の結果・`folio build` の出力のうち **srs.html の根拠の札の中の判断の記録を除いた全部**（ほかの 33 file は byte 一致・srs.html は札の中の差だけ・数え方は控えの chips-166.py）。
   - (3) 受付の時点の main で、(b) が使う item_row・fr_chapter・nfr_chapter・derive の呼び出し・face.rs の adr_face・check.rs の adrs の確かめ・init の雛形の `requirements: []` が base と違えば、数え直してから運ぶ（便 165 の着地の後なら (g) の 2 の写しの数を使う）。

## 2. 範囲

- 入れる: §1 (b) の 1・3、(c) の歯 3 本と口 skeleton。
- 入れない: 床（check.rs・link.rs）・rail と制約の basis・ほかの面・init の雛形・設計文書の正本・新しい fixture と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| adrs-chip | 根拠の札の判断の記録 | item_row が adrs の各 id を床と同じ式で形を見て、面が在れば番号だけのリンク・無ければ「（まだ分からない）」で根拠の札に積む |
| teeth | 歯 | tests/face_srs_adrs.rs の f166_ 3 本・口 skeleton |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い（便 165 とはどちらが先でもよい）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 .248 を閉じる。便 B1（.249）の受付の前に、tsuzuri が basis から外す ADR-n が adrs に在ることを確かめる（(a) の 4・20 本は両方に在る）。
- 控え: `.local/share/folio2/handoff-2026-09-27/d166-draft.md` と `d166-scripts/`（c166.patch・r166-teeth.patch・模擬・変異・床と面の比べ）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fm"
title = "f2-648.245 の便 A2（台帳 f2-648.248）: 要件書の面（crates/folio/src/face_srs_items.rs の item_row）は要件の行の adrs（判断の記録だけを受ける欄・ADR-13 決定 (3-b)（ア））を 1 つも描かない。rules の後に adrs を読み、各 id を床の check.rs と同じ式（ADR- で始まり adr.rs の is_basis_id）で確かめ（外れたら面は まだ分からない）、face.rs の adr_face で面が在れば 判断の記録 の後に番号だけを包む xref のリンク（本文の番号と同じ形・便 135）、無ければ 判断の記録 ADR-n（まだ分からない）として、同じ根拠の札（部品 hint）の中に「・」で続ける。fr_chapter・nfr_chapter・item_row に dir を足し、face_srs.rs は derive の呼び出し 2 行だけ変える。部品・床・init の雛形・設計文書の正本は変えず、folio2 の面は srs.html の根拠の札の中だけが変わる。歯は新しい tests/face_srs_adrs.rs の f166_ 3 本（folio init の骨格を一時 dir に書く）。門の対象外。base = main 8472b5c・受付の時点の main で数え直す"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_srs_items.rs", "crates/folio/src/face_srs.rs", "+crates/folio/tests/face_srs_adrs.rs"]
verify = ["cargo nextest run -p folio --test face_srs_adrs f166_", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/face_srs_adrs.rs の f166_ の 3 本（folio init の骨格に FR1〔basis 空・adrs ADR-1〕と FR2〔basis P-1・rules R-2・adrs ADR-1〕と NFR1〔basis 空・adrs ADR-1〕を足すと、素の check は rc 2 で違反 0、build --write は rc 2 で adr-1.html を書き、srs.html の FR1 と NFR1 の根拠の札の中身がちょうど 判断の記録 と adr-1.html への字 ADR-1 のリンク、FR2 の札は条のリンクと rules R-2 のリンクの後に「・」で同じ字が続き（札の末尾）、面の中の adr-1.html へのリンクは 3 つ / adrs を記録の無い ADR-9 が先の ADR-9, ADR-1 にすると、check は rc 1 で A-2 の requirements[0].adrs[0] の 判断の記録 ADR-9 が実在しない、build --write は rc 1 で配信先を作らず、folio face --face srs は rc 0 で札の中身がちょうど 判断の記録 ADR-9（まだ分からない）・ADR-1 のリンクの順で adr-9.html の字が無い / adrs に P-1・ADR-01・R-2 を足すと、check は rc 1 で 判断の記録の id の形でない を持ち、folio face は rc 2 で requirements[0].adrs[1] の同じ字を持ち面を書かない）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数が同じで、srs.html の根拠の札の中の判断の記録を除いて byte 一致"
<!-- contracts:end -->
