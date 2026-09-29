# 設計: 便 192 — folio derive --from-root と、face / check / serve の説明の字と serve の断りの字（main.rs の小さな便）

- 要件: FR11（導出物を生成し差分 0 を数える＝`folio derive` の口）・FR7（組み立てて見せる＝`folio serve`）・FR5（検査結果を必ず返す＝`folio check`）・FR4（面を 1 つの生成器で出す＝`folio face`）。本流の要件書に在る id で、字は変えない。
- 条: P-6.3（器の導出 file を探す根と --out の根を 1 つの式にする）・P-4.1（根が解けなければ「まだ分からない」）・N-3.1 には当たらない（旗は出力先の解き方だけで、どの検査も無効にしない）・N-6.1（第 1.2 版の字＝同じ端末の中〔loopback〕でも tailnet の中でもない bind 先を拒む）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 台帳 f2-648.228（tsuzuri の気づき (b)・--out を repo の根からの相対でも受ける）と f2-648.185（天井の 31 周目 実態 F-4・命令の説明の字が古い・判断の記録 ADR-12 の帰結「断りの文言と命令の一覧の説明は次に配信の実装を触る便で実態へ揃える」）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gm` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 7 本（src 4・歯の file 3〔新 1〕）。縮む file・消す file・新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流の binary で `folio ceiling --gate --dir design-intent --write-set <write-set の 7 本>` は **0（通す・設計文書の正本を書き換えない便）**。
- 前提: **base = 本流 6467915**（便 185 の着地の後）。この契約の数はすべて 6467915 の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d192`（commit **a85925b**・親 6467915）。`git diff 6467915 a85925b` が便の全体の差分（7 file・+233 −19・18,434 byte）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout a85925b -- <write-set の file>`）。write-set の外は変えない。
- 並行の便との重なり: 起草の時点の見本の枝で、`main.rs` は便 187（impl/d187・床の穴・着地の順は本便より後）が触り、`note.rs` は便 195（impl/d195・写しの負債・本便より後）が触る。どちらも本便と違う塊（便 187 は `run` の床の段・便 195 は参照 id の読み）。編集時の止めのレーン（便 198〜200）も `main.rs` を触る見込みなので、本便は `main.rs` の説明の字 3 つと旗 1 つ（+10 −5）だけに絞った。本便は 187 と 190 を待たずに先に受け付ける（席の順・2026-09-28）。後から着地する便の側で数え直し、本便より先に着地した便が在れば受付の時点の本流で数え直す（§1 (g)）。

## 1. 設計

### (a) いま起きていること（base 6467915 の実測・参考値）

1. **--out は置き場（--dir）からの相対か絶対 path だけ。** `folio derive --dir design-intent --out <絶対 path> --check` は 0（絶対 path は今も受ける）。repo の根からの相対の `--out contracts` は `design-intent/contracts` を指し「置き場が無い」で 2。器（scribe2）は repo の根の `contracts/` を読むので、利用者は置き場の深さに合わせて `../contracts` と書く（tsuzuri の気づき (b)）。器の導出 file（`contracts/schema.toml`）を探す根は、便 123 から `note.rs` の `external_path` が「置き場を含む版管理の根・無ければ置き場の親」で解いている（判断の記録 ADR-16 決定 (2)(キ)）。
2. **命令の説明の字が古い（`crates/folio/src/main.rs`）。**
   - `folio face` = 「正本から見本 3 面の 1 面を導出して書く…本便で生成器を持つのは憲法の面だけ」。見本 3 面は 2026-09-18 に退役し、`--face` は 5 面（index・constitution・srs・adr・note）を受ける。
   - `folio check` = 「design-intent の正本 7 file の形を検査し」。床は 7 file のほかに判断の記録・設計ノート・凍結 anchor・索引の欄の決まり・参照 id と、索引が組めるか（`main.rs` が素の床の後に撃つ）を数える（`check.rs` の頭の注）。
   - `folio serve` = 「配信先を tailnet の内側だけで見せる（bind 先が tailnet の外なら起動を拒む）」。実装（`serve.rs` の `inside`）と条 N-6.1（第 1.2 版）は同じ端末の中（loopback）も通す。断りの字「拒否 — bind 先 <住所> は tailnet の外」も同じ（ADR-12 の帰結が次に配信の実装を触る便へ回した字）。
3. **base の歯（参考値）。** workspace の nextest 1050 / 1050・clippy 0 警告・床 4 本 rc 0・`folio build --write` 37 file。`git grep -n f192_ -- crates` は 0 件・行 id `gm` は 0 件。

### (b) 直す先

1. **`crates/folio/src/note.rs`。** 根を解く式を関数 `root_of`（置き場を含む版管理の根・無ければ置き場の親・Err は読めない理由の字）に切り出し、`external_path` はその下の `contracts/schema.toml` にする（器の導出 file の置き場の答えは変えない）。
2. **`crates/folio/src/derive.rs`。** `run` に引数 `from_root` を足す。真なら --out の相対を `note.rs` の `root_of` からの相対に解き、根が解けなければ「まだ分からない」で 2。偽なら今のまま（--dir からの相対）。絶対 path はどちらでもそのまま。
3. **`crates/folio/src/main.rs`。** `folio derive` に旗 `--from-root`（「--out の相対を、置き場を含む版管理の根〔無ければ置き場の親 dir・器の導出 file を探す根と同じ〕からの相対に解く」）を足して `derive::run` へ渡す。--out の説明に「〔--from-root なら根からの相対〕」を足す。説明の字 3 つを次にする。
   - face =「正本から 5 面（入口・憲法・要件書・判断の記録・設計ノート）の 1 面を導出して書く（--write）・検査する（--check）」
   - check =「設計文書の置き場の床（正本 7 file〔憲法・規則の表・語彙・要件書・入口・相談窓口・天井〕の形と、判断の記録・設計ノート・凍結 anchor・索引の欄の決まりと、参照 id と、索引が組めるか）を検査し、合格 0 / 不合格 1 / まだ分からない 2 で終わる」
   - serve =「配信先を同じ端末の中（loopback）か tailnet の中だけで見せる（bind 先がそのどちらでもなければ起動を拒む）」
4. **`crates/folio/src/serve.rs`。** 頭の注と断りの字を「拒否 — bind 先 <住所> は同じ端末の中（loopback）でも tailnet の中でもない」にする（判定の式 `inside` は変えない）。
5. **変えないもの。** --out の既定なし（消費側が宣言する・床の定数の placement の字のまま）・--from-root の無いときの解き方（folio2 の床の `--out ../contracts` はそのまま）・器の導出 file の置き場・導出物の字・serve の判定の式と終了コード・ほかの命令の説明・設計文書の正本・folio2 の床 4 本の結果・`folio build` の出力。

### (c) 歯（f192_・base で 0 件）

binary 経由。期待の字は歯の側の手書き。

1. **`crates/folio/tests/derive.rs`（本文に足す）。** 置き場を 1 段深く（`<根>/docs/di/`）置き、凍結の対の設計ノート derive-anchor.yaml を置いた一時の根。今の dir は `<根>/docs`。
   - **f192_from_root_resolves_out_from_the_repo_root**: 根を版管理の根にして `--dir di --out contracts --from-root --write` は `<根>/contracts/derive-anchor.toml` を凍結の期待と byte で同じに書き（置き場の下と今の dir の下には書かない）、同じ宣言の `--check` は 0、旗が無ければ `di/contracts` を指して 2、1 byte 変えると 1。
   - **f192_from_root_without_a_repo_is_the_parent_of_the_place**: 版管理の根が無ければ置き場の親（`<根>/docs/contracts`）に書く（器の導出 file を探す根と同じ）。
   - **f192_absolute_out_is_taken_as_is_with_or_without_from_root**: 絶対 path の --out は旗の有無で同じ所。
2. **`crates/folio/tests/help.rs`（新）。**
   - **f192_face_help_names_the_five_faces**・**f192_check_help_names_what_the_floor_counts**・**f192_serve_help_names_loopback_and_the_tailnet**: `folio <命令> --help` の 1 行目が (b) 3 の字と等しい。face は --face の 5 つの名とも揃う。
   - **f192_derive_help_names_from_root**: `folio derive --help` に旗と説明が在る。
   - **f192_the_command_list_carries_the_same_texts**: `folio --help` の命令の一覧に古い字 4 つが無く、3 つの説明が在る。
3. **`crates/folio/tests/serve.rs` の既存の歯 1 本の字の期待を直す。** `serve_refuses_a_bind_host_outside_the_tailnet` は 3 つの住所（0.0.0.0・192.168.0.1・8.8.8.8）で終了コード 1 のまま、断りに (b) 4 の字を丸ごと探す（前は「tailnet の外」の部分の字）。
4. **RED の実測。** 歯の file だけ（見本の tests の字）を base に当てると binary の f192_ の 8 本と (c) 3 の歯 1 本がどれも落ちる（base の binary は旗 --from-root を知らず clap が 2 で断る・説明と断りの字が古い・起草の記録の red-192.log・nextest の rc 100）。同じ file の serve_refuses_a_dir_that_holds_version_control は緑のまま。

### (d) 採らなかった形

1. **--out の相対を常に今の dir か根からの相対に変える。** folio2 自身の床（`--out ../contracts`）と、これまでの契約の done の字と、tsuzuri の今の命令が全部変わる。旗を足す形なら既存の命令は 1 字も変わらない。
2. **--out を省いたら根の `contracts/` にする（既定を置く）。** 床の定数の placement の字「path は消費側が宣言する」（生成区間）と食い違い、床の定数と生成区間の直しが要る。旗なら消費側が宣言する形のまま。
3. **根の式を derive.rs に別に書く。** 器の導出 file を探す根と --out の根が 2 つの式になる（P-6.3）。`note.rs` の `root_of` 1 つを共有した。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本の写しで workspace の nextest 1058 / 1058（base + f192_ の 8 本）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 37 file（base と全 file が byte で同じ）。字の期待を直した既存の歯は (c) 3 の 1 本だけ。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、f192_ と serve の断りの歯を撃つ）。** 11 通りとも落ちる（生き残り 0・mut-192.log）。

| 変異 | 落ちる歯（歯の名は (c)） |
| --- | --- |
| M1 --from-root を読まない | f192_from_root_resolves…・f192_from_root_without_a_repo… |
| M2 根を常に置き場の親にする | f192_from_root_resolves… |
| M3 根を今の dir にする | f192_from_root_resolves… |
| M4 版管理の根が無いとき置き場そのものに倒す | f192_from_root_without_a_repo…・serve の歯 2 本（器の導出 file が引けず build が止まる） |
| M5 絶対の --out も根からの相対にする | f192_absolute_out… |
| M6 旗を derive へ渡さない | f192_from_root_resolves…・f192_from_root_without_a_repo… |
| M7 face の説明を古い字に戻す | f192_face_help…・f192_the_command_list… |
| M8 check の説明に古い字を足す | f192_check_help…・f192_the_command_list… |
| M9 serve の説明を古い字に戻す | f192_serve_help…・f192_the_command_list… |
| M10 serve の断りを古い字に戻す | serve_refuses_a_bind_host_outside_the_tailnet |
| M11 --from-root の説明を縮める | f192_derive_help… |
3. **外の置き場（tsuzuri の写し・参考値）。** HEAD 0918967（pin 6467915）で、見本の binary の床 3 本（check・schema --check・derive --out ../contracts --check）と build（45 file）は base の binary と同じ答えと byte。`folio derive --dir design-intent --out contracts --from-root --check` は 0（一致 23・差分 0・根の contracts/）で、旗の無い命令と同じ置き場を指す（base の binary は旗を知らず 2）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file は `+crates/folio/tests/help.rs`。
2. **余地（CapHeadroom）。** write-set の src の 4 本（python と awk の 2 実装で一致・cap-small.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/main.rs` | 698 | 802 | 703（+5） | 797 |
| `crates/folio/src/note.rs` | 1010 | 490 | 1016（+6） | 484 |
| `crates/folio/src/derive.rs` | 426 | 1074 | 434（+8） | 1066 |
| `crates/folio/src/serve.rs` | 419 | 1081 | 419（0） | 1081 |

