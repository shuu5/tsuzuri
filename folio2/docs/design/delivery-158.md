# 設計: 便 158 — 承認欄の雛形の印「未記入」を空と見て、日付の形を見る（群 A・台帳 f2-648.235）

- 要件: FR24（始まりの凍結・書いた後はふだんの床が全部の検査を回す）。規範文・確かめ方・受入基準 AC21・AC23 は変えない。
- 条: P-15.2（止める判定の式を事後の検査が同じ関数で確かめる）/ P-12.2（承認は逐語と日付を添えて記帳する）/ N-4.1。
- 出所: 便 155 の独立の検証（`.local/share/folio2/handoff-2026-09-27/d155-verify.md` の不一致 4・非 blocking N-3）と台帳 **f2-648.235**。後続の台帳 f2-648.233（列の根の表・案 (c) 二段の根）の前提「根は承認欄の形を満たす第 1.0 版」を機械の側で締める。
- 置き場: 審査の材料は行 `fe` が指す §1 だけ。write-set は 3 本で、新しい file も dir も無い。
- 門: 対象外。作業ツリー planner-d158 の一番上で base の binary に write-set 3 本を渡すと **0（通す・設計文書の正本を書き換えない便）**。base と本便の写しでも 0。
- 前の便: **base = main a6a868d（便 157 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。便 157 は write-set が重ならない。
- 改訂 b: 独立の検証（d158-verify.md）の B-1 と非 blocking 1〜3 を採った。歯の形を足し（(c) の 2・3）、変異の表（(e)）・原理の字（(b) の 3）・借り（(i) の 2）を直した。

## 1. 設計

### (a) いま起きていること（参考値・観測は eecbff9 で撃った・a6a868d との差は check.rs の値域の節だけ）

1. **凍結の前も後も「未記入」を空と見ない。** 床の承認一覧の確かめ（`crates/folio/src/anchor.rs` の check_approvals・便 155 から凍結の前にも掛かる）は、承認者・裁定 id・逐語が空でないことと、裁定 id の台帳 id の形だけを見る。init の雛形の印「未記入」は空でない字として通り、日付は見ない。
2. **形ごとの答え。** `folio init` の骨格の憲法の承認欄（meta.approval）だけを 11 の形に変え、列の根の表に名の行を足した**測りだけの binary**（契約に入れない・行 D-11）で --freeze-start → commit → 素の check を撃つと、**11 形とも凍結して凍結の後の床も 0**。
   - 印の 4 形: E（台帳 .235 の形＝裁定 id だけ p1-1・ほかは 未記入）・承認者だけ・逐語だけ・逐語が前後に空白の付いた印。
   - 日付の 4 形: 未記入・2026/09/27・2026-9-27・欄が無い。良い 3 形: 全部を埋めた形・引用符付きの年-月-日・印を含むだけの逐語。

   骨格のまま（4 欄とも 未記入）は便 155 のとおり「approvals[0].ruling に台帳 id が無い」で断る（裁定 id の印は台帳 id の形が落とす）。
3. **判断の記録の承認欄（`crates/folio/src/adr.rs` の check_approval・N-4）との食い違い。** 骨格の ADR-1 を accepted にして承認欄を足し、素の check を撃った。承認者・日付・裁定 id・対話面の印は、それぞれの形の決まり（値域・年-月-日・台帳 id）が落とす。**逐語だけは空でないことしか見ないので、逐語だけ 未記入 の承認欄は違反 0 で通る**（前後に空白の付いた印も同じ）。空の字（引用符 2 つだけ）は「verbatim が空」で落ちる。
4. **base（a6a868d）の歯。** workspace の nextest 974 / 974・clippy 0 警告・床 4 本 rc 0・`folio build --write` の出力 34 file・2146076 byte。`--test freeze_root` は 11 本。`f158_` と行 id `fe` は 0 件。

### (b) 直す先

1. **`crates/folio/src/adr.rs`。**
   1. 雛形の印の定数 UNFILLED（値は 未記入）と判定 unfilled を足す。判定は前後の空白を落として印そのものか。印を含むだけの字は印でない。
   2. 年-月-日の判定 is_date を crate の中へ開く（式は変えない）。
   3. check_approval で、逐語が印なら「<id>.approval.verbatim が 未記入（init の雛形の印・空と同じ）」の N-4 を積む。ほかの欄の判定は変えない。
2. **`crates/folio/src/anchor.rs` の check_approvals（凍結の前と後の同じ関数）。** 裁定 id の台帳 id の形の確かめの後に、次の 2 つを足す。どちらも違反を 1 件積んで次の項へ進む。
   1. 承認者か逐語が印なら「approvals[n] の who / verbatim が 未記入（init の雛形の印・空と同じ）」。印の欄だけを並べる。
   2. 日付が年-月-日でないなら「approvals[n].date「<値>」が年-月-日でない」。欄が無いときの値は None と出る。
