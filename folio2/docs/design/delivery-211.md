# 設計: 便 211 — tailnet の範囲の住所の字を、歯の中でも部品の数から組み、追跡される file の字を数える（S）

- 要件: FR7（組み立てて、見せる・serve は同じ端末の中か tailnet の中だけで配信する・第 1.57 版）。規範文・確かめ方・受入基準は変えない。
- 条: 規則の表の行 D-8（版管理へ載せる file に tailnet の住所・機器名・口座名を書かない・種別は人の目の確かめ）・N-6.2（配信の出力を公開ネットに晒さない）・P-3.1（機械で決定的に検査できる項目は床に置く）。
- 出所: tsuzuri の席の求め（2026-09-29）。folio2 を持ち込むと、tsuzuri の公開の走査（追跡される file の字に tailnet の範囲の IPv4 の住所の形を探す）が folio2 の 4 行に当たる。席が folio2 の追跡される file を全部走査して確かめた。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `hf` が指す §1 だけ。write-set 2 本（src 1・歯の file 1）・新しい file も dir も無い。
- 門: **0（通す）**。本流の binary で `folio ceiling --gate --dir design-intent --write-set crates/folio/src/serve.rs crates/folio/tests/serve.rs` の答えは「通す（設計文書の正本を書き換えない便）」。
- 前の便: **base = 本流 0c910db に、この契約と同じ枝 `docs/f2` の文の直し（fdd4326・便 17 の契約の 2 行）を足した木**。数は base の写しの実測（参考値・行 D-13）。組み直す手順は控え `~/.local/share/folio2/handoff-2026-09-28/f2-scripts/`（chain.sh・red.sh・mut.py・cap.sh・scan_v4.py）。
- **順（受付の前提）**: 席の取り込みの要求（枝 `docs/f2` = 文の直し + この契約）を**先に本流へ取り込み**、その後で便を受け付ける。本便の歯は、便 17 の契約の文の直しが本流に在るときだけ緑になる（§1 (c)）。
- 見本: origin の枝 `impl/d211`（**2e4980b**・親 fdd4326・1 commit）。`git diff fdd4326 2e4980b` が便の全体の差分（2 file）。作業者は write-set の file をこの commit の中身にしてよい。

## 1. 設計

### (a) いま起きていること（参考値・base の実測）

1. 本流 0c910db の追跡される file（`git ls-files`・783 本）のうち、tailnet の範囲（100.64.0.0/10）の IPv4 の住所の字（範囲そのものの字を除く）は 4 行・6 個。
   - `crates/folio/src/serve.rs` の単体の歯 serve_binds_only_inside_the_tailnet_or_loopback の 2 行（内側の例の範囲の先頭と末尾・in_tailnet の例 1 つ）。
   - `docs/design/delivery-17.md` の §1 (b) 4.（tailscale の名前引きの住所）と §1 (d) の単体の歯の説明（範囲の先頭と末尾）の 2 行。
2. 後の 2 行は席の持ち分（設計ノートの文）で、同じ枝 `docs/f2` の fdd4326 で意味を変えずに住所の字でない言い方へ直した（名前引きの住所は「tailnet の範囲の中の固定の 1 つ」、範囲の例は「範囲の先頭と末尾」）。base（fdd4326）で残るのは serve.rs の 2 行・3 個。
3. 実装の側（serve.rs の in_tailnet・inside・名前引きの住所の定数 TAILSCALE_DNS）に住所の字は無い。in_tailnet は範囲を数（頭が 100・2 つ目が 64 以上 128 未満）で持ち、TAILSCALE_DNS は部品の 4 つの数から組む。範囲を書いた注は `/10` の付いた範囲そのものの字で、走査に当たらない。
4. IPv6 の tailnet の住所の頭と tailnet の名の終わりの字は、本流の追跡される file に 0 件（同じ独立の走査で数えた）。本便は IPv4 の住所だけを扱う。
5. 過去の commit の本文に在る住所の字（便 17 の着地の commit）は、tsuzuri の側が走査の範囲の直しで見ない。本便は commit の本文を扱わない。

### (b) 直す先（変えないもの）

1. `crates/folio/src/serve.rs` の serve_tests の区間の serve_binds_only_inside_the_tailnet_or_loopback: 住所を字の parse でなく、部品の 4 つの数の配列から組む。内側の例 3 つ（loopback の 1 つ・範囲の先頭・範囲の末尾）・外の例 4 つ（範囲の末尾の次・未指定の住所・私設の 2 つ）・in_tailnet の 2 例（範囲の中の 1 つは真・loopback は偽）は、同じ住所・同じ真偽のまま。断りの字（〜は内側のはず／〜は外のはず）も同じ（住所の表示は同じ字になる）。
2. 判定の実装（in_tailnet・inside・bind_host・tailnet_address）と、ほかの単体の歯 3 本は変えない。
3. `crates/folio/tests/serve.rs` に歯を 1 本足す（(c)）。頭の注に便 211 の 2 行を足す。既存の歯 6 本は変えない。

