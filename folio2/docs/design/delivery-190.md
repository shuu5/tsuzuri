# 設計: 便 190 — 床の版管理の照合は、取り込んでいない枝の履歴を数えない（判断の記録 ADR-34）

- 要件: FR5（構造の床は結果を 3 値で返し、実行できなかった検査を合格と表示しない）。本流の要件書に在る id で、字は変えない。床の偽の違反（本物でない「不合格」）を消し、本物の違反は今までどおり落とす。
- 条: P-3.1（決まった答えの出る検査は床に置く）・P-4.1 と P-4.2（読めない版管理は「まだ分からない」のまま）・P-10.1（期待の字は歯の側の手書き・凍結 anchor を測り直す）・P-10.3（anchor 0 本は「まだ分からない」）。
- 出所: 判断の記録 ADR-34（proposed・持ち主の承認の前・枝 docs/trust）。台帳 f2-648.258・f2-648.177。**本便は ADR-34 の発効の後に受け付ける。**
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gk` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 7 本（src 2・設計文書の正本の生成区間 1・凍結 anchor 1・歯の file 3〔本文不変 1〕）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: `folio ceiling --gate --dir design-intent --write-set …`（本流 44c14ac の binary・write-set 7 本）は **0（通す・印の周 2026-09-27-round51〔判定 合格〕に書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）**。
- 前提: **base = 本流 44c14ac（便 185 と 192〜194 の着地の後）+ ADR-34 の枝 docs/trust 178e9c8**（本流を merge で積んだ枝）。この契約の数はすべてその写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d190`（commit **83f049d**。080d9f7〔実装と歯 9 本〕→ e8165af〔席の裁定 (a) の歯 2 本〕→ c5d8a01 と d01d8dd〔改訂 a = 独立の検証 trust-verify.md の B1 と非 blocking 1・--full-history と refs/stash の除外・歯 4 本と単体の兄弟の行・注 2 つ〕→ 45a3060〔docs/trust 178e9c8 = 本流 44c14ac を merge で積む・force なし〕→ 83f049d〔limits_note の括弧を 1 文にし、便 194 の外の置き場へ写す字で判断の記録の番号の項だけが落ちる形にする〕）。`git diff 178e9c8 83f049d` が便の全体の差分（6 file・+389 −19・47,364 byte）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 83f049d -- <write-set の file>`）。ADR-34 の字は見本の後に直した（write-set の外）。write-set の外は変えない。
- 並行の便との重なり: (f) の 5。

## 1. 設計

### (a) いま起きていること（base の実測・参考値）

1. **床の中で版管理を読むのは `crates/folio/src/gitcheck.rs` だけ。** 命令は全部で 7 種。範囲は次のとおり。

| 関数 | 命令 | 読む範囲 |
| --- | --- | --- |
| `toplevel`（器の導出 file の置き場を解く） | `rev-parse --show-toplevel` | 作業ツリー |
| `check_git` | `rev-parse --show-toplevel` | 作業ツリー |
| `check_git` | `rev-parse --verify -q HEAD` | 先頭（HEAD）が在るか |
| `check_git` | `rev-parse --is-shallow-repository` | 浅い写しか |
| `check_git` | `check-ignore -q --no-index <path>` | 作業ツリーの除外の規則 |
| `check_git` | `ls-tree -r --name-only HEAD -- <anchors/>` | 先頭の木 |
| `check_git` | `log --all --format=%H --name-status -- <anchors/>` | **全ての参照の履歴** |
| `check_git` | `show <commit>:<path>` | 上の log が出した commit |

   全ての参照を読むのは `log --all` の 1 か所（と、その commit の `show`）。この履歴から 3 つを組む: 履歴に在った anchor の名（`ever`・消えたら違反「版管理の履歴に在ったが作業ツリーに無い」）・同じ形式の anchor の中身（`hist`・違えば違反「版管理の履歴の同じ形式の anchor と中身が違う」）・列の始め直しの判定（`Tracked::seen`・凍結の命令が読む）。
2. **偽の違反（最小の手書きの写し・`tests/fixtures/anchor/` の 2 組・本流の binary）。** 取り込んでいない枝 w を切って commit し、元の枝へ戻って床を撃つ。
   - 偽 1: w が anchor（constitution-v1.1.yaml）を足す → 違反「履歴に在ったが作業ツリーに無い」1。
   - 偽 2: w が v1.0 の anchor の題を 1 字変える → 違反「中身が違う」1（元の枝の anchor は元のまま）。
   - 偽 3: anchor の無い元の枝と、最初の anchor を足した w → 違反 2・終了コード 1（本来は anchor 0 本の「まだ分からない」2）。
3. **本物の違反（同じ binary で落ちる・本便の後も同じ数で落とす）。** 本物 1: 今の枝で anchor の削除を commit → 違反 1。本物 2: 根の無い枝（orphan）へ切って anchors/ を外す → 違反 2（元の枝の履歴）。本物 3: 今の枝で anchor の書き換えを commit → 違反「中身が違う」1。
4. **実の例。** 台帳 .177（2026-09-24・取り込んでいない枝 docs/adr17 の constitution-v1.3.yaml で本流 628c547 の床が違反 1）と .258（2026-09-27・止めた便 170 の枝と控えの枝 impl/f2-648.254-run1 の adr-seals.yaml で本流の根の床が違反 1・本物の design-intent を撃つ歯 5 本も手元で落ちた）。folio2 の写しで、封の一覧が入る前の本流 b0f09d4 に控えの枝の参照を持たせると、本流の binary は `[anchor] anchors/adr-seals.yaml は版管理の履歴に在ったが作業ツリーに無い` を出す。CI は浅い写しで落ちない。
5. **判断の記録。** ADR-2 決定 (3) の字「全ての参照（ref）の履歴を見て…」の字どおりの帰結で、ADR-34（proposed）がこの範囲を狭める（ADR-2 の本文は変えない・ADR-30 決定 (1)）。今の規範の字は `design-intent/adr/schema.yaml` の生成区間の注 anchor_note（「全ての参照（--all）の履歴を見る」）と limits_note（「全ての参照の照合」）が持つ（床の定数 `crates/folio/src/floor_adr.rs` の写し）。
6. **base の歯（参考値）。** workspace の nextest 1070 / 1070（docs/trust 178e9c8 の写し）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 38 file。`git grep -n f190_ -- crates` は 0 件・行 id `gk` は 0 件。
7. **取り込みの簡約の穴（base から在る・独立の検証の d6 と b5）。** 履歴の一覧の命令 `git log -- <anchors/>` は既定で履歴を簡約し、取り込みの commit の anchors/ が片方の親と同じなら、その親だけを辿る。anchor の無い頃の commit から切った枝で中身の違う同じ版を置き直し、衝突を枝の側で解いて本流へ取り込むと、本流の側の親（元の anchor を足した commit）を辿らず、base の本流の床は「中身が違う」を出さない（repro.sh の t5b・base で 0）。`--full-history` で簡約を切ると出る。

### (b) 直す先

1. **`crates/folio/src/gitcheck.rs` の `check_git`。** 履歴の一覧の命令を `log --exclude=refs/stash --all --full-history --format=%H --name-status -- <anchors/>` にし（作業の一時置き場を数えない・取り込みの commit で本流の側の親を落とさない＝(a) 7）、命令を 1 本足す: `rev-list --parents --exclude=refs/stash --all --not HEAD`（先頭から辿れない commit と、その親の一覧）。`ls-tree`・`log` と同じく、起動できない・待ち上限を超えたは「版管理（git）が無いか読めない」、失敗（rc が 0 でない）は「版管理を読めない（ls-tree / log / rev-list が失敗）」の「まだ分からない」で終える。`log` の行を読むとき、commit が次の関数 `aside` の集合に在れば、その commit の状態の行を `ever` にも `hist` にも入れない（commit を `None` にする）。
2. **関数 `aside`（新・同じ file）。** 上の命令の出力から、先頭から辿れず祖先に先頭から辿れる commit を持つ commit（取り込んでいない枝）の集合を返す。組み方: 出力の commit の集合を「外」とし、親が「外」に無い commit（親が先頭から辿れる）から始めて、「外」の中の子へ（兄弟の枝も全部）辿って拾う。どの祖先も先頭から辿れない commit（根の無い枝）は拾わない＝今までどおり数える。
3. **`crates/folio/src/floor_adr.rs` の注 2 つ。** anchor_note の版管理の項の「全ての参照（--all）の履歴を見る。」を「全ての参照（--all・作業の一時置き場 refs/stash を除く）を読み、照合するのは先頭（HEAD）の祖先の履歴と、HEAD と共通の祖先を持たない根の無い枝の履歴だけ（HEAD と共通の祖先を持つ取り込んでいない枝の履歴は数えない・取り込みの commit では両方の親の履歴を辿る〔--full-history〕・判断の記録 ADR-34）。」に、limits_note の「全ての参照の照合・同じ形式の anchor の中身の照合・列の根の digest の固定まで。」を「根の無い枝の履歴の照合・同じ形式の anchor の中身の照合・列の根の digest の固定まで（HEAD と共通の祖先を持つ取り込んでいない枝の履歴は数えない＝anchor の前の commit から切り直した枝の上で anchor を置き直す細工はその枝の床では数えず、本流へ取り込んだ時点で本流の床が落とし、本流の床を通さない参照の付け替えは床の外・判断の記録 ADR-34）。」にする（括弧の中を 1 文にする＝便 194 の外の置き場へ写す字〔`floor.rs` の `text_for`〕が判断の記録の番号の項だけを落とし、ほかの句は tsuzuri の写しにも残る）。注なので床（folio check）は数えない。
4. **`design-intent/adr/schema.yaml`。** `folio schema --dir design-intent --write` で生成区間を書き直す（上の 2 行だけが変わる・28,247 → 28,979 byte）。手では書かない。
5. **凍結 anchor。** `tests/fixtures/schema/adr-region.txt` を新しい生成区間（begin と end の行を除く）に測り直し、`crates/folio/tests/schema.rs` の定数 3 つ（REGION_LINES 151 のまま・REGION_BYTES 28247 → 28979・REGION_SHA256 → daf05362…2613）を直す。値は folio と独立の数え（python の hashlib・起草の記録の anchors-190.py）。`tests/schema.rs` は便 194 の着地の後、器の式（幅 120 正規化・空行 1・歯 f89〔`tests/schema_docs.rs` の f89_schema_teeth_are_split_and_under_the_cap〕）で **700 行＝上限 700 ちょうど（余地 0・wc -l では 699）**。本便は定数 2 行の値を書き換えるだけで、どちらの行も 120 字の内＝器の式の行数は 700 のまま（見本 83f049d で python と awk の 2 実装が 700・歯 f89 が緑）。**作業者は `tests/schema.rs` に行を足さない**（足すと f89 が落ちる・共通の検証の workspace の nextest が撃つ）。
6. **変えないもの。** 違反と「まだ分からない」の字（「読めない」の括弧の中の命令の名だけ rev-list を足す）・先頭の木との照合・除外・未追跡・版管理の根・浅い写し・環境変数の遮断・列の根の digest・凍結の命令の振る舞い（`seen` は同じ範囲の `ever` から組む）・`toplevel`・憲法と要件書と判断の記録の字・生成区間の注 2 つの外・床の fixture（`tests/floor_cases.yaml`）の字・`folio build` の出力（38 file・base と byte で同じ）・folio2 自身の床 4 本の結果。

### (c) 歯（f190_・base で 0 件）

binary 経由の歯は `crates/folio/tests/gitcheck.rs`（既存の file の末尾・git の助けは標準出力を返す形にし、床を撃って写しを残す `run_check` と、v1.1 を足す `add_v11` を分ける）。今の歯と同じく `tests/fixtures/anchor/` の組を一時 dir の `design-intent/` に写し、根で git init と 1 commit にする。取り込んでいない枝 w は 2 つの commit（根の file だけの commit の後に anchor を変える commit）で作り、元の枝へ戻ってから床を撃つ。期待の数と字は歯の側の手書き。

1. **f190_side_branch_added_anchor_is_not_lost（偽 1）。** 組 root-digest-drift。w が constitution-v1.1.yaml を足す。元の枝の床の違反は 1 行だけ（組の土台の違反「列の根の表に無い」）で、「履歴に在ったが作業ツリーに無い」は 0。
2. **f190_side_branch_rewrite_is_not_a_swap_on_head（偽 2）。** w が v1.0 の題を 1 字変える。元の枝の違反は 1 行だけで、「中身が違う」は 0。
3. **f190_side_branch_first_anchors_leave_head_unknown（偽 3）。** 組 no-anchor。w が root-digest-drift の anchors/ を足す。元の枝の終了コードは 2 で違反 0。
4. **f190_deletion_committed_on_head_still_fails（本物 1）。** 今の枝で v1.0 の削除を commit。終了コード 1・「履歴に在ったが作業ツリーに無い」がちょうど 1（v1.0）。
5. **f190_side_branch_merged_then_deleted_fails（本物 4）。** w（v1.1 を足す）を元の枝へ取り込み（merge --no-ff）、その後に v1.1 の削除を commit。「履歴に在ったが作業ツリーに無い」がちょうど 1（v1.1）＝取り込んだ枝の履歴は先頭の祖先として数える。
6. **f190_rootless_branch_still_sees_the_other_history（本物 2）。** 根の無い枝 clean へ切って anchors/ を版管理と作業ツリーから外して commit。終了コード 1・「履歴に在ったが作業ツリーに無い」がちょうど 2（元の枝の v1.0 と索引）。
7. **f190_rewrite_committed_on_head_still_fails（本物 3）。** 今の枝で v1.0 の題を 1 字変えて commit。終了コード 1・「中身が違う」がちょうど 1。
8. **f190_side_refreeze_fails_once_merged_into_head（敵対の形・席の裁定 (a)）。** 共通の祖先を持つ枝 w で v1.0 の削除を commit し、その後に中身の違う v1.0 を置き直して commit（凍結し直しの形）。取り込む前の元の枝の違反は土台の 1 行だけ。w を元の枝へ取り込む（merge --no-ff）と、終了コード 1・「中身が違う」がちょうど 1・「履歴に在ったが」は 0。
9. **f190_foreign_rootless_ref_is_not_a_violation（席の裁定 (a)）。** 版管理の外の用途の参照（台帳の置き場の形の `refs/dolt/data`）に、anchor の置き場に触れない根の無い 2 commit（空の木）を置く。違反は土台の 1 行だけ。
10. **f190_cut_before_anchors_refreeze_fails_once_merged（検証の d6 の形・改訂 a）。** 組 no-anchor の commit P の上に root-digest-drift の anchors/ を足す commit A（本流）。P から切った枝 cut で同じ anchors/ を足して v1.0 の題を 1 字変える commit B（凍結し直しの形）。cut の上の床の「中身が違う」は 0（範囲の外・ADR-34 決定 (4)）。本流で `merge -X theirs cut`（衝突を枝の側で解く）の後、終了コード 1・「中身が違う」がちょうど 1（A の中身と違う）。base では取り込みの後の本流が 0（履歴の簡約で A を辿らない）。
11. **f190_remote_tracking_side_branch_is_not_lost（検証の V8・台帳 .258 の形・改訂 a）。** w（v1.1 を足す）を `refs/remotes/origin/w` だけが指す形にする（`refs/heads/w` は消す）。違反は土台の 1 行だけで「履歴に在ったが」は 0。
12. **f190_rev_list_failure_is_unknown（検証の V7・条 P-4.1・改訂 a）。** 引数に rev-list を持つときだけ終了コード 128 で終え、ほかは本物の git へ渡す shell の git を、folio の子の処理の PATH の先頭に置く。標準エラーに「# まだ分からない: 」で始まり「版管理を読めない（ls-tree / log / rev-list が失敗）」を含む行が在り、「履歴に在ったが」と「中身が違う」は 0。
13. **f190_stash_is_not_counted（改訂 a）。** 未追跡の v1.1 を `git stash -u` で退ける（未追跡の file の commit は根の無い commit）。違反は土台の 1 行だけで「履歴に在ったが」は 0。
14. **f190_schema_notes_name_the_counted_history。** 実の `design-intent/adr/schema.yaml` に、(b) 3 の新しい字の 4 句（anchor_note の範囲の句・「作業の一時置き場 refs/stash を除く」・limits_note の最初の句・「本流へ取り込んだ時点で本流の床が落とす」）がちょうど 1 回ずつ在り、古い字「全ての参照（--all）の履歴を見る」「全ての参照の照合」が無い。
15. **単体の歯 f190_aside_takes_only_branches_joined_to_head（`crates/folio/src/gitcheck.rs` の tests の区間）。** 手書きの親の表 8 行（取り込んでいない枝 s1〜s4〔s1 は子 s2 と s4 の 2 本に分かれる〕・根の無い枝 o1 と o2・根の無い枝を先の親に先頭の祖先を後の親に持つ取り込み m1 とその子 x1）で、`aside` がちょうど m1・s1・s2・s3・s4・x1 を返す。空の出力は空。
16. **RED の実測。** 歯の file だけ（r190-teeth.patch）を base に当てると、binary の歯 14 本のうち 1・2・3・8・10・11・12・14 の 8 本が落ちる（起草の記録の red-190-bin.log に落ちた歯の本文）。残りの 4・5・6・7・9・13 の 6 本は base でも緑（13 は base の `log` が既定の簡約で stash の未追跡の commit を辿らないので緑）（本物の違反を今までどおり落とすことと、外の用途の参照と作業の一時置き場で偽の違反が出ないことの固定）。base の `--bin folio f190_` は 0 本（関数 `aside` が無い・nextest の rc 4）。床の fixture の根の無い枝の場合（git-orphan-branch-refreeze）は字を変えずに base と見本の両方で落ちる（verify 5）。

### (d) 採らなかった形（ADR-34 の案 b〜e）

1. **先頭から辿れる履歴だけを数える（`log --all` を `log HEAD` に）。** 根の無い枝の手口の防ぎが消える。見本をこの形に変えると、歯 6 と床の fixture `tests/floor_cases.yaml` の git-orphan-branch-refreeze（`floor_cases_all_pass_with_folio`）が落ちる（変異 M2）。
2. **先頭を祖先に持つ参照も数える（台帳 .258 の候補）。** 本流から切って本流がまだ進んでいない作業枝は先頭を祖先に持つので、偽 1〜3 がそのまま残る。
3. **既定の枝の参照（origin/HEAD）も数える。** 本流が先へ進んだ作業枝で、本流が新しく足した anchor を「無い」と数える偽の違反が出る。remote の名と設定は置き場ごとに違う。
4. **`--full-history` を足さず、字だけ直す（検証の提案 (b)）。** (a) 7 の穴が残り、決定 (4) の枝を普通の merge で本流へ入れても本流の床が黙るので、ADR-34 の撤退条件 (2) を本流の床で数えられない。`--full-history` だけを足して refs/stash を外さないと、`git stash -u` の未追跡の commit（根の無い commit）で偽の違反が出る（歯 13・変異 M13）。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本 83f049d の写しで workspace の nextest **1085 / 1085**（base 1070 + f190_ の 15 本・`--test-threads 2`）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0・schema --check 一致）・`folio build --write` 38 file（base と全 file が byte で同じ・要約 f0dd78844514bdfb）。字の期待を直した既存の歯は `tests/schema.rs` の凍結 anchor の定数 2 つだけ（(b) 5）。
2. **突然変異（見本 83f049d の写しの src だけを 1 通りずつ変え、f190_ の 15 本を撃つ・M2・M3・M11 は tests/floor_cases と tests/schema も撃つ）。** 16 通りとも落ちる（生き残り 0・mut-190.log・M14〜M16 は独立の検証の V5・V7・V8 で、改訂 a の前は生き残った）。

| 変異 | 落ちる歯（番号は (c)・ほか） |
| --- | --- |
| M1 `aside` を空にする（base と同じ） | 1・2・3・8・10・11・15 |
| M2 `log --all` を `log HEAD` にする（案 (d) 1） | 6・floor_cases |
| M3 先頭の外の commit を全部数えない | 6・15・floor_cases |
| M4 先の親だけを見る | 15 |
| M5 子へ辿らない | 1・2・3・8・11・15 |
| M6 取り込んでいない枝を `ever` からだけ外す | 2・8・10 |
| M7 取り込んでいない枝を `hist` からだけ外す | 1・3・11 |
| M8 rev-list から `--not HEAD` を落とす | 1・2・3・8・10・11 |
| M9 親の無い commit も取り込んでいない枝に数える | 6・15 |
| M10 子の表の向きを逆にする | 1・2・3・8・11・15 |
| M11 注を元の字に戻す（実装だけ変える） | tests/schema の 8 本（生成区間が導出と違う） |
| M12 `log` から `--full-history` を落とす | 10 |
| M13 `log` と rev-list から `--exclude=refs/stash` を落とす | 13 |
| M14 子の表を上書きする（兄弟の枝を 1 本しか辿らない・検証の V5） | 15 |
| M15 rev-list の失敗を見ない（検証の V7） | 12 |
| M16 rev-list の `--all` を `--branches` にする（検証の V8） | 11 |

3. **外の置き場（tsuzuri の写し・参考値）。** HEAD a618238（枝 154 本・先頭の外の commit 146・anchors/ に触れる commit 6 はどれも先頭の祖先）と、改訂 a の見本で撃ち直した HEAD 7d633b6（参照 160 本・先頭の外の commit 151・写しは clone --no-checkout の後に core.hooksPath が無く hook が sample だけなことを確かめてから checkout）で、base と見本の binary の床の出力が byte で同じ（合格 違反 0・まだ分からない 0・本流 44c14ac の上の base 178e9c8 と見本 83f049d の binary で HEAD 7d633b6 を撃ち直しても同じ）。取り込むと `folio schema --check` が判断の記録の欄の決まりの注 2 つで落ちる（床は注を数えないので落ちない）＝着地の後に `folio schema --write` を 1 回。
4. **実の例の写し。** folio2 の本流 b0f09d4 の写し（控えの枝の参照つき）で、base の違反「anchors/adr-seals.yaml は版管理の履歴に在ったが作業ツリーに無い」1 が見本で 0。folio2 の今の写し（参照 428 本）では base・見本とも床 4 本 rc 0・合格。
5. **命令の時間。** 増える `rev-list` は folio2 の写し（参照 400 本余り・commit 1,432）で 0.02 秒（独立の検証の写し〔参照 750 本〕で 0.03 秒）。
6. **独立の検証（trust-verify.md）の 39 場面。** 改訂 a の形（`--full-history` と `--exclude=refs/stash`）で、d6・d6b・b5 だけが本流で終了コード 1（中身が違う 1）に変わり、ほかの 36 場面と folio2・tsuzuri の写しの床の出力は変わらない（検証役の実測・起草役は repro.sh の t5a・t5b・t6 で d6 の形を撃ち直した）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file は無い。`crates/folio/tests/floor_cases.rs` は本文を変えない（verify の `--test` の scope）。差分 47,364 byte（`git diff 178e9c8 83f049d | wc -c`・6 file・+389 −19）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 2 本（python と awk の 2 実装で一致・cap-190.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/gitcheck.rs` | 403 | 1097 | 465（+62） | 1035 |
| `crates/folio/src/floor_adr.rs` | 511 | 989 | 514（+3） | 986 |