3. **原理は 1 つ。** 非空を課す欄のうち、形の決まり（値域・年-月-日・台帳 id）を持たない欄だけが印を空と見る。承認一覧（非空を課すのは承認者・裁定 id・逐語）では承認者と逐語、判断の記録の承認欄では逐語がそれに当たる。形の決まりを持つ欄の印は、その形が既に落とす（2 重に数えない）。
   - 確かめの順は「空（今のまま）→ 台帳 id の形（今のまま）→ 印 → 日付」とする。骨格のままの答えと既存の歯の期待の字は変わらない。
4. **変えないもの。** freeze.rs・素の check の対象・3 つの凍結の旗の前提・列の根の表・床の定数（FLOOR と、その写しの生成区間）・init の雛形・命令の旗・folio2 自身の床と面。

### (c) 歯（関数名 f158_・`crates/folio/tests/freeze_root.rs`・binary 経由）

1. **f158_skeleton_freeze_start_reads_the_mark_and_the_date。**
   - 口 Work::init の骨格で、承認欄の裁定 id だけを p1-1 にして commit する。--freeze-start は rc 1 で違反ちょうど 2 件になる。1 件は承認欄の行「approvals[0] の who / verbatim が 未記入（…）」、もう 1 件は表に無い名。anchors/ は作らない。
   - 日付だけ 未記入 にした形では、承認欄の行が「approvals[0].date「未記入」が年-月-日でない」になる。
   - **base では違反が 1 件＝RED。**
2. **f158_the_mark_and_the_date_are_the_same_rows_before_and_after_the_freeze。** 床の土台の写しで、同じ書き換えを 2 か所に当てる。凍結の前は、列の無い置き場の憲法 meta.approval を変えて --freeze-start。凍結の後は、凍結済みの anchor の承認一覧を変えて素の check。
   - 行の頭を除いた本文が、前と後でちょうど同じになる（同じ関数・P-15.2）。
   - 承認者 → 未記入 と、全角と半角の空白を付けた印（「　未記入 」）・日付 → 2026/09/12・2026-9-12・2026-09-12 20:20 は 1 行ずつ。逐語 → 未記入の欄は無いので承認する は 0 行。
   - どちらも rc 1。凍結の前は anchors/ を作らない。
   - **base では前も後も 0 行＝RED。**
3. **f158_adr_approval_verbatim_mark_is_empty。** 床の土台の写しで ADR-1 の逐語を書き換え、素の check を撃つ。
   - 未記入 と、前後に空白の付いた印では、rc 1 で違反はちょうど [N-4] の逐語の 1 行になる。
   - 未記入の欄は無い（印を含むだけ）では、rc 0 で違反 0 になる。
   - 承認者 → 未記入 では、違反は値域外の 1 行だけ（印の行を 2 重に積まない）。
   - **base では rc 0＝RED。**

歯の file には口 skeleton_approval と anchor_approval を足し、頭の注に 10 を足す。fixture は足さない。

### (d) 採らなかった形

1. **印を最初の「空」の確かめに入れる（裁定 id も含む）。** 骨格のままの答えが「who / ruling / verbatim が無い」に変わり、便 155 の歯 2 本の期待の字が落ちる（(e) の M17）。裁定 id の印は台帳 id の形で既に落ちるので、足す利益が無い。
2. **判断の記録の床の値域（承認者 3 つ・対話面 R-8）を承認一覧にも課す。** folio2 の v1.0 は承認者 持ち主（shuu5）・対話面が散文で、folio2 自身の床が落ちる（便 155 の (d) の 2）。
3. **印を床の定数（FLOOR）に置いて生成区間へ写す。** 設計文書の正本（adr/schema.yaml の生成区間）を書き換える便になり、天井の印が古い今は書かない（(i) の 2）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は 0 本。** 本便を当てた写しで workspace の nextest 977 / 977・clippy 0 警告・床 4 本 rc 0・`--test freeze_root` 14 / 14。rustfmt --check の差の塊の数は 3 本とも base と同じ（2・1・16）。
2. **RED。** 歯だけを base に当てると f158_ の 3 本が落ち、ほかの 11 本は緑。
3. **突然変異。** 写しの実装を 1 通りずつ変え、`--test freeze_root` を撃った（番号は独立の検証の vmut.py と同じ）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 承認一覧で印を見ない / M2 日付を見ない / M7 印と日付を凍結の後だけ見る | 骨格・前後 |
| M3 印を含む字も印と見る / M4 前後の空白を落とさない | 前後・判断の記録 |
| M5 ASCII の空白だけ落とす / M6 凍結の前だけ見る | 前後 |
| M8 判断の記録の逐語の印を見ない / M13 判断の記録で承認者の印も数える（2 重） | 判断の記録 |
| M9 承認一覧だけ月日 1 桁も通す / M10 承認一覧だけ時刻付きを通す / M11 共有の is_date が時刻付きを通す / M12 承認一覧だけ空白を落とさずに比べる | 前後 |
| M14 承認一覧の承認者に判断の記録の値域を課す（(d) の 2） | f121_ の 4 本・f134_・f155_ の 1 本・前後・判断の記録 |
| M15 日付を印より先に見る / M16 印の違反の後も日付へ進む | 骨格 |
| M17 裁定 id の印も最初の空の確かめで空と見る（(d) の 1） | 骨格・前後・f155_ の 2 本 |
| M18 日付の違反の後も次へ進む | 生き残る |

   表の 骨格・前後・判断の記録 は (c) の 1・2・3。生き残るのは M18 だけで、欄 adr の在る項でだけ N-4 の行が 1 本増えうるほぼ等価な変異（rc と凍結の可否は変わらない）。