### (c) 歯（f211_・base で 0 件）

`crates/folio/tests/serve.rs` の f211_tracked_files_hold_no_tailnet_ipv4_address（binary を撃たない・file を読むだけ）。

1. 判定の形（tsuzuri の公開の走査の IPv4 の判定と同じ形を、この file の中で独立に書く）: 字の各位置から、前の字が数字でも `.` でもない所で始まり、1〜3 桁の数 4 つを `.` で繋いだ字で、どの数も 255 以下、後ろに `.` と数字が続かず、頭の数が 100・2 つ目が 64〜127 のもの。範囲そのものの字（範囲の先頭の後に `/10` が続き、その後に数字が続かない）は数えない。
2. 判定の見本を先に確かめる（凍結の土台・P-10.1）: 当たる = 範囲の中の 4 つの住所（範囲の先頭・名前引きの住所・範囲の末尾・先頭の次）を、そのまま・前に英字・括弧と port 付き・日本語の句点の前に置いた字と、3 行目に置いた字（行の番号 3）と、範囲の先頭の後に `/100`。当たらない = 前に数字か `.`・後ろに `.5`・範囲そのものの字（地の文の中も）・範囲の外の 4 つ（先頭の前・末尾の次・頭が 10・頭が 101）・4 桁の部分・256 の部分・1000 の部分。
3. 走査: repo の根で `git ls-files -z` を撃ち（落ちたら歯が落ちる）、一覧にこの歯の file が在ることを確かめ（一覧が読めていない場合を合格にしない・P-4.1）、作業の木に在る file をすべて byte で読んで数える（作業の木から消した file は飛ばす）。当たりが 1 つでも在れば、場所（path と行の番号）だけを出して落ちる（住所の字は出さない）。
4. 住所の字は歯の中でも部品の 4 つの数から組む（この file も走査の対象で、自分自身に当たらない）。
5. **RED**: base（fdd4326）に歯の file だけを当てると、この歯が落ちる（当たり 3 件 = serve.rs の 2 行）。便 17 の文の直しも外した本流 0c910db の木では当たり 6 件（4 行）で落ちる。見本 2e4980b では緑。どちらの件数も、独立の python の走査（控えの scan_v4.py）の数と一致した。
6. serve_binds_only_inside_the_tailnet_or_loopback は base でも見本でも緑（意味を変えない直し）。

### (d) 採らなかった形

1. **単体の歯を src の serve.rs の中に置く**: write-set は 1 本で済むが、追跡される file の全部を読む歯は serve の判定の単体ではない。行 D-8 の断り（版管理へ tailnet の住所を書かない）を頭に持つ `tests/serve.rs` に置いた。
2. **歯の判定を tsuzuri の走査の写しにする（または呼ぶ）**: 外の repo の code に依ると、検査が検査される側と同じ物になる（P-10.2）。判定の形だけを揃え、実装はこの file で独立に書き、判定の見本で固定した。
3. **行 D-8 の種別を機械の検査へ変える**: 行の変更は裁定 id が要る（P-17.1）。本便は歯を足すだけで、行は変えない（(g)）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. 既存の歯で落ちるものは無い。見本 2e4980b の workspace の nextest は 1183 / 1183（参考値・base の本数はこれより f211_ の 1 本少ない）・clippy 0 警告。
2. 突然変異 16 通り（控えの mut.py・見本の木で 1 つずつ当てて絞り込んだ歯を撃つ）: 期待と違う 0。
   - 落ちる（捕まえる）14: 単体の歯を字の形へ戻す・便 17 の文の直しを戻す・in_tailnet の上の端を 1 つ縮める／下の端を 1 つ縮める／上の端を 1 つ広げる／頭の数を変える・inside から loopback を外す・追跡される新しい file に範囲の中の住所・歯の判定の上の端を縮める／後ろの `.` と数字の断りを外す／範囲そのものの字の断りを外す／前の数字か `.` の断りを外す／255 の上限を外す・歯の走査の根を crate の dir にずらす（一覧にこの歯の file が無い）。
   - 緑のまま（当たらない主張の確かめ）2: 追跡しない file に範囲の中の住所・追跡される file に範囲そのものの字と範囲の外の住所だけ。

### (f) 大きさ・余地・verify と done