3. **size は S。** src の増分は +65 で S の見積 100 の内。余地の最小（floor_adr.rs の base 989）は S の 100 を超える。
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。見本 83f049d の写しで 6 行とも rc 0（1 行目 14 本・2 行目 1 本・3 行目 18 本・4 行目 21 本・5 行目 12 本・clippy 0 警告・起草の記録の verify-impl6.log）。
   1. `cargo nextest run -p folio --test gitcheck f190_` = (c) の 1〜14（14 本）。
   2. `cargo nextest run -p folio --bin folio f190_` = (c) の 15（1 本）。
   3. `cargo nextest run -p folio --test gitcheck` = 便 8 の版管理の照合の歯 4 本と (c) の 14 本（今の違反の字と数が同じ）。
   4. `cargo nextest run -p folio --test schema` = 生成区間と凍結 anchor の一致（(b) 4・5）。
   5. `cargo nextest run -p folio --test floor_cases` = 床の fixture（根の無い枝の場合 git-orphan-branch-refreeze を今の字のまま・環境変数・浅い写しの場合を含む）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（gitcheck・schema・floor_cases）は全部 write-set に在る。`--bin folio` の歯の在り処（gitcheck.rs）も write-set に在る。
6. **並行の便との重なり（2026-09-28 20:1x の時点・origin の impl/d187 14bca08・impl/d195 80aec6c・impl/d196 1a7c4c7・impl/d197 8a0abe9・impl/d198 15b235a と各契約の write-set を読み、見本 83f049d との `git merge-tree` を撃った＝5 本とも字の衝突 0）。** 着地の順は 185 → 小さな直し（192〜194・着地済み）→ 床の穴（187）→ 本便 → 写しの負債（195〜197）→ 編集時の止め（198〜）。

