# 設計: 便 159 — 生成区間の置き場が無い CLAUDE.md へ inject --write が区間を作り、区間の外の規範語は folio2 の置き場だけで数える（群 A・台帳 f2-648.238）

- 要件: FR6（憲法を AI の手元に届ける）。規範文と受入基準 AC4 は変えない（§1 (i) の 1）。
- 条: P-14（注入）/ N-2（機構の注「検査対象は folio2 が所有する文書に限る」）/ N-1.1 / P-4.1 / P-10.1。
- 出所: 席の外の置き場の通し（2026-09-27）と台帳 **f2-648.238**。審査の材料は行 `ff` の §1 だけ。新しい file も dir も無い。
- 門: 対象外。base・本便・本流の binary とも **0（通す・設計文書の正本を書き換えない便）**。
- **base = main 346f825（便 158 の着地の後）。数は写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 改訂 b（2026-09-27・席の裁定）: 区間の外の規範語の検査を、条 N-2 の機構の注どおり folio2 の所有する CLAUDE.md に限り、外の置き場では判定の外の 1 行で知らせる（§1 (b) の 5・(c) の 5）。数は撃ち直した。
- 改訂 c（2026-09-27・検証役の記録 d159-verify.md・席の裁定 台帳 f2-648.238 notes 07:44 JST）: blocking B-1・B-2 の歯（(c) の 1・5）と非 blocking 1〜9（絶対の path・行き先の無い symlink・表が空・往復の歯の字・(b)2 の改行の規則・(b)5 の理由と行 D-11・(i)1 の古くなる字）を採り、base を 346f825 へ載せ替えて数を撃ち直した。

## 1. 設計

### (a) いま起きていること（base 346f825 の実測・参考値）

1. **外の置き場での答え。** 一時 dir で git の init → `folio init --dir design-intent` → commit の後に inject を撃った。

| CLAUDE.md の形 | base | 本便の後 |
| --- | --- | --- |
| 無い・--write | 2（読めない） | 0（marker の対と区間だけの file）・続く --check 0・2 回目は「差が無い」 |
| marker の無い file（末尾に改行あり・なし・空）・--write | 2（marker が 1 対でない）・byte 不変 | 0（利用者の字の後に足す）・続く --check 0 |
| begin だけ・2 対・--write | 2・byte 不変 | 同じ |
| 無い・--check | 2（読めない） | 同じ（作らない） |
| 区間の外に「…する。」の行・--check | 1（marker を手で置いた後） | 0 と知らせの 1 行 |

2. **止まる所。** `crates/folio/src/inject.rs` の run は、CLAUDE.md を読めなければ（無いを含む）「読めない」、marker が 1 対でなければ（0 本を含む）「marker が 1 対でない」の 2 で終える。init は CLAUDE.md を書かない（init.rs の頭の注・tests/init.rs 436 行・FR22 と AC19 の 11 本）ので、marker を手で書くまで注入が回らない。通しの歯 f152 も marker を手で書いてから撃つ（tests/init.rs 801〜806 行）。
3. **区間の外の規範語。** 利用者の CLAUDE.md に「…する。」「…ない。」で終わる行が在ると、区間を足した後の --check が 1（関数 normative_outside）で、利用者が言い換えるまで通らない。出所の条 N-2 の機構の注は「検査対象は folio2 が所有する文書に限る」と書くが、実装はどの CLAUDE.md にも掛ける（席の裁定で本便が注に揃える）。
4. **通しのほかの手の要る所（既知・運ばない）。** 凍結の前の床と build の 2（--freeze-start は承認欄と列の根の表の行が要る・.233）・derive の置き場の dir（空の dir は版管理に載らない）・ceiling --write の設計ノートの面・ceiling --gate の印。
5. **folio2 自身は変わらない**（marker の対を持ち、置き場の名は folio2 の名）。
6. **base の歯。** nextest 977 / 977・clippy 0・床 4 本 rc 0・build 34 file。`--test inject` 14 本・`--test init` 21 本。`f159_` と `ff` は 0 件。