### (f) 大きさ・verify と done の対応

1. **write-set。** 3 本とも印なし（書き換えるだけ）: `crates/folio/src/anchor.rs`・`crates/folio/src/adr.rs`・`crates/folio/tests/freeze_root.rs`。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/anchor.rs` | 1047 | 453 | 1071（+24） | 429 |
| `crates/folio/src/adr.rs` | 1034 | 466 | 1049（+15） | 451 |

   src の外は `tests/freeze_root.rs` 643 → 810。
3. **size は S。** src は 2 本で +39。
4. **verify は 3 行**で、done の 3 の塊と 1 対 1: `--test freeze_root f158_`（(c) の 1〜3）・`--test freeze_root`（f155_ 3 本を含む全部・参考値 14 本）・clippy。base では 1 が 0 件で終了コード 4、2 は 11 本で緑、3 は 0 警告。`--bin` の行は無い。

### (g) 門・受付・並行の便

門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0。便 157 は着地済み。

### (h) 今の置き場の床が変わらないこと（撤退条件 (2) の実測）

1. **folio2 自身。** anchors/ の 5 版の承認一覧に印は無く、日付は年-月-日。床 4 本・build（34 file）・凍結の 4 つの旗の出力（標準出力・標準エラー・rc・書いた file）が base と本便で 1 byte も違わない（a6a868d で撃ち直した）。
2. **tsuzuri。** design-intent と contracts を写しへ cp して撃った（tsuzuri の repo では何も書かず git も撃たない）。v1.0 の承認一覧は 2026-09-25・持ち主・これでよい。素の check と 4 つの旗の出力が base と本便で一致した（素の check の 1 は両方とも ceiling.yaml の生成区間の 2 行だけ）。
3. **凍結済みの根。** base が測りだけの binary で凍結した 11 形の根を本便の床で撃つと、形の違反の 8 形は凍結の前の断りと同じ本文の 1 行で落ち、良い 3 形は 0 のまま。

### (i) 要件との関係・運ばないもの・撤退条件

1. **要件との関係（正本は書き換えない）。** FR24 の規範文と確かめ方は変えない。本便は「ほかの検査」の中身を締めるだけで意味は同じ。骨格のままの素の check は rc 2・違反 0 のまま（FR22）、FR23 の表も変えない。実装の直しに収まるので行 D-17 の持ち主の承認は要らない。
2. **運ばないもの（借り・行 D-11）。** 印の定数 UNFILLED と、承認一覧に日付の形を課す判定は、行 D-11 の範囲（違反・拒否を出す判定の閉じた一覧と式）に入り、P-5.6 の写しを要する。本便は写しを運ばない。理由は 3 つある。
   - 設計文書の正本（adr/schema.yaml の生成区間）を書く便は、今の天井の印の門が止める。
   - 生成区間の写しが変わると、全ての置き場の schema --check が撃ち直すまで落ちる（ADR-16 撤退条件 (2)）。
   - 生成区間の規則を変える便は、行 D-17 の持ち主の承認を要する。

   写しの便（床の定数の注 date_format_note・non_empty_note と anchor の注・印の値・FR22 の注の古い 1 文）は台帳 f2-648.239 に控え、次に設計文書を書ける節目で運ぶ。設計ノートの承認欄（note.rs）が逐語の印を通すのは台帳 f2-648.240 に控えた（write-set の外）。
   - 残る穴: 承認一覧の対話面は形も空も見ない（未記入・null が通る）。承認者の値域・印でない置き字・台帳に在るか・暦・持ち主の承認そのもの（P-12.1・P-12.3）も機械では締まらない。.233 の前提の字は「承認者と逐語が空でも印でもなく、裁定 id が台帳 id の形・日付が年-月-日の第 1.0 版」までに弱める。
3. **撤退条件。**
   - (1) 既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。
   - (2) 着地の後の main で、次のどれかが着地の直前と 1 byte でも違ったら止めて席へ返す: folio2 自身の床 4 本の結果・`folio build` の出力・tsuzuri の写しの素の check。
   - (3) 受付の時点の main で、anchor.rs の check_approvals か adr.rs の check_approval、tests/freeze_root.rs の口 Work が base と違えば、下の手順で数え直してから運ぶ。

数え直しの手順（行 D-13）: 記録は `.local/share/folio2/handoff-2026-09-27/d158-draft.md`、script は同じ dir の d158-scripts（repo の外）。観測 = repro・adr・build-probe・trap、模擬 = setup と apply-158-teeth.py・apply-158.py（改訂 b は c158b.patch・r158b-teeth.patch）、検査 = suite・verify・red（改訂 b は red-158b）・mut（改訂 b は mut-158b.py）・lines・fmt、置き場 = floor・tsuzuri、門 = gate（各 -158）。

## 2. 範囲

- 入れる: §1 (b) の 1・2、(c) の歯 3 本と口 2 つ。
- 入れない: freeze.rs・3 つの凍結の旗の前提・列の根の表・床の定数とその写し・init の雛形・命令の旗・設計文書の正本・新しい fixture と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| mark | 雛形の印 | adr.rs の UNFILLED・unfilled・is_date の公開と check_approval の逐語 |
| rows | 承認一覧の形 | anchor.rs の check_approvals の印と日付の 2 つの確かめ |
| teeth | 歯 | tests/freeze_root.rs の f158_ 3 本・skeleton_approval・anchor_approval |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は便 155（base に入る）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 .235 を閉じ、.233 の案 (c) の討論へ根の形が締まったことと (i) の 2 の残る穴を渡す。借りは台帳 .239・.240 に控え済み。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fe"
title = "群 A（台帳 f2-648.235）: 床の承認一覧の確かめ（crates/folio/src/anchor.rs の check_approvals・凍結の前と後の同じ関数）が init の雛形の印 未記入 を空と見ず日付の形も見ないので、裁定 id だけ台帳 id の形にした承認欄が凍結の前も後も通る。判断の記録の承認欄（adr.rs の check_approval）も逐語の 未記入 を通す。adr.rs に印の定数 UNFILLED と判定 unfilled を足して is_date を crate に開き、check_approval で逐語の印を N-4 で落とし、check_approvals で台帳 id の形の後に承認者と逐語の印・年-月-日でない日付を落とす（形の決まりを持たない欄だけが印を空と見る・骨格のままの答えは変えない）。freeze.rs・凍結の旗の前提・列の根の表・床の定数・init の雛形・設計文書の正本は変えず、folio2 と tsuzuri の写しの床も変わらない。歯は tests/freeze_root.rs の f158_ 3 本。門の対象外。base = main a6a868d・受付の時点の main で数え直す"
req = ["FR24"]
section = "1"
write-set = ["crates/folio/src/anchor.rs", "crates/folio/src/adr.rs", "crates/folio/tests/freeze_root.rs"]
verify = ["cargo nextest run -p folio --test freeze_root f158_", "cargo nextest run -p folio --test freeze_root", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/freeze_root.rs の f158_ の 3 本（init した骨格で承認欄の裁定 id だけを p1-1 にすると --freeze-start が rc 1 で違反ちょうど 2 件〔constitution-v1.0.yaml（凍結で書く承認一覧・憲法 meta.approval の写し）: approvals[0] の who / verbatim が 未記入（init の雛形の印・空と同じ） と 表に無い〕で anchors/ を作らず、日付だけ 未記入 なら承認欄の行が approvals[0].date「未記入」が年-月-日でない / 床の土台の写しで承認者を 未記入 か全角と半角の空白を付けた印・日付を 2026/09/12 か 2026-9-12 か 2026-09-12 20:20・逐語を 未記入の欄は無いので承認する にすると、列の無い置き場の --freeze-start と凍結済みの anchor の承認一覧を同じに変えた素の check が同じ本文の承認欄の行〔印と日付の 5 形は 1 行ずつ・逐語は 0 行〕を出してどちらも rc 1 / 床の土台の写しの ADR-1 の逐語を 未記入 か前後に空白の付いた印にすると素の check が rc 1 で違反はちょうど [N-4] ADR-1.approval.verbatim が 未記入（init の雛形の印・空と同じ） の 1 行、未記入の欄は無い なら rc 0 で違反 0、承認者を 未記入 にすると違反は値域外の 1 行だけ）が緑、tests/freeze_root.rs の歯の全部（f155_ 3 本を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
