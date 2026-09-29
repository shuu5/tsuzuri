# 設計: 便 168 — 図の道具（archify）を組み立て時に焼き、置き場の親に写しが無いときだけ一時の置き場へ書き出して撃つ（台帳 f2-648.251・f2-648.245 の便 A3）

- 要件: FR15（図は道具を子の処理で呼んで生成し、凍結 anchor が落ちたら まだ分からない）・FR7（全面を配信先へ生成する）・FR4（要件書の面を 1 つの生成器で出す）。規範文・確かめ方・受入基準は変えない。どの規範文も道具の置き場を定めていない（§1 (i) の 1）。
- 条: P-6.3（焼く元は repo の vendor/archify/ 1 つ・binary はその導出物）/ P-4.1（在るのに使えない写しは焼いたものへ替えない）/ P-10.3（焼いた道具にも凍結 anchor を掛ける）/ N-1.1（消すのは folio 自身が作った一時の置き場だけ）/ N-3.1（旗も環境の値も足さない）。
- 根拠: 判断の記録 **ADR-27** 決定 (1)「面の生成を外の置き場へ広げる」の実装の取りこぼし（決定 (2) は様式 2 file と部品目録だけを焼いた）。「無い は見つからないときだけ」は決定 (2) の規則をそのまま使う。ADR-4 決定 (5)・行 R-15（道具 2.16.0・要約値 sha256・写し 43 file）・vendor/README.md（許諾 MIT）の字は変えない。
- 出所: 調べ facestops-study.md §8（10 番目の止まる所・52 周目の実態 F-1）と台帳 **f2-648.251**。
- 置き場: 審査の材料は行 `fo` が指す §1 だけ。write-set は 4 本（組み立ての script 1・src 2・歯の file 1）で、新しい file も dir も無い。
- 門: 対象外。作業ツリー planner-d168 の一番上で base の binary に write-set 4 本を渡すと **0（通す・設計文書の正本を書き換えない便）**。base と本便の写しでも 0。
- **base = origin/main 8472b5c（便 161 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 改訂 a: 独立の検証（d168-verify.md・合格）の V10 を席の裁定で blocking に上げ、歯 6（焼いた道具にも凍結 anchor が掛かる）を足した。あわせて V5 を落とす歯 5 を足し、歯 1 の後始末の確かめを「TMPDIR に何も残らない」に広げ（V1b）、(e) の 4 に時刻を変えない書き換えの 1 行を、(b) の 2 に途中が壊れた symlink の 1 句を、(g) の 2 に一括 30 を足した。src・write-set・verify・size は変えない。

## 1. 設計

### (a) いま起きていること（参考値・base 8472b5c）

1. **道具の置き場が固定。** `crates/folio/src/figure.rs` の関数 tool_path は道具を「--dir の親 dir / vendor/archify/bin/archify.mjs」（定数 TOOL）に要り、無ければ Err「図の道具が無い」。呼ぶのは render（面の図 1 枚ごと）と check_anchor（凍結 anchor の照合・process の中で 1 度）。build は全部か無しかなので、要件書・判断の記録・設計ノートに図が 1 枚でも在ると 0 file になる。folio figure と folio face も同じ口を通る。
2. **骨格からの実測。** 一時 dir で git の init → `folio init` → 凍結 anchor の型付き記述（`tests/fixtures/figure/anchor/spec.json`）の図 1 枚を足して commit → `folio build --write`。

| 場面 | base | 本便の後 |
| --- | --- | --- |
| 要件書に図 1 枚・親に写し無し | **rc 2・0 file・「vendor/archify/bin/archify.mjs: 図の道具が無い」** | rc 2・6 file・148,403 byte |
| 要件書に図 1 枚・親に vendor/archify を写す | rc 2・6 file・148,403 byte | 同じ |
| 設計ノートに図 1 枚・親に写し無し | **rc 2・0 file（同じ字）** | rc 2・7 file・153,912 byte |

   素の check はどれも rc 2（違反 0・まだ分からない 2＝骨格の凍結の基準の不在）で、床は道具を撃たない。build の rc 2 はこの床の答え（FR5）。本便の後、親に写しが無い場合の出力は、写しを置いた場合と diff -r で一致した。
3. **folio2 自身と組み立て。** repo の根の vendor/archify/（43 file・2,004,077 byte）を撃ち、build は 34 file・2,146,076 byte。`crates/folio/build.rs` は部品目録と憲法の型の一覧を焼いているが、道具の写しは binary に入っていない。実行時に R-15 の要約値は確かめていない（版の違いは凍結 anchor が拾う）。
4. **base の歯。** workspace の nextest 985 / 985・clippy 0 警告・床 4 本 rc 0・`--test figure` 23 本。`f168_` と行 id `fo` は 0 件。