| 並行の便 | 重なりうる file | 扱い |
| --- | --- | --- |
| 185（着地済み 6467915） | 無し | base に含む |
| 192〜194（着地済み 44c14ac・小さな直し） | 無し（194 は `floor.rs` の外の置き場へ写す字と `tests/schema.rs` の外の歯を書いた＝本便は merge で積み、注の括弧を 1 文にした。`tests/schema.rs` は積んだ後 器の式で 700 行＝上限ちょうど・本便は定数 2 行の値だけ（(b) 5）） | base に含む |
| 床の穴 187（impl/d187 14bca08） | write-set の `tests/floor_cases.rs` だけ（両便とも本文を変えない verify の scope・字はぶつからない）。ほかは cursor.rs・face_note.rs・figure.rs・main.rs・site.rs・新 tests/floor_faces.rs と本文不変の tests 4 本 | 順は 187 が先・数え直し不要（受付の先撃ちで live な run との重なり 1 件が出たら順序どおり） |
| 写しの負債 195（impl/d195 80aec6c） | 無し（graph.rs・link.rs・mentions.rs・note.rs・prose.rs・refs.rs・tests/graph.rs・tests/modules.rs・tests/schema_docs.rs・graph.yaml・graph-region.txt） | 本便が先 |
| 写しの負債 196（impl/d196 1a7c4c7） | `tests/schema.rs`（196 は設計ノートの生成区間の定数 NOTE_REGION_ の 3 行、本便は判断の記録の REGION_ の 2 行＝別の行で、merge-tree の衝突 0・取り込んだ後も器の式で 700 行）と write-set の `tests/floor_cases.rs`（両便とも本文不変）。判断の記録の生成区間には触れない | 本便が先。196 は本便の着地の後に取り込んで数え直す |
| 写しの負債 197（impl/d197 8a0abe9） | 無し（intake.rs・rules.rs・sheet.rs・tests/schema_docs.rs・tests/sheet.rs・intake.yaml・rules.yaml と区間 2 つ） | 本便が先 |
| 編集時の止め 198（impl/d198 15b235a） | `gitcheck.rs`（198 は子の git に版管理を探す天井を渡す `ceil_at` と `NO_GIT` の可視性・本便は `check_git` と `aside`＝別の場所で merge-tree の衝突 0・取り込んだ後 477 行・余地 1023） | 本便が先。198 は本便の着地の後に取り込んで数え直す |