1. **write-set 2 本**（印なし）: `crates/folio/src/serve.rs`・`crates/folio/tests/serve.rs`。差分は +148 −9（参考値）。
2. **余地（参考値）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致（控えの cap.sh）。

| file | base | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: |
| serve.rs | 419 | 424 | 1076 |

3. **size は S**（余地 1076 ≥ 100）。
4. **verify は 4 行**で、done の塊と 1 対 1: `--test serve f211_`（本便の歯 1 本）・`--test serve`（serve の歯の file の全部 7 本）・`--bin folio serve_tests`（serve.rs の単体の歯 4 本）・clippy。見本では 4 行とも rc 0（1・7・4 本と 0 警告）。base（fdd4326）では f211_ の行が 0 件で終了コード 4、ほかは緑（6・4 本と 0 警告）（控えの verify.sh）。

### (g) 受付・並行の便・運ばないもの

1. **受付の順**: §0 の順のとおり、`docs/f2` の取り込みの後。取り込みの前の本流で受け付けると、本便の歯は便 17 の契約の 2 行で落ちる（当たり 3 件）。
2. **並行の枝との重なり**（2026-09-29 19:3x の origin の枝・読むだけ）: 2 本の file を書く未取り込みの枝は impl/d192 だけで、その中身は本流に着地済み（便 192）。2026-09-28 以後に commit の在る未取り込みの枝のうち、本流との差の file に範囲の中の住所の字を足す枝は無い。受付の時点で本流が 0c910db と違えば、この歯を本流の木で撃ち直してから運ぶ（後の便が住所の字を足していれば、その便の側で字を直す）。
3. 本便の後は、追跡される file に範囲の中の住所の字を足す変更は workspace の nextest で落ちる（便の契約・設計ノート・fixture を含む）。住所が要る歯は部品の数から組む。
4. 運ばないもの: 行 D-8 の種別の変更（持ち主の裁定の要る行の変更）・IPv6 の住所と tailnet の名の歯（本流に 0 件で、求めは IPv4 だけ）・commit の本文。

### (h) 床・面・天井が変わらないこと

設計文書の正本も生成器も触らない。見本で床 4 本（check・inject --check・schema --check・derive --check）は base と同じく rc 0（check は合格・違反 0・まだ分からない 0）、`folio build` の出力は base と file の数（40）も sha も同じ（参考値・控えの run.sh と chain2.sh）。門は「通す（設計文書の正本を書き換えない便）」。

## 2. 範囲

- 入れる: §1 (b) の 1 と 3（単体の歯の住所の組み方と、f211_ の歯 1 本）。
- 入れない: 判定の実装・ほかの歯・便 17 の契約の文（席の枝で先に運ぶ）・規則の表の行・新しい file と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| U1 | 住所の組み方 | serve.rs の単体の歯の住所を部品の 4 つの数から組む |
| U2 | 追跡される file の字の歯 | tests/serve.rs の f211_ の歯と、その判定の 2 つの下請けの関数 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提は `docs/f2`（便 17 の契約の文の直し）の本流への取り込み。
- host に要る命令: git（歯が `git ls-files` を撃つ・器の許す命令の内側）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。tsuzuri の席へ、folio2 の追跡される file の IPv4 の住所の字が 0 になったことを知らせる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "hf"
title = "tsuzuri の公開の走査に当たる tailnet の範囲の IPv4 の住所の字を folio2 の追跡される file から無くし、歯で数える（規則の表の行 D-8 の IPv4 の住所の部分）。src/serve.rs の単体の歯 serve_binds_only_inside_the_tailnet_or_loopback の住所を字の parse でなく部品の 4 つの数から組む（内側 3・外 4・in_tailnet の 2 例と判定の実装は変えない）。tests/serve.rs に f211_ の歯を 1 本足し、判定の見本を先に確かめてから、git ls-files の全部に tailnet の範囲（100.64.0.0/10）の IPv4 の住所の字（範囲そのものの字は除く）が 0 件であることを数える（住所の字は歯の中でも部品から組む）。base = 本流 0c910db + 同じ枝 docs/f2 の便 17 の契約の文の直し（先に本流へ取り込む）"
req = ["FR7"]
section = "1"
write-set = ["crates/folio/src/serve.rs", "crates/folio/tests/serve.rs"]
verify = ["cargo nextest run -p folio --test serve f211_", "cargo nextest run -p folio --test serve", "cargo nextest run -p folio --bin folio serve_tests", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/serve.rs の f211_ の歯 1 本（判定の見本と、追跡される file の tailnet の範囲の IPv4 の住所の字が 0 件）が緑、tests/serve.rs の歯の全部が緑、serve.rs の単体の歯（serve_tests の区間・範囲の判定を含む）の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と byte で同じ"
<!-- contracts:end -->