### (b) 直す先

1. **焼く（build.rs）。** 定数 TOOL（`../../vendor/archify`）と関数 tool_files・walk を足す。写しの全 file を相対 path の byte 順（vendor/README.md の要約値の規則と同じ順）に並べ、（相対 path・`include_bytes!`）の対の列 TOOL_FILES を OUT_DIR の archify_files.rs に書く。symlink など file でも dir でもないもの・UTF-8 でない名・**LICENSE が無い**は組み立てを失敗させる（MIT の写しを許諾の字なしに焼かない）。再組み立ての条件は `cargo:rerun-if-changed=<vendor/archify>` の 1 行（dir を名指すので中の file の変更・足す・消すで走り直す・(e) の 4）。
2. **選ぶ（tool_path）。** 親 dir の vendor/archify が `parts::absent`（symlink を辿らずに引いて NotFound のときだけ真・便 153 の関数）なら焼いた道具を使う。在れば今までどおり入口（bin/archify.mjs）を要り、入口が symlink・無い は Err（まだ分からない）。判定を入口でなく写しの dir で行うのは、入口だけ欠けた写しは壊れた写しで、焼いたものへ替えると P-4.1 に反するため（既存の歯 3 本がこの形を縛る・(e) の M2）。途中の vendor そのものが壊れた symlink なら vendor/archify は NotFound で焼いた道具を使う（便 153 の preview/ と同じ振る舞い・歯には無い）。
3. **書き出す（baked_tool・unpack）。** 焼いた道具が要る最初の 1 回だけ（OnceLock・1 回の命令で 1 度）、`$TMPDIR/folio-archify-<process の番号>-<ns>/vendor/archify/` に 43 file を書いて入口を返す。置き場は `create_dir` で新しく作り（在れば Err）、途中で書けなければ消して Err。凍結 anchor の照合も tool_path を通るので、焼いた道具にも anchor が掛かる（歯 6）。
4. **消す（sweep・main.rs）。** main を「命令を run で回し、終わりに figure::sweep を呼ぶ」形に分ける。sweep は書き出した置き場だけを消し、書き出していなければ何もしない。一時の置き場の名は関数 scratch にまとめ、deliver と共有する（deliver の名は変わらない）。
5. **変えないもの。** 親に写しが在るときの道具と出力（folio2 自身の build は byte 不変）・Node の要件（無ければ今どおり「図の道具を起動できない（node）」）・R-15・ADR-4・vendor/ の file・仕上がりの段・凍結 anchor の fixture・床・命令の旗と help・Cargo.toml と Cargo.lock。

### (c) 歯（`crates/folio/tests/figure.rs` の binary 経由 5 本 + 単体 1 本）

1. **f168_a_skeleton_with_a_figure_builds_without_the_tool_beside_it。** 一時 dir で git の init → `folio init` → 要件書に凍結 anchor の図 1 枚 → commit。親に写しを置かず、環境の値 TMPDIR を一時 dir の下の tmp にして build --write と --check を撃つ。--write は rc 2 で「書いた（6 file・」を出し、srs.html に図の本体（凍結 anchor の body.svg に便 67・82 の置き換えを当てた byte 列・既存の口 frozen_body_ja）が在る。--check は rc 0。tmp に何も残らず（名に依らず数える）、置き場の親に vendor/archify を作らない（口 builds_with_the_baked_tool）。**base では 0 file・「図の道具が無い」＝RED。**
2. **f168_the_tool_beside_the_place_is_the_one_that_runs。** 親に repo の写しを置き、入口に「撃たれたら写しの中に file ran を足す」1 行を足す（図の出力は変えない＝anchor は保たれる）。build --write は rc 2・6 file・ran が空でない・tmp に書き出さない。base でも緑（保つ歯）。
3. **f168_a_tool_beside_the_place_that_cannot_run_is_not_swapped。** 親の写しを 4 通りに壊して build --write を撃つ。drift（utils.mjs の class の字・既存の口 drift_tool）は「凍結 anchor が落ちた」、no-entry（入口を消す）・dangling（vendor/archify が壊れた symlink）・file（vendor/archify が file）は「図の道具が無い」。どれも rc 2・まだ分からない・配信先も一時の置き場も作らない。base でも緑（保つ歯）。
4. **単体 f168_the_baked_tool_is_the_copy_frozen_in_r15（figure.rs の figure_tests）。** TOOL_FILES が path の byte 順・LICENSE を含む・連結の sha256（crate の sha256）と file の数が、正本 rules.yaml から既存の読み手で読んだ R-15 の value の要約値と「写し N file」に在る。**base では TOOL_FILES が無く組み立てが落ちる（E0425）＝RED。**
5. **f168_a_vendor_dir_without_the_tool_is_no_tool。** 親に vendor/other/x だけを置き（vendor/ を別の用途で持つ置き場）、歯 1 と同じ口で撃つ。写しの dir が無いので焼いた道具で 6 file を書き、tmp に何も残らない。**base では 0 file＝RED。**
6. **f168_the_baked_tool_is_held_to_the_frozen_anchor。** 親に写しを置かず、環境の値 PATH の先頭に「本物の node を撃った後に出力の Archify を Archifx に替える」node の shim を置く（外の端末の実行環境が違う描き方をする場面）。build --write は rc 2・まだ分からない・「凍結 anchor が落ちた」・配信先を作らない・tmp に何も残らない。**base では「図の道具が無い」で字が違う＝RED。**