### (b) 直す先 — `crates/folio/src/inject.rs` の関数 run

1. **無い file。** --write で読めず、かつ dir の項目として無い（関数 absent＝`symlink_metadata` が NotFound）ときだけ、空の file と見て先へ進む。ほかの読めなさ（UTF-8 でない・dir・行き先の無い symlink など）は今のまま「読めない」の 2 で、上書きも先の作成もしない。
2. **marker が 1 本も無い --write。** 利用者の字を 1 byte も変えずに頭に置き（BOM・CRLF・末尾の空白と空行もそのまま）、挟みの後に「begin の marker・改行・本文・改行・end の marker・改行」を足す。挟みは利用者の字の最後の 1 字で決める＝改行なら改行 1 つ、改行でなければ改行 2 つ、空の file と無い file は無し（末尾が空行の file では空行が 2 つになる）。区間は LF で書く。字は「〈絶対の path〉: 無いので区間だけの file を作った（n 行 / m byte）」か「〈絶対の path〉: marker が無いので末尾に区間を足した（…）」で 0（path は `std::path::absolute`＝相対の --claude-md でも作った場所が分かる）。R-2 の上限は今のまま読む前に検査する。
3. **書く口を 1 つに。** 関数 write_md（書けなければ「書けない」の 2）を足し、今の置き換えの道もそれを使う（字と終了コードは同じ）。頭の注に 1 行足す。
4. **`crates/folio/src/main.rs` の説明 2 行。** --write に「CLAUDE.md が無ければ区間だけの file を作り、marker が 1 本も無ければ末尾に区間を足す」、--check に「区間の外の規範語の行（folio2 の置き場だけ・外の置き場は数えない 1 行を出す）」を足す。
5. **区間の外の規範語は folio2 の所有する CLAUDE.md だけで数える（改訂 b）。** 関数 foreign_name は、置き場の憲法の名（`adr::place_name`＝meta.id）が読めて、列の根の表 ROOT_DIGESTS の最初の行（folio2 の行）の鍵と違うときだけ名を返す。名が在れば --check は normative_outside を撃たず、判定の後に 1 行「# 区間の外の規範語の行は数えていない（置き場の憲法の名「〈名〉」が列の根の表の folio2 の行と違う＝folio2 の所有する CLAUDE.md でない・判定の外）」を足す（終了コードは変えない・便 156 の行 R-17 の知らせと同じ形）。名が読めないか表が空なら数える（表の最初の行が無ければ None を返す形・P-4.1）。骨格の名「未記入」は数えない側＝比べるのは名の同一性で、未記入 が folio2 の名でないことは決まる（便 158 が承認欄の 未記入 を空と見るのは欄が埋まったかの判定で食い違わない・空と見ると外の置き場の手直しが戻る＝V14）。**この判じ方を選んだ理由**: 列の根の表は憲法の名を鍵にした既存の型付きの表で、最初の行が folio2 の行（floor_adr.rs の注と歯 f134 が先頭に縛る）＝folio2 専用の定数を新しく焼かない（ADR-16 決定 (2)）。この式は --check が 1 を出すかを分ける判定の式（行 D-11）で、根拠は席の裁定（台帳 f2-648.238 notes 2026-09-27 07:06 JST）。名は inject が既に読む constitution.yaml から便 154 の place_name で読み、鍵は条の id でなく憲法の名（比べた案は (d) の 3）。**条 N-3.1**: folio2 が名を替えて数えを逃れると、同じ名で表を引く床が不合格・違反 2 で落ちる（写しで実測）＝例外の口にならない。
6. **変えないもの。** marker が在って 1 対でない形は今のまま 2 で byte 不変（書く場所を決められない）。--check と --print は作らない（--check は区間が無ければ非 0）。導出・R-2・folio2 の置き場での区間の外の検査・init・列の根の表・folio2 自身の inject の出力（3 つの旗とも 1 byte も変えない）。

