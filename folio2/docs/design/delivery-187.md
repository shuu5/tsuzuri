# 設計: 便 187 — 床が build と同じ関数で面を組み、面が組めない置き場を面の字のまま違反に数える（床の穴の束）

- 要件: FR5（配信先を組み立てるたびに構造の床を実行し 3 値で返す・実行できなかった検査を合格と表示しない）と FR10（契約表の欄は器の導出 file から読む）。本流の要件書に在る id で、字は変えない。
- 条: P-3.1（決定的に検査できる項目は床）・P-3.3（床の合格を完成として扱わない）・P-4.1（生成できなかった結果を異常なしにしない）・P-6.3（読み手を 2 つにしない＝床は面の関数そのものを呼ぶ）・P-15.2（編集時の guard が後で呼ぶ床の関数を、面と同じにしておく）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 持ち主の指示（2026-09-28 16:3x JST・逐語「床は通るのにページ生成が止まる穴の束は進めて良い」）。台帳 f2-648.249（要件書・入口・憲法の床が面の要る形を見ない）・f2-648.274（判断の記録の帰結の項が写像に読まれても床は合格）・f2-648.242（設計ノートの承認欄の一覧の形の値）・f2-648.180（床の穴の控え＝条の中の行の欄・最上位の節の中の欄・mechanism の鍵の不在と必須の欄）。元の表は f2-648.245（9 か所・うち章の上限は便 179 で済み）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gh` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 13 本（src 6・歯の file 7〔新 1・期待の字を 1 つ変える 1・本文不変 5〕）。縮む file・消す file・新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流 44c14ac の木で本流の binary に write-set 13 本を渡した `folio ceiling --gate --dir design-intent --write-set …` は **0（通す・設計文書の正本を書き換えない便）**（起草の記録の gate-187a.log）。
- 前提: **base = 本流 44c14ac**（便 185 と、小さな直しの便 192〜194 の着地の後・着地の順は席の決め 2026-09-28）。この契約の数は base の写しの実測（参考値・規則の表の行 D-13）。ただし §1 (a) の 2・3（全数と突然変異の割れ）は 6467915 の写しの実測で、便 192〜194 は面の経路の Err を作る行を足さない（3 便の差分の面の経路の src の非 test の行で Err を作る・上げる行の増え 0。便 193 で承認欄の日付の読みが 席の裁定 の行にも掛かるようになった所は、今の行のまま掛かる行が増えた形）。
- 実装の見本: origin の枝 `impl/d187`（commit **ac9ca7b**。3ad67a6〔本便〕→ 94ee033〔本流 6467915 の merge〕→ 0d4de9f・14bca08〔歯の行〕→ 030f475〔改訂 a・読めないを まだ分からない に・段の順と読めないの歯〕→ 10a0912〔本流 44c14ac の merge〕→ 430c9c7〔check の説明の字〕→ ac9ca7b〔改訂 b・読めないの歯に 5 行・src は同じ〕）。`git diff 44c14ac ac9ca7b` が便の全体の差分（8 file・+899 −22・50,950 byte）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout ac9ca7b -- <write-set の file>`）。write-set の外は変えない。
- 並行の便との重なり: 便 185（索引・`graph.rs` ほか）と便 192〜194 は着地済み（base）で、本便は索引の床（`graph::check_index`）の後ろに段を 1 つ足す。便 192 の契約 (g) 3 の席の割りにより、`folio check` の説明の字に 面が組めるか を足すことと、その歯 `f192_check_help_names_what_the_floor_counts`（`tests/help.rs`）の期待の字を合わせることは本便が運ぶ。並行の起草の見本（着地の順は本便が先）との重なり: 便 190（`impl/d190`・検査の信頼）・便 195〜197（写しの負債）は書き換える file の重なり 0（190 の verify の scope の `tests/floor_cases.rs` は両便とも本文不変・本便の見本 430c9c7 との `git merge-tree` は衝突 0・ac9ca7b は歯の file の行だけを足した）。`stamp.rs` と `tests/help.rs` はどの見本も書き換えない。便 198（`impl/d198`・編集時の止め）は `main.rs` が重なる（198 は check の段の `check_dir` と `graph::check_index` を 1 つの床の関数へ移す）＝198 が本便の着地の後に積み直す。198 の起草役の決め（判断の記録 ADR-33 の下書きの決定 (6)・枝 docs/guard 9f971be）は、check の段を「198 の床の関数の後に本便の面の段」とし、編集時の口は面の段を撃たない（口の写しは版管理の外で 測れない が出るので面の段は回らない・口の止めた行は素の床の行の部分集合のまま）。本便の見本 14bca08 に 198 を積んだ写しで両便の歯は合格（198 の起草役の実測・その後の本便の `main.rs` の変更は check の説明の字だけ）。本便の見本と契約の write-set は変えない。

## 1. 設計