### (g) 門と受付

1. **門。** 冒頭のとおり 0（通す）。
2. **受付。** ADR-34 の発効（持ち主の承認・枝 docs/trust の取り込み）の後。受付の先撃ち（precheck）は枝 docs/trust の本契約で 契約に起因する断り 0（preflight ok・`f190_` は base で 0 件・起草の記録の precheck-190.log）。
3. **着地の後。** 席は tsuzuri へ「床の版管理の照合は取り込んでいない枝の履歴を数えない（ADR-34）・`folio schema --write` を 1 回（判断の記録の欄の決まりの注 2 つ）」を返す。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/trust-draft.md`、script と log は同じ dir の trust-scripts。

1. 模擬: 見本 83f049d（c190.patch = `git diff 178e9c8 83f049d`・歯だけ = r190-teeth.patch）。chain-190d.sh（本流 44c14ac の上の数）。run-190.sh（組み立て・nextest〔`--test-threads 2`〕・clippy）・floor4.sh（床 4 本と build）・chain-190.sh（base と 080d9f7）・chain-190b.sh（e8165af）・chain-190c.sh（改訂 a の d01d8dd の数・RED・変異 16 通り・余地）。
2. 最小の写し: repro.sh（偽 3 形・本物 3 形・範囲の外 1 形を base と見本の binary で撃つ）。
3. 凍結 anchor: anchors-190.py（生成区間を写し、定数を hashlib で測る・冪等）。
4. RED: red-190.sh。突然変異: mut-190.py（16 通り）。余地: cap-190.sh（lines.py と lines.awk）。
5. 外の置き場: tz-190b.sh（tsuzuri の .git だけを写し、その写しの中では git を撃たず、clone --no-checkout の後に core.hooksPath が無く hook が sample だけなことを確かめてから checkout し、枝を局所の枝にしてから remote を外す・前の版の tz-190.sh は checkout を先にしていた）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** ADR-34 の本文（枝 docs/trust・席が持ち主へ出す）・ADR-2 の本文・要件書と憲法の字・床の fixture の字（`tests/floor_cases.yaml` の why の「全 ref の履歴」は根の無い枝の場合の説明として今も当たる）・床の土台と fixture の中の判断の記録の欄の決まりの写し（注は床が数えない・床の土台 `tests/fixtures/floor_base/design-intent/adr/schema.yaml` の生成区間は base で既に 23,235 byte〔導出は 28,247〕で導出と揃えていない）・tsuzuri の写しの書き直し（tsuzuri の手番）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** (1) anchor の無い頃の commit から切り直した枝（本流と共通の祖先を持つ）と、根の無い枝に anchor の無い頃の commit を繋いだ枝では、本便の後は、その枝の上の床が早く止めない（anchor を外しただけなら 0 本の「まだ分からない」か記録が在るのに anchor が無い違反・置き直せば通りうる＝repro.sh の t4 と t5a・歯 10 の前半）。その枝を本流へ取り込んだ時点で本流の床が落とす（merge・squash・rebase のどれでも・歯 10 の後半）。本流の床を通さない道は参照の付け替えだけで床の外（ADR-34 決定 (4)）。浅い写しで全部の枝を持つ形（clone --depth 1 --no-single-branch）では、取り込んでいない枝の先頭が根の無い commit になり偽の違反が残る（浅い写しは変えない）。(2) 取り込み（merge）の commit の状態の行は `log` が既定で出さないので、取り込みで足された anchor は、取り込んだ枝の側の commit で数える（今と同じ）。
3. **撤退条件。** (1) 本便が要件書 FR5 か ADR-34 か憲法の字を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `gitcheck.rs` の `check_git`、`floor_adr.rs` の注 2 つ、`adr/schema.yaml` の生成区間のどれかが base と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に (b) 5 の定数の外の既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果が変わるか、`folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `gitcheck.rs` の `check_git` の履歴の一覧の命令の旗 2 つ（`--full-history`・`--exclude=refs/stash`）と命令 1 本と行の飛ばし・関数 `aside`・単体の歯 1 本、`floor_adr.rs` の注 2 つ、`adr/schema.yaml` の生成区間の書き直し、凍結 anchor `adr-region.txt` と `tests/schema.rs` の定数、`tests/gitcheck.rs` の f190_ の 14 本と助けの 3 つ（`git` が標準出力を返す・`run_check`・`add_v11`）。
- 入れない: 違反の字・先頭の木との照合・除外・未追跡・浅い写し・環境変数・列の根・凍結の命令・`toplevel`・憲法と要件書と判断の記録の字・床の fixture の字・外の置き場・台帳への記帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| side | 取り込んでいない枝の集合 | `gitcheck.rs` の `aside`・命令 `rev-list --parents --all --not HEAD` |
| skip | 行の飛ばし | `check_git` の `log --exclude=refs/stash --all --full-history` の行の読み（`ever`・`hist`・`seen` が同じ範囲） |
| note | 今の規範の字 | `floor_adr.rs` の注 2 つ → `adr/schema.yaml` の生成区間 |
| anchor | 凍結 anchor | `adr-region.txt`・`tests/schema.rs` の定数 |
| teeth | 歯 | `tests/gitcheck.rs` の f190_ の 14 本と `gitcheck.rs` の単体の 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 185 と 192〜194（本流 44c14ac）と ADR-34 の発効（枝 docs/trust）。
- 本便の着地の後に席が見ること: 台帳 f2-648.258 と f2-648.177 を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ `folio schema --write` の 1 回を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gk"
title = "床の版管理の照合は、取り込んでいない枝の履歴を数えない（判断の記録 ADR-34・台帳 f2-648.258 と f2-648.177）: crates/folio/src/gitcheck.rs の check_git の履歴の一覧の命令を log --exclude=refs/stash --all --full-history にし（作業の一時置き場を数えない・取り込みの commit で本流の側の親を落とさない）、命令 rev-list --parents --exclude=refs/stash --all --not HEAD を足し（起動できない・失敗は今と同じまだ分からない）、新しい関数 aside で先頭から辿れず祖先に先頭から辿れる commit を持つ commit（取り込んでいない枝）の集合を組み、log の行のうちその commit の状態の行を ever にも hist にも入れない（先頭の祖先と根の無い枝の履歴は今までどおり数える・違反の字は変えない）。crates/folio/src/floor_adr.rs の注 anchor_note と limits_note の全ての参照の句を ADR-34 の範囲の字に直し、folio schema --write で design-intent/adr/schema.yaml の生成区間を書き直し、凍結 anchor tests/fixtures/schema/adr-region.txt と crates/folio/tests/schema.rs の定数 REGION_BYTES と REGION_SHA256 を測り直す。歯は crates/folio/tests/gitcheck.rs の f190_ の 14 本（偽の違反 3 形と remote-tracking の参照だけが指す枝と作業の一時置き場で偽の違反が出ない・本物の違反 4 形と、共通の祖先を持つ枝で凍結し直して先頭へ取り込む形と、anchor の無い頃から切った枝で凍結し直して衝突を枝の側で解いて取り込む形が落ちる・版管理の外の用途の根の無い参照で偽の違反が出ない・rev-list の失敗はまだ分からない・注の字）と gitcheck.rs の単体の 1 本（aside の親の表・兄弟の枝）"
req = ["FR5"]
section = "1"
write-set = ["crates/folio/src/gitcheck.rs", "crates/folio/src/floor_adr.rs", "design-intent/adr/schema.yaml", "tests/fixtures/schema/adr-region.txt", "crates/folio/tests/gitcheck.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/floor_cases.rs"]
verify = ["cargo nextest run -p folio --test gitcheck f190_", "cargo nextest run -p folio --bin folio f190_", "cargo nextest run -p folio --test gitcheck", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "crates/folio/tests/gitcheck.rs の f190_ の 14 本（取り込んでいない枝が anchor を足す・書き換える・最初の anchor を足すの 3 形と、remote-tracking の参照だけが指す枝と、stash -u の未追跡の commit で先頭の床に版管理の違反が出ない・今の枝での削除と書き換えの commit・取り込んだ後の削除・根の無い枝で anchors/ を外すの 4 形は今までどおり落ちる・共通の祖先を持つ枝で凍結し直して先頭へ取り込むと落ちる・anchor の無い頃から切った枝で凍結し直し衝突を枝の側で解いて取り込むと本流が落ちる・refs/dolt/data の形の根の無い参照で違反が増えない・rev-list だけが失敗するとまだ分からない・判断の記録の欄の決まりの注が ADR-34 の範囲の字）が緑、binary の単体の f190_ の 1 本（aside の親の表・兄弟の枝）が緑、tests/gitcheck.rs の歯の全部（便 8 の 4 本を含む）が緑、tests/schema.rs の歯の全部（生成区間と凍結 anchor の一致）が緑、tests/floor_cases.rs の歯の全部（根の無い枝の場合 git-orphan-branch-refreeze を今の字のまま・環境変数・浅い写しの場合を含む床の fixture）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check が 0・folio derive --dir design-intent --out ../contracts --check が一致"
<!-- contracts:end -->