### (c) 歯（関数名 f159_・`crates/folio/tests/inject.rs`・binary 経由）

期待は手書きの `tests/fixtures/inject/ok/CLAUDE.md`（「# fixture」・空行・区間）から取る。fixture は足さない。

1. **f159_write_puts_the_region_where_there_is_none。** fixture ok の正本で ① 無い ② 空 ③「# fixture」と改行 ④ 改行の無い「# fixture」に --write が 0 で、①② は ok/CLAUDE.md から頭の 2 行を除いた byte、③④ は ok/CLAUDE.md と等しい。字は ① が「作った」・②〜④ が「足した」。続く --check は 0、2 回目は「差が無い」で byte 不変。BOM・CRLF・末尾の空行・末尾の空白の 4 形（改訂 c・B-1）は、書いた byte が「元の byte・挟み・区間」の手書きの連結と等しく --check 0。相対の --claude-md で撃つと字が作った file の絶対の path を出す。folio2 の写しへの --write で CLAUDE.md は変わらない。**base では ① が 2＝RED。**
2. **f159_write_still_refuses_markers_that_are_not_one_pair。** begin だけ・end だけ・逆順・2 対で --write が 2（marker が 1 対でない）・byte 不変。base でも緑（今の断りを縛る・(e) の V7・V8）。
3. **f159_write_makes_nothing_it_must_not。** UTF-8 でない CLAUDE.md への --write は「読めない」の 2 で byte 不変。無い CLAUDE.md に --check は 2・--print は 0 で作らない。上限を超える fixture over-limit の --write は 1 で作らない。無い dir の下への --write は「書けない」の 2。行き先の無い symlink への --write は「読めない」の 2 で先を作らない（改訂 c）。**base では「書けない」の形が「読めない」＝RED。**
4. **置き換える既存の歯 1 本（`crates/folio/tests/init.rs`・名は変えない）。** f152_the_skeleton_runs_every_command_without_hand_edits から marker を手で書く 5 行を外す。CLAUDE.md を置かずに --write と --check が 0 で、書いた file は marker で始まり marker と改行で終わり、P-1.1 と R-2 を持つ。**base では --write が 2＝RED。**
5. **f159_outside_lines_count_only_in_folio2s_own_place（改訂 b）。** ① `folio init` した骨格（名 未記入）で、利用者の「回答は日本語でする。」の行を持つ CLAUDE.md に --write が 0（利用者の字が頭に残る）・--check が 0 で、知らせの行がちょうど 1 本・名「未記入」。② folio2 の写しの名を表の 2 行目の tsuzuri-constitution に替え、区間の外に「これは規範とする。」を足すと --check 0 と知らせ（名「tsuzuri-constitution」）。③ 名が folio2 のままなら 1 で知らせなし。④ 行を足さない folio2 の写しの --check は 0 で標準エラーは 1 行。① の外の置き場でも区間と正本の差分は数える＝区間の 1 字の差は 1（知らせつき）・空の区間は 2（改訂 c・B-2・条 P-14.2）。**base では ① が 2＝RED。** 名の無い fixture は数える側のまま（inject_fixture_outside_fails は緑）。
6. **既存の往復の歯に 2 句（改訂 c）。** inject_write_round_trip_from_drift に、--write の字が「folio inject: wrote 」で始まり --check の字が「 byte 一致」で終わることを足す（置き換えの道を write_md に移すので字を縛る）。

### (d) 採らなかった形