### (a) いま起きていること（実測・参考値。2・3 は 6467915 の写し）

1. **床と面の読み手が 2 つ。** 床（`folio check`＝`check::check_dir` と索引の口 `graph::check_index`）は面の生成器を 1 度も呼ばない。面（`folio build`＝`site.rs` の `build_all`）は正本を型付きの木（`cursor.rs` の `X`）で辿り、要る欄が無い・型が違う・値域の外・上限を超える・参照が解けない、で Err を返し、build は全部か無しかで 1 file も書かず 2 で終わる。だから床が合格のまま面だけが止まる所が生まれる。tsuzuri の席は設計文書の YAML を直接書くので、床（事後の強制）だけでは止まらない。
2. **全数（code から・起草の記録の sites.py と sites-6467915.tsv）。** 面の経路（`site.rs`・`face*.rs` 11 本・`figure.rs`）で Err を作るか上げる所は **1019**（189 の関数）。内訳は、正本の形を見る所 **932**、下の層の関数（`adr::`・`note::`・`stamp::` など）の Err を上げる所 **28**、図の道具と環境（道具の置き場・Node・凍結 anchor・道具の出力・様式の file・配信先）**31**、build の経路の外（`folio face` と `folio figure` の口）**28**。床が同じ関数でこの条件を見ている所は、面が床の関数を呼ぶ下の層の 28 の一部（章の上限〔便 179〕など）だけで、形の 932 は床が別の読み方をするか見ない。1 か所ずつ床に写すと読み手が 2 つのまま 932 本の写しになる（P-6.3）。
3. **割れの実測（突然変異・起草の記録の sweep.py）。** 床の土台（`tests/fixtures/floor_base/design-intent/` を git の 1 commit にした写し・素の床は合格 0 / 0）の YAML の欄と一覧の項を 1 つずつ消す・一覧に・表に・別の字に・空に替えた写し 8606 のうち、**床が合格で面が止まる 808**。土台に提案中の判断の記録 1 本と発効した設計ノート 1 本を足した写しの、その 2 file の 493 のうち **55**。合わせて 863・面の字の型 425。この数は、base を図の道具を撃たない測りの binary で測ったもので、図の型付き記述（spec）の中の割れ（図の道具の検査＝archify の schema で止まる）を数えていない。検証役の folio2 自身の写しでは、spec の写し 959 のうち **692**（figures.yaml 344・example.yaml 211・srs.yaml 137）が床 合格で build が止まる（(i) の 2 の (4)）。当てた file ごと（欄が無い / 型が違う / 値域の外 / 参照が解けない / 上限・数 / その他）: 要件書 304・憲法 203・設計ノート 102・入口 87・規則の表 78・支度表 39・判断の記録 30・語彙 10・相談窓口 8・天井 2。folio2 の本流・床の土台・tsuzuri の写し（b229dd9）の素の置き場はどれも床 0 / 0 で build が通る（今の割れ 0）。
4. **台帳の項が表のどこか（どれも 3. の割れの行・(c) の歯の行）。**

| 台帳 | 項 | 面の字（base の build の まだ分からない） |
| --- | --- | --- |
| .249 | 役 道具 の actor が 1 つでない | srs.yaml.actors: role が「道具」の actor が 0 で 1 つでない（2 も同じ形） |
| .249 | 帯の上限 | srs.yaml.actors: 入れる側の帯が 5 で上限 4（部品目録の context-band の max_per_band）を超える（出る側も同じ形） |
| .249 | rail ≤ 7・verdicts ≤ 4・憲法の改訂の段 ≤ 7 | 段が 8 で上限 7・答えが 5 で上限 4（state-strip）・改訂の段が 8 で上限 7 |
| .249 | outputs.from が要件 | srs.yaml.outputs[2].from: 要件 id「AC3」が無い |
| .249 | basis は条だけ | srs.yaml.requirements[0].basis[0]: 条 id「ADR-1」が無い |
| .249 | 承認欄の入れ物の形 | srs.yaml.meta.approval: 一覧でない（入口も同じ）。憲法の承認欄を一覧にすると床が既に落とす（凍結 anchor の承認の写し） |
| .249 | 憲法の amendment の表の形 | constitution.yaml.amendment: 欄 effective_step が無い |
| .274 | 判断の記録の帰結の項が写像（項の字の中の「: 」や「, id: 」で YAML が表に読む） | adr/ADR-11.yaml.consequences[1]: 文字列でない（発効した記録は封が先に落とす＝提案中の記録で起きる・どちらの字の形も同じ字で落ちる） |
| .242 | 設計ノートの承認欄の一覧の形の値 | design-note/decide.yaml.meta.approval[0].who: 文字列でない |
| .180 (3) | 条の mechanism の鍵が無い・条の rationale が無い・rationale の行の ref が無い・north_star の judged_by が無い | constitution.yaml.articles[0]: 欄 mechanism が無い（ほかも同じ形・床の土台で 4 つとも割れ）。条の plain・規範文の strength・mechanism の live の欠けは床が既に落とす |
| .180 (1)(2) | 条の中の行（rationale・retreat）と最上位の節（north_star・amendment・sources の行）の未知の欄 | 面は止まらない（面は知らない欄を読まない）＝割れでなく床だけの穴。本便では閉じない（(i) の 1）。relations と規範文の行の未知の欄は床が既に落とす |