fixture は足さない。新しい口は git（tests/init.rs と同じ形）・skeleton_with_a_figure・build_in（shim を渡すと PATH の先頭に置く）・leftovers・builds_with_the_baked_tool・real_node・not_swapped だけ。

### (d) 採らなかった形

1. **置き場から上へ辿って探す。** 利用者の repo に道具が無い限り止まり、別の repo の道具を拾いうる。
2. **環境の値か組み立て時の CARGO_MANIFEST_DIR で folio2 の作業ツリーを指す。** 作業ツリーを消すと黙って変わり、値で道具を差し替える隠れた口になる（N-3.1 の向き）。
3. **図だけ まだ分からない にして他の面は書く。** build の全部か無しかと FR15 の意味を変える＝要件の変更（行 D-17）。
4. **置き場の親へ写しを書き出して残す。** 置き場に写しを置かない（ADR-27 決定 (2)）・写しは元とずれる。
5. **図 1 枚ごとに書き出す。** 命令 1 回で（図の数 + 1）回 2 MB を書く。席の裁定は命令 1 回で 1 度。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は 0 本。** 本便を当てた写しで workspace の nextest **991 / 991**（base 985 + 6）・clippy 0 警告。folio2 自身の床 4 本と凍結の 4 つの旗の標準出力・標準エラー・rc と、`folio build --write` の出力（34 file・2,146,076 byte）は base と diff -r で一致。
2. **RED。** 歯だけを base に当てると、`--test figure` 28 本のうち歯 1・5・6 の 3 本だけが落ち（どれも「図の道具が無い」）、`--bin folio f168_` は組み立てが落ちる（E0425）。歯 2・3 は base でも緑で、焼いた道具の枝が ADR-27 決定 (2) の規則を崩さないことを縛る。
3. **突然変異（build.rs・figure.rs・main.rs を 1 通りずつ変え、`--test figure`・`face_srs_figure`・`face_note` と単体の全部 217 本を撃つ）。** 13 通りとも落ちる（V は検証役が足した変異）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 親を見ない（いつも焼いた道具） | 歯 2・3・既存 7 本（anchor_drift_ の 4 本・道具の不在の 3 本） |
| M2 親の入口が使えなければ焼いた道具へ替える | 歯 3・既存の道具の不在の 3 本（figure・face_srs_figure・face_note） |
| M3 凍結 anchor の照合を焼いた道具で撃ち、親の写しに掛けない | 歯 3・既存の anchor_drift_ の 4 本 |
| M4 無いの判定が symlink を辿る（exists） | 歯 3（dangling） |
| M5 無いの判定が dir でないこと（is_dir） | 歯 3（dangling・file） |
| M6 sweep が何もしない | 歯 1・5・6 |
| M7 命令の終わりに sweep を呼ばない | 歯 1・5・6 |
| M8 LICENSE を焼かない（build.rs の断りも外す） | 歯 4 |
| M8b LICENSE を焼かない（断りは残す） | 組み立てが落ちる（「許諾の写し LICENSE が無い」） |
| M9 焼く順を byte 順にしない（逆順） | 歯 4 |
| V1b 固定の名 .archify-cache を使い回し、消さない | 歯 1・5・6 |
| V5 無いの判定を vendor/archify でなく vendor で行う | 歯 5 |
| V10 焼いた道具のときは凍結 anchor の照合を飛ばす | 歯 6 |

   親に写しが在っても書き出す（使わない）変異は、出力も後始末も同じで外から測れない（等価として数えない）。