1. **init が CLAUDE.md を書く。** FR22 の規範文・AC19 の書く file 11 本の閉じた一覧・init の「置き場の外には何も書かない」と食い違い、要件の直しが先に要る。区間は注入の持ち物（条 P-14）で、inject が作れば 1 つの口で閉じる。
2. **--claude-md の既定を版管理の根に替える。** 今の既定は呼んだ場所（条 P-13 の機構の注の書き込み先）で、変えると注の字が動く。作った場所は絶対の path で字に出す（(b) の 2）ので、取り違えは黙らない。
3. **folio2 の所有を別の data で判じる（改訂 b）。** ⓐ 置き場の憲法に同じ機構の条が在るか: 機構は条 N-2 の注の散文にしか無い。条 id「N-2」を鍵にすると台帳 f2-648.236 と同じ借り、注の字で探すと散文の規則（条 N-2.1）。ⓑ 名だけ（便 154 の name_of）: 比べる folio2 の名が表の外に無く、字「folio2」を新しく焼く。ⓒ 表のどれかの行に在るか: 利用者の行（tsuzuri）も数える（(e) の V12）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は (c) の 4 の 1 本だけ。** 本便の写しで nextest **981 / 981**・clippy 0・床 4 本 rc 0・build 34 file は base と diff -r 一致・folio2 の inject の --check・--print・--write の出力（標準出力・標準エラー・終了コード）は base と byte で一致し CLAUDE.md の差 0。`--test inject` 18 / 18・`--test init` 21 / 21。
2. **RED。** 歯だけを base に当てると、歯 1・歯 3・歯 5（(c) の 5）・置き換えた f152 が落ち、歯 2 と (c) の 6 とほかは緑。
3. **突然変異（写しの inject.rs を 1 通りずつ変え、`--test inject` と `--test init` を撃つ・改訂 c で検証役の V1〜V30 を写して V31〜V34 を足した）。** 34 通りのうち 33 通りが落ちる。生き残りは V19（知らせの行を標準エラーの先頭に置く・害が無いので縛らない＝検証役の 10）。「表が空なら外と見る」形は今の表で振る舞いが同じで、歯では殺せない。

| 変異 | 落ちる歯 |
| --- | --- |
| V2〜V5 利用者の字の末尾の空白・CRLF・BOM・末尾の改行の並びを直す・V28 CRLF の file に区間も CRLF で書く | 歯 1（改訂 c の 4 形） |
| V18 差が無くても書く・V25 作った と 足した を分けない・V27 改行でない末尾に改行 1 つ・V31 渡した path の字のまま出す | 歯 1 |
| V1 頭に足す | 歯 1・歯 5 |
| V6 end の後の改行 2 つ・V26 無い・空の file にも頭に改行 | 歯 1・f152 |
| V7 1 対でなければ足す・V8 begin が無ければ足す | 歯 2 |
| V11 --check でも作る・V21 無い dir の親を作る・V24 読めなさを全部 無い と見る・V32 行き先の無い symlink も 無い と見る・V33 書けなくても合格 | 歯 3 |
| V9 外の置き場で区間の差も数えない・V10 外の置き場で空の区間を 2 にしない | 歯 5（改訂 c・B-2） |
| V12 表のどれかの行の名で数えない・V14 未記入 を読めないと見る・V29 外の置き場でも数えて知らせ・V30 知らせを出さない | 歯 5 |
| V15 folio2 でも知らせ | 歯 5・往復の歯 |
| V20 外の置き場の --check を 2 にする | 歯 5・f152 |
| V22 表の最後の行・V23 短い名で比べる | 歯 5・inject_pinned_normative_line_outside_fails |
| V34 いつも区間の外を数えない | 歯 5・inject_fixture_outside_fails・inject_pinned_normative_line_outside_fails |
| V13 名が読めなければ数えない | inject_fixture_outside_fails・往復の歯 |
| V16 wrote の字・V17 一致 の字を変える | 往復の歯（(c) の 6） |

### (f) 大きさ・verify と done の対応