5. **表に在って台帳に無い行。** 3. の 863 のほとんど（目標・受入基準・制約の行の欄〔例 srs.yaml.goals[0]: 欄 text が無い〕・規則の表の行の種別と状態の値域・入口の棚と道の欄・支度表の欄・語彙の行の形 など）。加えて、契約表の行の verify と done を欠く設計ノート（面は「欄 done が無い」で止まる）は、床が器の導出 file の要否 conditional に従って合格にし（既存の歯 `f120_conditional_fields_pass_with_and_without_values` がその答えを固定）、面だけが要る欄にしていた＝面の側の読み違い（FR10）。
6. **base の歯（参考値・44c14ac）。** workspace の nextest の本数 1070（`cargo nextest list`・本流 44c14ac の CI は緑）・床 4 本 rc 0・`folio build --write` 37 file。`git grep -n f187_ -- crates` は 0 件・行 id `gh` は 0 件。
7. **床のほかの段が通す正本の、型付きの木に読めない値。** base では、要件書などに `0o17`・`0x1F` のような正規化できない scalar があると、床は合格のまま build が まだ分からない で止まる（`cursor::load` の型付きの読み）。本便の後は、これも §1 (b) 3 の「型付きの木に読めない」として まだ分からない 1 件に数える（床と build が同じ字・改訂 a の前の見本は [面] の違反に数えていた）。

### (b) 直す先 — 床が build と同じ関数で面を組む（向き (a)）と、契約表の行の 2 欄（向き (b)）

1. **`crates/folio/src/site.rs`。** 床の口（名は check_faces）と違反の種類の定数（名は FACE_KIND・字は「面」）を足す。床のほかの段が何も数えていない（違反・読めない・測れない がどれも 0）ときだけ、build と同じ `build_all` を 2. の図の口と 3. の読み直さない口の中で回し、Err なら面の字のまま種類「面」の違反 1 件に数える（標準出力の行は `[面] <面の字>`）。ただし面の字が 3. の口が覚えた読めないの字を含むなら、違反でなく まだ分からない 1 件に数える（標準出力の行は `# まだ分からない: <面の字>`・床のほかの段と `cursor::load` の決まりと同じ・P-4.2）。様式 2 本（`preview/folio.css`・`preview/folio-ui.js`）の読めないも同じ口を通す。ほかの段が何かを数えていれば回さない（床は既に合格でなく、同じ原因を 2 度数えない・`graph::check_index` と同じ形）。`write_after_floor`（`--write` の床）は索引の口の後でこれを呼ぶ＝床が落ちれば今どおり 1 file も書かず 1。単体の歯 2 本（(c) の 8・9）と冒頭の注 2 行。
2. **`crates/folio/src/figure.rs`。** 図の口（名は dry）を足す: 閉包のあいだだけ `render` は型（閉じた一覧）・型付き記述が表・JSON への写し、までを確かめ、道具の置き場・凍結 anchor・道具を撃たずに空の本体を返す（thread_local の旗・閉包を出たら戻す）。道具と Node と凍結 anchor は床の外（床は Node に依らない・道具の答えは今どおり build が まだ分からない で知らせる）。
3. **`crates/folio/src/cursor.rs`。** 読み直さない口（名は memo）を足す: 閉包のあいだだけ `load` は同じ path の 2 度目から 1 度目に読めた木の写しを返す（読めなかった file は覚えない・閉包を出たら忘れる）。memo は閉包の値と、閉包のあいだに読めないの口（名は unreadable・字をそのまま返し、memo の中なら覚える）を通った字の一覧を返す。`load` の読めない（fs の読みの失敗・UTF-8 でない・YAML として読めない・型付きの木に読めない）はこの口を通し、重複キーは通さない（形の誤り＝床のほかの段でも違反）。判断の記録の面は 1 枚ごとに憲法・規則の表・要件書を読み直す（folio2 で要件書を 35 回）ので、無いと folio2 の debug の床の面の段だけで約 20 秒かかる（便 185 の前の写しの実測・ある形は (e) の 4）。答えは変えない。
4. **`crates/folio/src/main.rs`。** `folio check` が索引の口の後で 1. を呼ぶ（注 1 行）。check の説明の 1 行目（`--help`）の「索引が組めるか」を「索引と面が組めるか」にする（便 192 の契約 (g) 3 の席の割り）。凍結と書き出しの旗の後始末（`freeze::after`）はその後で、今どおり（索引の口の後・凍結の後始末の前という順は (c) の 6 が固定する）。
5. **`crates/folio/src/face_note.rs`。** 契約表の行の `verify` と `done` を在るときだけ出す（器の導出 file `contracts/schema.toml` で need が conditional・要否は床〔`note.rs`〕が導出 file から数える）。無いと done の段落も検証の札も出さない。器の導出 file の読めないは 3. の口を通す（字は今どおり）。ほかの欄は今どおり。
6. **`crates/folio/src/stamp.rs`。** 面の天井の名札のための印（`preview/ceiling-stamp.yaml`）の 読めない と parse できない を 3. の口に通す（字は今どおり・memo の外では今と同じ Err）。印の欄の欠け（例 sources が読めない）は形の誤りのまま。
7. **変えないもの。** 床のほかの段の判定と字・面の生成器の字と出力（素の置き場の build は base と全 file が byte で同じ＝(e) の 3）・図の道具を撃つ build の道・`folio face` と `folio figure` の口・設計文書の正本（生成区間を含む）・憲法と要件書と判断の記録の字・外部 crate。