4. **再組み立ての条件（写しで実測）。** 何も変えなければ build.rs は走らない。写しの file の時刻を変える・file を 1 本足す・消すと 1 回走る。足した間は焼いた数が 44 で歯 4 が「焼いた要約値 … が R-15 に無い」で落ち、消すと 43 で緑。時刻を変えない中身だけの書き換え（`cp -p`・`touch -d`）は、rerun-if-changed の限りで組み直されず binary は古い中身のまま（検証役の実測・歯 4 は緑、既存の版の固定の歯が写しのずれを落とす・CI は毎回組み直す）。

### (f) 大きさ・verify と done の対応

1. **write-set。** 4 本とも印なし: `crates/folio/build.rs`・`crates/folio/src/figure.rs`・`crates/folio/src/main.rs`・`crates/folio/tests/figure.rs`。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/figure.rs` | 700 | 800 | 805（+105） | 695 |
| `crates/folio/src/main.rs` | 668 | 832 | 674（+6） | 826 |

   src の外は build.rs 528 → 604・tests/figure.rs 821 → 1075。rustfmt --check の差の数は 4 file とも base と同じ（2・2・0・2）。
3. **size は S**（src 2 本で +111・最小の余地は figure.rs の 800）。
4. **verify は 4 行**で、done の 4 の塊と 1 対 1。
   1. `cargo nextest run -p folio --test figure f168_` = (c) の 1〜3・5・6。
   2. `cargo nextest run -p folio --bin folio f168_` = (c) の 4。
   3. `cargo nextest run -p folio --test figure` = 図の歯の全部（参考値 28 本）。
   4. `cargo clippy --workspace --all-targets -- -D warnings`。

   base では 1・2 が 0 件で終了コード 4、3 は 23 本緑、4 は 0 警告。本便の写しでは 5・1・28 本緑と 0 警告。

### (g) 門・受付・並行の便・外の依存・大きさ

1. **門** は対象外で 0（冒頭）。受付の先撃ち（precheck）は契約に起因する断り 0（起草役の実測）。
2. **並行の便と重ならない。** 便 162（床の欄の決まり 41 本）・163（face_note・face_labels と歯 3 本）・164（freeze・anchor・gitcheck・adr と歯 3 本）・165（face_srs・face_index_read・face_constitution・init）・166（face_srs_items）・一括 30 / 31（design-intent/ と tests の check・face_constitution・face_srs）の write-set に本便の 4 本は無い。便 162 と一括 30 は rules.yaml を書くが、R-15 の value は変えず（一括 30 は ruling の欄だけ）、歯 4 は value だけを既存の読み手で読む。
3. **条 A-3.1 に当たらないという席の判断の根拠。** crate・Node の版・道具の版・要約値のどれも増えも変わりもしない（Cargo.toml・Cargo.lock・R-15・vendor/ は不変）。binary に入るのは持ち主が 2026-09-19 に A-3.1 で承認した同じ写しの導出物で、MIT の条件（許諾の字を写しに添える）は LICENSE を必ず一緒に焼き一緒に書き出すことで満たす（M8b）。
4. **消すことの範囲（条 N-1.1）。** sweep が消すのは、この process が `create_dir` で新しく作った一時の置き場 1 つだけで、憲法 N-1 の注の「図の道具を撃つために folio 自身が作った一時の置き場」に当たる。管理下の対象（正本・生成した面・支度表・台帳）には触れない。
5. **binary の大きさ。** debug 49,980,312 → 52,034,416 byte（+2,054,104・+4.1%）、**release 4,833,416 → 6,844,896 byte（+2,011,480・+41.6%）**。増分はほぼ写しの中身 2,004,077 byte。

### (h) 数え直す手順（行 D-13）

script と log は `~/.local/share/folio2/handoff-2026-09-27/d168-scripts/`（repo の外）。setup-168.sh（clone）→ sim-168.sh（c168.patch）→ probe-figure.sh（(a) の 2）・nextest と clippy・floor-168.sh・verify-168.sh・gate-168.sh・lines-168.sh・rerun-168.sh・size-168.sh → red-168.sh（r168-teeth.patch）→ mut-168.py → `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-168.md#fo`。

### (i) 要件と判断の記録との関係・残る穴・撤退条件