1. **write-set。** 4 本とも印なし: `crates/folio/src/inject.rs`・`crates/folio/src/main.rs`・`crates/folio/tests/inject.rs`・`crates/folio/tests/init.rs`。
2. **余地。** 各行 ceil(字数 / 120)・python と awk で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/inject.rs` | 263 | 1237 | 318（+55） | 1182 |
| `crates/folio/src/main.rs` | 668 | 832 | 668（±0） | 832 |

   src の外は `tests/inject.rs` 249 → 571・`tests/init.rs` 942 → 941。rustfmt --check（edition 2024）の差の塊は 3 本とも base と同じ（0・0・31）。
3. **size は S。** src は inject.rs +55 と main.rs の 2 行の書き換え。
4. **verify は 4 行**で、done の 4 つの塊と 1 対 1。
   1. `cargo nextest run -p folio --test inject f159_` = (c) の 1〜3 と 5。
   2. `cargo nextest run -p folio --test inject` = 注入の歯の全部（AC4 の歯を含む・参考値 18 本）。
   3. `cargo nextest run -p folio --test init` = (c) の 4 と init の歯の全部（参考値 21 本）。
   4. `cargo clippy --workspace --all-targets -- -D warnings`。

   base では 1 が 0 件で 4、2 は 14 本・3 は 21 本で緑、4 は 0 警告。単体の歯は足さない。

### (g) 門・受付・並行の便

門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0（(h) の 3）。便 158（行 fe・anchor.rs・adr.rs・tests/freeze_root.rs）は着地済みで base に含む（本便の write-set と adr::place_name・列の根の表に触らない）。便 160（行 fg・tests/sheet.rs）の契約は本流に在り（01cf7b8・docs だけ）、write-set は重ならない。

### (h) 数え直す手順（行 D-13）

記録は `.local/share/folio2/handoff-2026-09-27/d159-draft.md`、script は同じ dir の d159-scripts（repo の外）。

1. 再現: repro-159.sh（(a) の 1・3・4）。模擬: build-sim-159.sh（apply-159.py・改訂 b の apply-159-b.py・改訂 c の apply-159-c.py・apply-159-teeth.py・c159.patch）・suite-159.sh・floor-159.sh・verify-159.sh。
2. RED: red-159.sh。突然変異・余地・fmt・門・名の替え: mut-159.py・lines-159.sh・fmt-159.sh・gate-159.sh・rename-159.sh。
3. 受付の先撃ち: `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-159.md#ff`。

### (i) 要件との関係・運ばないもの・撤退条件

1. **要件との関係（正本は書き換えない）。** FR6 の規範文は区間が無いときの扱いを定めない。本便は書く場所が 1 つに決まる 2 形（file が無い・marker が 1 本も無い）で書けるようにするだけで、中身・決定性・R-2 の上限は変えない（条 P-14.1・P-14.3）。--check の区間と正本の差分の検出は変えない（条 P-14.2・AC4 の歯は緑）。区間の外の検査は FR6 と条 P-14 の規範文の外（出所は条 N-2 の機構の注）で、実装を注の範囲に揃えるだけ。どちらも規範文の意味は変わらない（行 D-17 に当たらない）。init は今も CLAUDE.md を書かない（tests/init.rs 436 行は緑）。**古くなる字（全数）**: 設計文書は FR22 の注と ADR-16 決定 (3) の「骨格は注入の対象にしない（M3 の外）」だけ（便 152 の (i) ④・台帳 .189 が控え済み・次の節目で直す）。古くならないもの＝FR6・AC4・語彙の 注入 と 生成区間・R-2・CON9・憲法の書き込み先の注・相談窓口の q5・N-2 の機構の注 ②（実装と揃う）。実装の注（write-set の外・控え）＝`crates/folio/src/schema.rs` 2 行目の「憲法の生成区間（inject.rs）と同じ型」（marker が無いときの --write が違う）。過去の記録（書き換えない）＝delivery-2.md §1・delivery-152.md の (a) の表と f152 の説明と done。**P-5.6 の借り**: normative_outside の式（「する。／ない。」で終わる行・`<!--` を除く）と本便の判じ方の式は型付きの写しを持たない（行 D-11）＝台帳 f2-648.239 の一括に積む（本便では運ばない）。
2. **運ばないもの。** (a) の 4 の手の要る所（凍結の前の床・derive の置き場・ceiling の面と印）・--claude-md の既定・列の根の表・設計文書の正本・台帳への記帳・外部 crate。
3. **撤退条件。** (1) (c) の 4 のほかの既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で folio2 自身の床 4 本・`folio build` の出力・inject の --check / --print / --write の出力と CLAUDE.md が着地の直前と 1 byte でも違ったら止めて席へ返す。(3) 受付の時点の main で inject.rs の run・列の根の表の最初の行・`adr::place_name`・tests/inject.rs の口（folio_inject・temp_dir・pinned・edit）・tests/init.rs の f152 が base と違えば、(h) で数え直してから運ぶ。