### (c) 歯（f187_・base で 0 件）

binary 経由の歯は `crates/folio/tests/floor_faces.rs`（新）。写しは床の土台を一時 dir の `design-intent/` に、器の導出 file を写しの根の `contracts/` に作り、提案中の判断の記録 ADR-11（土台の ADR-4 の字から id と状態を替え承認欄を外す）と発効した設計ノート decide（承認欄 1 行・散文の節と判断の表の節）を足して git の 1 commit にする（素の床は合格 0 / 0）。字は作業ツリーだけに当てる。期待の字は歯の側の手書き。

1. **f187_srs_splits_fail_the_floor_with_the_face_words（要件書の 10 行）。** 道具 0・道具 2・入れる側 5・出る側 5・出る側の from が受入基準・rail 8 段・verdicts 5・要件の根拠に判断の記録・目標に text が無い・承認欄が表 1 つ。各行で: 当てる前の床が合格 0 / 0、当てた後の `folio check` が 1 で `[面] <面の字>` の行と「不合格（違反 1・まだ分からない 0）」、`folio build --write` が 1 で「床 = 不合格（違反 1・まだ分からない 0）・書かない」を出し配信先を作らない、面の口 `folio face` が 2 で同じ字を出す（床の字は面の字そのもの）。
2. **f187_other_splits_fail_the_floor_with_the_face_words（入口・憲法・判断の記録・設計ノート・支度表の 7 行）。** 入口の承認欄が表 1 つ・憲法の改訂の段 8・改訂の欄 effective_step が無い・条 P-1 の mechanism の鍵が無い（.180 の (3)）・判断の記録の帰結の項が写像（.274）・設計ノートの承認欄の who が一覧（.242）・支度表の重複キー（形の誤りは面だけが読む file でも違反のまま）。撃ち方は 1. と同じ（判断の記録と設計ノートは `folio face --id`）。
3. **f187_the_face_stage_runs_only_on_an_otherwise_silent_floor。** 同じ面の欠け（道具 0）で、ほかが黙っている写しは `[面]`、最上位に知らない節を 1 つ足した写しは違反 1 件のまま `[面]` なし、版管理の無い写し（測れない 1 件）は まだ分からない のまま `[面]` なし。
4. **f187_the_floor_does_not_run_the_figure_tool。** git だけを置いた PATH（Node が無い）で、図を持つ写しの床は 0。同じ PATH で rail 8 段の写しは 1 で `[面]`。
5. **f187_contract_rows_without_verify_and_done_build（向き (b)）。** 土台の設計ノート example の契約表の行 a から verify と done を外すと、床は合格 0 / 0 のまま、`folio face --face note --id example` が 0 で書け、行 a の記事に done の段落も検証の札も無い。
6. **f187_the_face_stage_runs_after_the_index_and_before_the_freeze（段の順）。** 要件の id FR1 を一重の引用符にした（索引の節点の違反）写しに道具 0 を足すと、`folio check` も `folio build --write` も違反 1 件のまま（`[索引の節点]`・`[面]` なし）。発効して封の無い判断の記録（土台の ADR-4 の字から id だけ替える）に道具 0 を足した写しで `folio check --freeze-adrs` は 1 で `[面]` を出し、封の file は byte で同じ。道具 0 を戻すと同じ口が 0 で封を足す（写しが凍結の道を通ることの対）。
7. **f187_unreadable_face_files_stay_unknown（読めないの 8 行）。** 支度表 `intake-sheet.yaml` に YAML の構文の誤り・支度表を dir に・支度表に UTF-8 でない byte・支度表に `zz: 0o17`（型付きの木に読めない）・`preview/folio.css` を dir に・`preview/folio-ui.js` を dir に・`preview/ceiling-stamp.yaml` を dir に・天井の印に YAML の構文の誤り（parse できない）。各行で `folio check` が 2 で `# まだ分からない:` の行に面の字（`…: 読めない: …`）と「まだ分からない（違反 0・まだ分からない 1）」を出し `[面]` なし、`folio build --write` が 2 で「床 = まだ分からない（違反 0・まだ分からない 1）」を出し配信先を作らない。
8. **単体 f187_memo_reads_a_file_once_inside_and_every_time_outside（`site.rs` の tests の区間）。** 閉包の中では file を消した後の 2 度目の `load` が 1 度目と同じ木、YAML として読めない file の Err の字 1 つだけを memo が読めないの一覧で返す、外では Err。
9. **単体 f187_dry_render_checks_the_shape_without_the_tool（同じ区間）。** 図の口の中では道具の置き場の無い dir でも空の本体、型が表に無い・記述が表でないは今どおり Err、外に出ると空の本体を返さない（親に空の道具の dir を置いた一時 dir で撃つ＝焼いた道具を書き出さず Node も撃たずに「図の道具が無い」・一時 dir は消す）。
10. **既存の歯 `tests/help.rs` の f192_check_help_names_what_the_floor_counts（便 192）。** 期待の字の「索引が組めるか」を「索引と面が組めるか」にし、冒頭の注の 2. に面を足す（席の割り・本便で期待の字を変える既存の歯はこの 1 本だけ）。
11. **RED の実測。** 歯の file だけ（見本 ac9ca7b の字）を base 44c14ac に当てると、binary の 7 本とも落ちる（base の床は割れの写し・読めない写し・凍結の写しで合格のまま＝rc 0、契約表の行の写しは面の口が「欄 done が無い」で 2・nextest の rc 100・起草の記録の logs/d/red.log）。base の `--bin folio f187_` は 0 本（nextest の rc 4）。