1. **要件と判断の記録（正本は書き換えない）。** FR15 の規範文は「図の道具を子の処理として呼び」で置き場を定めず、本便の後も Node の子の処理で撃ち凍結 anchor も掛かる。FR7・FR4 の生成は外の置き場の図を持つ面にも届く。ADR-4 決定 (5) の版の固定は R-15 のままで、焼いた写しとの一致は歯 4 が CI で縛る。ADR-27 に帰結を 1 行足すかは席の裁定（行 D-16・D-18 の次の節目の材料）。
2. **残る穴。** 親の写しの要約値は今までどおり実行時に確かめない（違いは anchor が拾う）。命令の途中で process が殺されると一時の置き場が残る（deliver の一時 dir と同じ）。Node の無い端末では図を持つ置き場は今どおり まだ分からない。
3. **運ばないもの。** 道具の版上げ・R-15 と ADR-4 の字・vendor/ の file・Node の版を照らす口・床・命令の旗と help・設計文書の正本・台帳への記帳・外部 crate。
4. **撤退条件。** (1) 本便の後に既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で folio2 自身の床 4 本の結果か `folio build` の出力が着地の直前と 1 byte でも違ったら、止めて席へ返す。(3) 受付の時点の main で figure.rs の tool_path・deliver、main.rs の main、build.rs の main が base と違えば、(h) で数え直してから運ぶ。

## 2. 範囲

- 入れる: §1 (b) の 1〜4、(c) の歯 4 本。
- 入れない: §1 (i) の 3。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| bake | 焼く一覧 | build.rs の tool_files・walk |
| pick | 道具を選ぶ | figure.rs の tool_path |
| unpack | 書き出す・消す | figure.rs の baked_tool・unpack・scratch・sweep と main.rs の main |
| teeth | 歯 | tests/figure.rs の f168_ 5 本と figure.rs の単体の f168_ |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate と新しい dir は無い。前提の便 153（parts::absent）と ADR-27 は着地済み。着地の後に席は本流の target/debug/folio を組み直し、台帳 f2-648.251 を閉じる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fo"
title = "台帳 f2-648.251（f2-648.245 の便 A3・ADR-27 決定 (1) の取りこぼし）: 図の道具 archify の path が --dir の親の vendor/archify に固定で、外の置き場に図が 1 枚でも在ると folio build が 0 file になる。crates/folio/build.rs が repo の vendor/archify/ の全 file（LICENSE を必ず含む）を path の byte 順に include した定数 TOOL_FILES を焼き（rerun-if-changed は vendor/archify の dir）、crates/folio/src/figure.rs の tool_path は親の vendor/archify が parts::absent のときだけ焼いた写しを 1 回の命令で 1 度だけ一時の置き場へ書き出して撃ち、crates/folio/src/main.rs が命令の終わりに figure::sweep でその置き場を消す。親に写しが在ればそれを撃ち、入口が無い・symlink・壊れた写しは今までどおり まだ分からない。焼いた道具にも凍結 anchor が掛かる。folio2 自身の出力は変わらない。歯は tests/figure.rs の f168_ 5 本と figure.rs の単体の f168_。門の対象外。base = main 8472b5c"
req = ["FR15", "FR7", "FR4"]
section = "1"
write-set = ["crates/folio/build.rs", "crates/folio/src/figure.rs", "crates/folio/src/main.rs", "crates/folio/tests/figure.rs"]
verify = ["cargo nextest run -p folio --test figure f168_", "cargo nextest run -p folio --bin folio f168_", "cargo nextest run -p folio --test figure", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/figure.rs の f168_（親に写しの無い骨格の要件書に凍結 anchor の図 1 枚を足して commit し、TMPDIR を一時 dir にした build --write が rc 2 で 書いた（6 file・ を出して srs.html に図の本体〔凍結 anchor の body.svg に便 67・82 の置き換えを当てた byte 列〕を持ち、build --check が rc 0 で、TMPDIR に何も残らず親に vendor/archify を作らない。親に vendor/other だけを置いても同じ。PATH の先頭に出力の Archify を替える node の shim を置くと build --write が rc 2 で まだ分からない と 凍結 anchor が落ちた を出し、配信先を作らず TMPDIR に何も残らない。入口に印の 1 行を足した写しを親に置くと build --write が rc 2 で 6 file を書き印が残り書き出さない。親の写しが anchor の落ちる改変・入口が無い・壊れた symlink・file のときは rc 2 で まだ分からない と 凍結 anchor が落ちた か 図の道具が無い を出し、配信先も一時の置き場も作らない）が緑、figure.rs の単体の f168_（焼いた file が path の byte 順で LICENSE を含み、連結の sha256 と file の数が rules.yaml の R-15 の value に在る）が緑、tests/figure.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