## 2. 範囲

- 入れる: §1 (b) の 1〜5、(c) の歯 4 本と置き換え 1 本。
- 入れない: 区間の外の規範語の数え方（folio2 の置き場では今のまま）・init・--check と --print・--claude-md の既定・設計文書の正本・新しい fixture と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| place | 区間の置き場 | inject.rs の run が無い file と marker の無い file に区間を作る |
| scope | 数える範囲 | inject.rs の foreign_name が外の置き場で区間の外を数えず 1 行で知らせる |
| teeth | 歯 | tests/inject.rs の f159_ 4 本と tests/init.rs の f152 の置き換え |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い（base = main 346f825）。並行の便は §1 (g)。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。.238 を閉じる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ff"
title = "群 A（台帳 f2-648.238）: 外の置き場は CLAUDE.md か marker が無いと folio inject --write が まだ分からない で止まる。inject.rs の run を、--write のときだけ、CLAUDE.md が dir の項目として無ければ marker の対と区間だけの file を作り、marker が 1 本も無ければ利用者の字を 1 byte も変えずに挟みの改行の後の末尾に足す形にする（字は作った file の絶対の path）。改訂 b（席の裁定）: --check の区間の外の規範語は、条 N-2 の機構の注どおり、置き場の憲法の名が列の根の表の最初の行（folio2 の行）と同じか読めないときだけ数え、外の置き場は判定の外の 1 行で知らせる（区間と正本の差分は外の置き場でも数える）。1 対でない形・行き先の無い symlink・--print・R-2・init・folio2 自身の inject の出力は変えない。main.rs の説明 2 行を直す。歯は tests/inject.rs の f159_ 4 本と既存の往復の歯の 2 句と tests/init.rs の f152 の置き換え。門の対象外。base = main 346f825"
req = ["FR6"]
section = "1"
write-set = ["crates/folio/src/inject.rs", "crates/folio/src/main.rs", "crates/folio/tests/inject.rs", "crates/folio/tests/init.rs"]
verify = ["cargo nextest run -p folio --test inject f159_", "cargo nextest run -p folio --test inject", "cargo nextest run -p folio --test init", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/inject.rs の f159_ の 4 本（CLAUDE.md が無い・空・改行あり・改行なし・BOM・CRLF・末尾の空行・末尾の空白の 8 形に --write が 0 で、書いた byte が元の byte と挟みと手書きの ok/CLAUDE.md の区間の連結と等しく、--check 0・2 回目は 差が無い・相対の path でも字は絶対の path・folio2 の写しは不変 / 1 対でない 4 形に --write が 2 で byte 不変 / UTF-8 でない file・行き先の無い symlink・--check・--print・上限超え・無い dir で作らず上書きしない / init の骨格の置き場と名を tsuzuri-constitution に替えた folio2 の写しでは区間の外の規範語の行が在っても --check 0 と知らせの 1 行、骨格の置き場でも区間の 1 字の差は 1・空の区間は 2、名が folio2 なら 1 で知らせなし、行が無ければ標準エラーは 1 行）が緑、tests/inject.rs の歯の全部（AC4 の歯と、wrote と 一致 の字を縛る往復の歯を含む）が緑、tests/init.rs の歯の全部（根に CLAUDE.md を置かずに注入が 0 になる f152 の置き換えを含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio2 の inject の --check・--print・--write の出力と CLAUDE.md と folio build の出力が着地の直前の main と byte で変わらない"
<!-- contracts:end -->