### (d) 採らなかった形

1. **1 行ずつ床に写す（切り出した純な関数を行ごとに床から呼ぶ・f2-648.245 の調べの推奨の形）。** 形の 932 か所に対して床の写しが要り、読み手が 2 つのまま（P-6.3）、写し漏れが新しい割れになる（3. の 863 のうち台帳に挙がっていたのは 12 行の型だけ）。面の関数そのものを呼べば全数が 1 段で閉じる。
2. **測れない（pendings）が在る置き場でも面の段を回す。** 違反にすると、要件書 FR25・AC22 が まだ分からない と決めた場合（組み立てた版の外の値域）を面が同じ原因で 不合格 に変える（既存の歯 `floor_cases_all_pass_with_folio` の schema-amendment-outside-built-range と `f157_narrowed_range_is_pending_and_never_pass` が落ちた）。まだ分からない の 1 件として足す形は、同じ原因を 2 度数え、骨格の床の数を固定した既存の歯（`f165_a_present_field_in_a_wrong_shape_still_stops` ほか）の字を変える。どちらも取らず、骨格の置き場（測れない 6）では面の欠けを名指さない＝(i) の 2。
3. **床が図の道具も撃つ。** 床が Node と道具に依り、Node の無い端末で床が まだ分からない になる。道具の答えは build の段で知らせる（FR15）。
4. **図を撃たない旗を面の関数の引数で渡す。** 5 つの面の生成器と図の章の関数の引数が全部変わり write-set が面の 12 file に広がる。thread_local の旗を閉包のあいだだけ立てる形にした。
5. **面の段で様式 2 本を読まない（読めないを床が見ない）。** 様式の file が読めない置き場を床が名指さなくなり、build だけが まだ分からない で知らせる（床と build の割れが 1 つ残る）。読んで、読めないは まだ分からない に数える形にした。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本の写しで workspace の nextest **1079 / 1079**（base 44c14ac の 1070 + f187_ の 9 本・430c9c7 の木・ac9ca7b は歯の表の行だけで本数は同じ）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・verify の 9 行とも rc 0（430c9c7 は起草の記録の logs/c・ac9ca7b は logs/d）。字の期待を直した既存の歯は (c) の 10 の 1 本だけ。
2. **突然変異（見本 430c9c7 の src だけを 1 通りずつ変え、f187_ の 9 本を撃つ）。** 24 通りとも落ちる（生き残り 0・起草の記録の logs/c/mut.log と mut2.log）。検証役の変異のうち読めないの口を 1 か所ずつ外す 5 通り（支度表の fs の読み・UTF-8・型付きの読み、印の parse、様式 folio-ui.js）も、ac9ca7b の (c) 7 の各行が落とす（logs/d/mut-a.log）。check が面の段を呼ばない・--write の床が呼ばない・測れない が在っても回す・違反が在っても回す・違反でなく まだ分からない に数える・字を面の字から変える・床で図の道具を撃つ・図の口が形を確かめる前に返る・図の口を出ても旗を戻さない・読み直さない口を出ても写しを返す・何も覚えない・done を要る欄に戻す・verify を要る欄に戻す・面の段が判断の記録と設計ノートの面を組まない・check で面の段を索引の口の前に（検証役の V4）・凍結の後始末の後に（V5）・--write の床で索引の口の前に（V6）・読めないも違反に数える・面の字を全部 まだ分からない に数える・正本の YAML の読めないを覚えない・様式の読めないを覚えない・天井の印の読めないを覚えない・重複キーも読めないに数える・memo が読めないの字を返さない。
3. **外の置き場と folio2 自身（見本の binary）。** folio2 の本流 44c14ac・床の土台・tsuzuri の写し（db212b1・2026-09-28T20:06 の HEAD・検証役の places-a.log）とも、`folio check` の字と rc が base と byte で同じ（どれも合格 0 / 0・新しく落ちる正本 0）、`folio build --write` の出力は base と全 file が byte で同じ（37・17・56 file・places-187.sh と検証役の places-a.sh）。突然変異の写しでは、床の土台の割れ 808 のうち 805 は床が面の字で 違反 1 に落とし、残る 3 は (b) の 5. の行で面が通るようになる（割れでなくなる）。追加の 2 file の割れ 55 は 55 とも落とす。割れでない写し 7798 と 438 は床の答え（3 値・違反の数・まだ分からない の数）が 1 件も動かない。骨格（git の 1 commit・測れない 6）4340 は床の答えが 1 件も動かない（突然変異の写しの数は 6467915 の見本 94ee033 の測り・改訂 a で読めないの字が まだ分からない に替わるのは読めない写しだけで、突然変異の写しはどれも読める）。
4. **時間。** folio2 の本流 6467915 の `folio check` は release で 0.49 秒 → 1.29 秒・debug で 1.02 秒 → 4.72 秒（面を 37 枚組む）。debug の `folio build --write` は 18.2 秒 → 21.6 秒（床の段の分）で、出力の 37 file は byte で同じ。本流 44c14ac の木の debug の check（負荷のある時の 1 回）は folio2 1.31 秒 → 4.59 秒・床の土台 0.25 秒 → 0.44 秒・tsuzuri の写し（548fff4）0.73 秒 → 2.04 秒（places.log）。workspace の nextest の所要は base と同じ幅（8 分前後）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file は `+crates/folio/tests/floor_faces.rs`。`crates/folio/tests/help.rs` は期待の字を 1 つ変える（(c) の 10）。`crates/folio/tests/site.rs`・`crates/folio/tests/note.rs`・`crates/folio/tests/face_note.rs`・`crates/folio/tests/outside_faces.rs`・`crates/folio/tests/floor_cases.rs` は本文を変えない（verify の `--test` の scope）。差分は 8 file・+899 −22・50,950 byte（`git diff 44c14ac ac9ca7b | wc -c`）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 6 本（base 44c14ac と見本 430c9c7〔ac9ca7b も src は同じ〕・python と awk の 2 実装で一致・起草の記録の logs/c/cap.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/site.rs` | 274 | 1226 | 347（+73） | 1153 |
| `crates/folio/src/cursor.rs` | 202 | 1298 | 250（+48） | 1250 |
| `crates/folio/src/figure.rs` | 801 | 699 | 819（+18） | 681 |
| `crates/folio/src/face_note.rs` | 1034 | 466 | 1037（+3） | 463 |
| `crates/folio/src/main.rs` | 703 | 797 | 705（+2） | 795 |
| `crates/folio/src/stamp.rs` | 504 | 996 | 506（+2） | 994 |

3. **size は M。** src の増分は +146（うち単体の歯 45）で S の見積 100 を超え、M の見積 300 の内（14bca08 の時点は +110・site.rs 331）。余地の最小（face_note.rs の base 466・本便の後 463）は M の 300 を超える。
4. **verify は 9 行**で、done の 9 つの塊と 1 対 1 に揃える。見本の写しで 9 行とも rc 0。
   1. `cargo nextest run -p folio --test floor_faces f187_` = (c) の 1〜7（7 本）。
   2. `cargo nextest run -p folio --bin folio f187_` = (c) の 8・9（2 本）。
   3. `cargo nextest run -p folio --test site` = build の床の行と書かない道（面の段を足しても base と同じ）。
   4. `cargo nextest run -p folio --test note` = 設計ノートの床（conditional の欄の対・導出 file の読めない場合）。
   5. `cargo nextest run -p folio --test face_note` = 設計ノートの面（契約表の行の出し方）。
   6. `cargo nextest run -p folio --test outside_faces` = 骨格の置き場の面の寛容さと、在るのに形が違う欄の止まり（骨格の床は変わらない）。
   7. `cargo nextest run -p folio --test floor_cases` = 床の 146 場面の答え（FR25 の まだ分からない を含む）。
   8. `cargo nextest run -p folio --test help` = 命令の説明の字（check の説明に 面が組めるか・(c) の 10）。
   9. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は全部 write-set に在る。`--bin folio` の歯の在り処（site.rs）も write-set に在る。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/floorgap の本契約で 契約に起因する断り 0（preflight ok・`f187_` は base で 0 件）。
3. **着地の後。** 席は tsuzuri へ「`folio check` が面の生成器と同じ関数で面を組む（面が組めない置き場は種類 面 の違反・字は build の まだ分からない の字と同じ・図の道具は撃たない）。tsuzuri の今の置き場（db212b1）の床は変わらない」を返す。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/floorgap-draft.md`、script と log は同じ dir の floorgap-scripts。