3. **size は S。** src の増分は +19 で S の見積 100 の内。余地の最小（note.rs の base 490）は S の 100 を超える。
4. **verify は 5 行**で、done の 5 つの塊と 1 対 1 に揃える。見本の写しで 5 行とも rc 0。
   1. `cargo nextest run -p folio --test derive f192_` = (c) 1（3 本）。
   2. `cargo nextest run -p folio --test help f192_` = (c) 2（5 本）。
   3. `cargo nextest run -p folio --test derive` = 便 119・120 の導出の歯（旗の無い命令の解き方が変わらない）。
   4. `cargo nextest run -p folio --test serve` = (c) 3 と配信の歯の全部。
   5. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外・0（通す）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/small の本契約で 契約に起因する断り 0（preflight ok・起草の記録の precheck-192.log）。見本の写しで verify の 5 行とも rc 0。
3. **数え直し。** 受付の時点の本流で `main.rs`・`derive.rs`・`note.rs`・`serve.rs` のどれかが base と違えば、見本を本流に取り込み、余地と verify を数え直してから運ぶ。床の穴のレーンの便 187（見本 impl/d187 は `main.rs` の `folio check` に「面が組めるか」の床を足す）は本便の後に着地し、check の説明の字と歯 f192_check_help_names_what_the_floor_counts の期待に「面が組めるか」を足すのは便 187 の側が持つ（席の割り・2026-09-28）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/small-draft.md`、script と log は同じ dir の small-scripts。組み立てと全体の歯は run-small.sh・build の byte の比べは build-cmp.sh・RED は red-small.sh・突然変異は mut-small.py（`mut-small.py <見本の写し> 192`）・余地は cap-small.sh・外の置き場は prep-tz.sh と tz-small.sh。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** --out の既定・器の導出 file の置き場・導出物の字・serve の判定の式・設計文書の正本・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** 版管理の根は `git rev-parse --show-toplevel`（`gitcheck::toplevel`）の答えで、submodule や worktree の中の置き場ではその中の根になる（器の導出 file を探す根と同じ答え）。
3. **撤退条件。** (1) 要件書・判断の記録・憲法の字を変えないと書けないと分かったら、止めて席へ返す。(2) 本便の後に (c) 3 のほかの既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に folio2 自身の床 4 本の結果か `folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `folio derive --from-root`・`note.rs` の `root_of`・face / check / serve の説明の字・serve の頭の注と断りの字・歯の f192_ の 8 本と serve の歯 1 本の字の期待。
- 入れない: --out の既定・床・面・設計文書の正本・外の置き場・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| root | 根の式 | `note.rs` の `root_of`（`external_path` と `--from-root` が共有） |
| flag | 旗 | `main.rs` の `--from-root`・`derive.rs` の `run` の `from_root` |
| help | 説明の字 | `main.rs` の face / check / serve の doc comment |
| refuse | 断りの字 | `serve.rs` の `bind_host` の字と頭の注 |
| teeth | 歯 | `tests/derive.rs` の f192_ 3 本・`tests/help.rs` の f192_ 5 本・`tests/serve.rs` の 1 本の字 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 185（本流 6467915）。
- 本便の着地の後に席が見ること: 台帳 f2-648.228 と f2-648.185 を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ「`folio derive --dir design-intent --out contracts --from-root` が使える（旗の無い命令は今のまま）」を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gm"
title = "folio derive に旗 --from-root を足し、face / check / serve の説明の字と serve の断りの字を今の実装に揃える（台帳 f2-648.228 と f2-648.185・判断の記録 ADR-12 の帰結）: crates/folio/src/note.rs は根を解く式を関数 root_of（置き場を含む版管理の根・無ければ置き場の親）に切り出し external_path はその下の contracts/schema.toml にする。crates/folio/src/derive.rs の run は引数 from_root が真なら --out の相対を root_of からの相対に解き（根が解けなければ まだ分からない）、偽なら今のまま --dir からの相対、絶対 path はそのまま。crates/folio/src/main.rs は folio derive に --from-root を足して渡し、face の説明を 5 面に、check の説明を床の数える物に、serve の説明を同じ端末の中（loopback）か tailnet の中に直す。crates/folio/src/serve.rs は頭の注と断りの字を 同じ端末の中（loopback）でも tailnet の中でもない にする（判定の式は変えない）。--out の既定なし・導出物の字・設計文書の正本・folio2 の床 4 本と folio build の出力は変えない。歯は tests/derive.rs の f192_ 3 本と tests/help.rs（新）の f192_ 5 本と tests/serve.rs の既存の歯 1 本の字の期待。実装の見本は origin の枝 impl/d192 の commit a85925b（親 6467915）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 6467915"
req = ["FR11", "FR7", "FR5", "FR4"]
section = "1"
write-set = ["crates/folio/src/main.rs", "crates/folio/src/derive.rs", "crates/folio/src/note.rs", "crates/folio/src/serve.rs", "crates/folio/tests/derive.rs", "+crates/folio/tests/help.rs", "crates/folio/tests/serve.rs"]
verify = ["cargo nextest run -p folio --test derive f192_", "cargo nextest run -p folio --test help f192_", "cargo nextest run -p folio --test derive", "cargo nextest run -p folio --test serve", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "crates/folio/tests/derive.rs の f192_ の 3 本（版管理の根からの相対の --out に書いて --check が 0・旗が無ければ --dir からの相対・根が無ければ置き場の親・絶対 path は旗の有無で同じ）が緑、crates/folio/tests/help.rs の f192_ の 5 本（face・check・serve の説明の 1 行目が手書きの字・derive の --from-root の説明・命令の一覧に古い字が無い）が緑、tests/derive.rs の歯の全部（旗の無い導出の命令）が緑、tests/serve.rs の歯の全部（断りの字が 同じ端末の中（loopback）でも tailnet の中でもない）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