1. 全数: sites.py（面の経路の Err の出どころを file・行・関数・字の型・種別で拾う・sites-6467915.tsv）。
2. 割れ: sweep.py（mutants → run〔base は build・見本は check〕→ split → compare）・kinds.py（file と字の型の表）・cases.py（(c) の写しを base と見本で撃つ）。base の測りは base に図の道具を撃たない環境の値を足した測りだけの binary（契約の外）。
3. RED: red-187.sh。突然変異: mut-187.py。余地: cap.sh（lines.py / lines.awk）。順に撃つ: chain-187.sh（改訂 a の 44c14ac の上は chain-187b.sh・log は logs/c）。3 か所の床と build: places-187.sh。読めないの見立て: probe-b1.sh。帰結の項の写像の見立て: probe-cons.sh。tsuzuri の写し: prep-tz.sh（.git だけを写して clone・remote を外す）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 台帳 f2-648.180 の (1)(2)（条の中の行 rationale・retreat・amended_by と最上位の節 north_star・amendment・sources の行の未知の欄）: 面は止まらないので本便の段では閉じない。憲法の schema 節はこれらの行の欄の集合を持たず（article・mechanism・statement・meta・precedence だけ）、床に一覧を持たせるには schema 節を変える（A-2.2 で改訂）か実装の定数を置く（P-5.6 の写しの置き場が要る）ので、席へ返す。図の道具と環境（道具の置き場・Node・凍結 anchor・道具が記述を通さない・配信先）は床の外のまま（build が まだ分からない で知らせる・FR15）。様式 2 本・天井の印・正本の読めないは床が まだ分からない で名指す（違反にしない・(b) の 1）。役の字「道具」の設計文書への写し（P-5.6・f2-648.249 の後半）は生成区間を書くので別の便（門と tsuzuri の `folio schema --write` が要る）。面の要る形を人が読む欄の決まりへ写すこと。設計文書の正本・憲法と要件書と判断の記録の字・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** (1) 床が まだ分からない（測れない が在る）置き場では、面が止まっても床は面の字を名指さない（骨格の写しで 560 件・答えは base と同じ まだ分からない）。(2) 面の段は最初の 1 つの Err だけを名指す（build と同じ）。(3) 突然変異は欄の道ごとに一覧の最初の項だけを変えた（全数は code の側の 1019 が持つ）。(4) 道具が型付き記述を通さない図は、床が合格でも build が止まる（道具の答え・folio2 自身の spec の写しで 692 通り・(a) の 3）。(5) 印の欄の欠け（字が 読めない でも、例 sources が読めない）は形の誤りとして違反 [面] に数える。(6) 天井の印の要約値を測る読み（天井の正本が名指す file）は、床のほかの段が同じ file を先に読むので面の段まで届かない（届けば今どおり違反の字）。
3. **撤退条件。** (1) 本便が要件の規範文か憲法か判断の記録の字を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `site.rs` の `build_all` か `write_after_floor`、`main.rs` の check の段の順か check の説明の字、`figure.rs` の `render` が base と違えば、止めて席へ返す。(3) 本便の後に既存の歯が落ちたら（(c) の 10 の期待の字を除く）、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果か `folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: 床の面の段（`site.rs` の口と種類の定数・`main.rs` と `--write` の床からの呼び出し）・図を撃たない口（`figure.rs`）・読み直さない口と読めないの口（`cursor.rs`・様式と天井の印〔`stamp.rs`〕と器の導出 file の読みもこの口を通す）・契約表の行の verify と done を任意に（`face_note.rs`）・check の説明の字に 面が組めるか（`main.rs`・席の割り）・歯の file の f187_ の 7 本と単体の 2 本と、`tests/help.rs` の期待の字 1 つ。
- 入れない: 床のほかの段の判定・面の字と出力・図の道具を撃つ道・設計文書の正本と生成区間・憲法と要件書と判断の記録の字・台帳への記帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| stage | 床の面の段 | `site.rs` の口（ほかの段が黙っているときだけ build と同じ関数で面を組み、Err を種類 面 の違反に・読めないは まだ分からない に） |
| dry | 図を撃たない口 | `figure.rs`（型と記述の形までを確かめ、道具・Node・凍結 anchor を撃たない） |
| memo | 読み直さない口と読めないの口 | `cursor.rs`（閉包のあいだ同じ正本の木を 1 度だけ読み、読めないの字を覚えて返す・`stamp.rs` の印の読みも通す） |
| cond | 契約表の 2 欄 | `face_note.rs`（verify と done を在るときだけ出す） |
| teeth | 歯 | `tests/floor_faces.rs` の f187_ の 7 本と `site.rs` の単体の 2 本（`tests/help.rs` の期待の字 1 つ） |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 185（本流 6467915）と便 192〜194（本流 44c14ac）。
- 本便の着地の後に席が見ること: 台帳 f2-648.249・.274・.242 の件を閉じるか残りを書く（役の字の写し）。.180 は (3) が閉じ、(1)(2) が残る。本流の `target/debug/folio` を組み直す。tsuzuri へ床の面の段を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gh"
title = "床が build と同じ関数で面を組み、面が組めない置き場を面の字のまま違反に数える（床の穴の束・台帳 f2-648.249・.274・.242）: crates/folio/src/site.rs に床の口と違反の種類 面 を足し、床のほかの段が違反も読めないも測れないも数えていないときだけ、build と同じ build_all を crates/folio/src/figure.rs の図を撃たない口（型と型付き記述の形までを確かめ、道具・Node・凍結 anchor を撃たない）と crates/folio/src/cursor.rs の読み直さない口（閉包のあいだ同じ正本の木を 1 度だけ読む）の中で回し、Err を面の字のまま違反 1 件に数える（面の字が読めないの口を通った字〔正本・様式・天井の印・器の導出 file の読めない〕を含むなら違反でなく まだ分からない 1 件）。crates/folio/src/main.rs の folio check と site.rs の --write の床が索引の口の後・凍結の後始末の前でこれを呼ぶ。crates/folio/src/face_note.rs は契約表の行の verify と done を在るときだけ出す（器の導出 file で conditional）。crates/folio/src/stamp.rs は印の読めないを同じ口に通す（字は今どおり）。main.rs の check の説明の字に 面が組めるか を足し、crates/folio/tests/help.rs の f192_check_help_names_what_the_floor_counts の期待の字を合わせる（便 192 の契約の席の割り）。床のほかの段の判定と字・面の字と出力・図の道具を撃つ build の道・設計文書の正本は変えない。歯は crates/folio/tests/floor_faces.rs の f187_ の 7 本と site.rs の単体の 2 本。実装の見本は origin の枝 impl/d187 の commit ac9ca7b で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 44c14ac"
req = ["FR5", "FR10"]
section = "1"
write-set = ["crates/folio/src/site.rs", "crates/folio/src/main.rs", "crates/folio/src/figure.rs", "crates/folio/src/cursor.rs", "crates/folio/src/face_note.rs", "crates/folio/src/stamp.rs", "+crates/folio/tests/floor_faces.rs", "crates/folio/tests/help.rs", "crates/folio/tests/site.rs", "crates/folio/tests/note.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/outside_faces.rs", "crates/folio/tests/floor_cases.rs"]
verify = ["cargo nextest run -p folio --test floor_faces f187_", "cargo nextest run -p folio --bin folio f187_", "cargo nextest run -p folio --test site", "cargo nextest run -p folio --test note", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --test outside_faces", "cargo nextest run -p folio --test floor_cases", "cargo nextest run -p folio --test help", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "crates/folio/tests/floor_faces.rs の f187_ の 7 本（要件書の 10 行と入口・憲法・判断の記録・設計ノート・支度表の 7 行の割れの写しで、床が 1 で [面] の行に面の字・build --write が 1 で書かない・folio face が 2 で同じ字／ほかの段が数えているときは面の段を回さない／Node の無い PATH で図を持つ写しの床が 0／契約表の行の verify と done が無くても床は合格で設計ノートの面が書ける／索引の違反が在れば面の段を回さず、面が組めなければ --freeze-adrs が封を足さない／面だけが読む file の読めない 8 行で床と build --write が 2 で まだ分からない 1 件・[面] なし）が緑、binary の単体の f187_ の 2 本（読み直さない口と読めないの一覧と、図を撃たない口）が緑、tests/site.rs の歯の全部が緑、tests/note.rs の歯の全部が緑、tests/face_note.rs の歯の全部が緑、tests/outside_faces.rs の歯の全部が緑、tests/floor_cases.rs の歯の全部が緑、tests/help.rs の歯の全部が緑（check の説明の 1 行目が 索引と面が組めるか を名指す）、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数と中身は着地の直前の main と同じである"
<!-- contracts:end -->
