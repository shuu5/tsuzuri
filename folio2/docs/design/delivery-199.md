# 設計: 便 199 — 網の外の関数の中の突き合わせの字をつながりに数える（書く前の口が止めない族を広げる・判断の記録 ADR-33 の便 2）

- 要件: FR28（編集時の口・要件書 第 1.55 版・発効済み）と受入基準 AC31。規範文・確かめ方・受入基準は変えない。条 P-18.1（編集の時点で止める）の「食い違う」を形と凍結への食い違いと読み、まだ揃っていないつながりは事後の床が数える（ADR-33 決定 (2)・条 P-18.2）。
- 条: P-18.1・P-18.2（事後の床は今までどおり数える）・P-15.2（口と素の床は同じ関数・止めた行は素の床の行）・P-4.1（まだ分からない は止める側）・P-6.3（床の関数を 2 つにしない）。
- 出所: 判断の記録 ADR-33（accepted・持ち主の承認 2026-09-28 22:08 JST・本流 08a64bc で発効）の決定 (2)(7)。決定 (7) の「便 199（行 gt）が網の外の関数の中の突き合わせの字の族（設計ノート・計画・判断の記録・封・入口・相談窓口・天井）を運ぶ」。便 198（行 gs・本流 ca8640a）の着地の後に書く（余地を数え直す）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gt` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 12 本（src 8・歯の file 4〔本文を変える 2・本文不変 2〕）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流 5b8aa9a の組み立ての binary に write-set を渡した `folio ceiling --gate --dir design-intent --write-set …` は **0（通す・設計文書の正本を書き換えない便）**（起草の記録の `gate-19x.log`）。
- 前提: **base = 本流 5b8aa9a**（便 198 の着地 ca8640a・行 R-20・便 196 の着地の後）。この契約の数はすべて 5b8aa9a の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d199`（commit **85b6d84**・本流 49c4779 の上の c8048e0 に本流 5b8aa9a〔便 196 の着地〕を取り込んだ merge 5328526〔衝突なし〕の上に、検証役の Bb199-1・Bb199-2 の直しを積んだ commit）。`git diff 5b8aa9a 85b6d84` が便の全体の差分（10 file・+254 −39・29,028 byte）。**作業者は write-set の file をこの枝の中身にしてよい**（`git checkout impl/d199 -- <write-set の file>`）。
- 並行の便との重なり: §1 (g) の表。本便の write-set の file を書き換える便が先に着地したら、本便は余地と歯の数を数え直してから受け付ける。

## 1. 設計

### (a) いま起きていること（base 5b8aa9a の実測・参考値）

1. **書く前の口は、網の外の関数の中の突き合わせの字を止める。** 便 198 の口（`folio check --dir <置き場> --proposed <相対の file>`）は、後にだけ在る違反のうち、網の関数 5 つ（参照 id の解決と逆参照と件数・語彙・判断の記録と正本の突き合わせ・凍結 anchor の列と改訂の記録・散文の言及）の違反だけをつながりに数え、ほかの違反は全部止める。網の外の関数の中にも 2 か所を突き合わせる字が 15 か所在り、2 つの file を順に書く途中で必ず一度食い違うのに、口はそれで編集を止める。
2. **16 か所（関数と字の頭）。**
   - `crates/folio/src/note.rs`: 関数 check_meta の meta の supersedes と superseded_by の相手（字 `meta.<欄>「<id>」の設計ノートが実在しない`・相手が自分自身の字は除く＝(a) の 3）・関数 check_section の散文の参照 id（字 `§<n>: 散文の参照 id「<id>」が実在しない`）・関数 resolve_ids の契約表と節の参照 id（字 `<場所>: id「<id>」が実在しない`）。
   - `crates/folio/src/adr.rs`: 関数 check_between の前任と後継の実在（字 `<欄> <id> の判断の記録が実在しない`）・後継の双方向（字 `後継 <id> の supersedes に <id> が無い（双方向）`）・置き換えた記録の双方向（字 `置き換えた <id> の superseded_by が <id> でない（双方向）`・base の口は、新しい記録を先に書く順〔supersedes だけを書いた周〕でもこれで止める）・retired の後継の列の輪（字 `retired の後継の列が輪になっている（…）`）の 3 つは相手が自分自身の字を除く（(a) の 3）、関数 check_decided_by の欄の決まりの出所（字 `adr/schema.yaml meta.decided_by の <id> が実在しない（欄の決まりの出所の判断が消えている）`）。
   - `crates/folio/src/seal.rs`: 関数 check_seals の発効した判断の記録の封の欠け（字 `<id>: 発効しているのに封の行が無い（…）`）。
   - `crates/folio/src/entrance.rs`・`intake.rs`・`ceiling.rs`: 関数 check_entrance・check_intake・check_ceiling の英字の語と語彙の突き合わせ（字 `語彙に無い英字の語「<語>」`）。
   - `crates/folio/src/plan.rs`: 関数 check_plan の節の型 row-index と row-plan の置き場（規則の表の名札の行との突き合わせ）・行の索引の生成区間の中身と契約表からの導出の違い・行の索引に在る計画だけの行・宙に浮いた依存。
3. **同じ関数の中の 1 つの file の形の字。** 自分自身を指す字（判断の記録の superseded_by か supersedes が自分の id＝後継の列の輪の長さ 1・設計ノートの meta.supersedes か superseded_by が自分の文書 id）・設計ノートと判断の記録の retired に後継が無い・承認欄が無い・節の型の欄が無い・計画だけの行の id が 2 度在る・計画のノートの行の索引の節の数・生成区間の印の崩れ・印が節の rows の外に在る は、1 つの file だけで決まる形で、ADR-33 決定 (2) の「それ以外（file の形・…）は止める」に入る。計画の床の関数 drift（`plan.rs`・床と `folio derive --check` が同じ関数で比べる・条 P-15.2）は、形の理由 3 つと中身の違い 1 つを同じ字の列で返す。
4. **外の置き場（tsuzuri の写し f71082f・参考値）。** base の binary で、設計ノート surface-wavei.yaml の §2 の行 i-1 の req に FR999 を足した中身を口に渡すと終了 1（止める行 `[note] design-note/surface-wavei.yaml: §2 の行 i-1 の req[0]: id「FR999」が実在しない`）。
5. **base の歯（参考値）。** workspace の nextest は §1 (e) の表・clippy 0 警告・床 4 本 rc 0・`folio build --write` は 39 file。`git grep -n f199_ -- crates` は 0 件・行 id `gt` は 0 件。

### (b) 直す先

1. **`crates/folio/src/verdict.rs`。** `Report` に関数 link（新しい名・引数は種類と字）を足す。違反を 1 つ積み、その添字を欄 links に積む（便 198 の関数 links_from と同じ印）。字・数・判定・素の床の出力は変えない。
2. **16 か所の積み方を関数 link に替える。** (a) の 2 の 16 か所で、違反を積む呼び出しを関数 link の呼び出しに替える（字と引数は 1 字も変えない）。`adr.rs` の双方向 2 つと輪の 1 つは、関数 one_or_link（新しい名・引数は Report と自分自身かの真偽と字）で積み、相手が自分自身なら止める違反、ほかはつながりにする（輪は後継の列の 1 歩目で自分に戻るとき＝長さ 1）。`note.rs` の meta の相手も、自分の文書 id なら止める違反のまま。
3. **`crates/folio/src/plan.rs` の中身の違いの理由を定数に分ける。** 関数 drift の理由のうち、行の索引の生成区間の中身が契約表からの導出と違う理由の字を定数 CONTENT_DRIFT（新しい名・字は今と同じ `行の索引の生成区間が契約表からの導出と違う（folio derive --write で書き直す）`）に置き、drift はその定数を返す。関数 check_plan は、drift の理由がその定数と同じならつながり（関数 link）、ほかの理由（節の数・印の崩れ・印が節の外）なら今までどおり止める違反に積む。drift の引数と戻りの型と `crates/folio/src/derive.rs` は変えない（derive --check の字と終了は同じ）。
4. **止めるまま（変えない所）。** 計画だけの行の id の重なり（`plan.rs`）・設計ノートの retired に後継が無い・承認欄が無い・節の型の欄が無い（`note.rs`）・判断の記録の retired に後継が無い・状態の値域・封の行が在るのに本文が違う（`adr.rs`・`seal.rs`）・判断の記録の「retired の後継の列の先 X が発効していない」（`adr.rs`・相手の記録の状態を読む突き合わせだが、後継を先に発効させる順で通り 2 つの記録が互いに止め合わない＝ADR-33 問い 1 の理由〔止め合い〕に当たらないので止めるまま・席の裁定 2026-09-29・検証役 N-19x-6）・英字の語の形の誤り以外の入口と相談窓口と天井の字。封の 2 つの字の分け方: `seal.rs` の「封の行が在るのに本文が違う」（発効した判断の記録の本文と封の一覧の照合）は、要件書 FR28 の括弧と ADR-33 決定 (2) がつながりから除く凍結なので止めるままにする。(a) の 2 の `seal.rs` の 1 か所「発効しているのに封の行が無い」は、それとは別の検査で、ADR-33 決定 (2) が網の外の突き合わせの字として名指す「発効した判断の記録の封の欠け」に当たるのでつながりに数える。決定 (6) は「判断の記録の発効と封」を 2 か所を順に書く双方向の決まりに挙げる（status を accepted に書いた後に `folio check --freeze-adrs` が封の行を足すので、その間は必ず一度欠ける）。FR28 の除く「本文と封の一覧の照合」は封の行が在るときの本文の一致を指し、封の行の欠けを含まない（決定 (2) の止める側の列も「発効した判断の記録の本文の封」と書き、欠けを名指さない）。
5. **変えないもの。** 素の `folio check` の出力・判定・終了コード（関数 link は violation と同じ字と数を積む）・口の出力の形と順・網の関数 5 つ・設計文書の正本と生成区間・憲法と要件書と判断の記録の字・`folio build` の出力・`folio derive` の出力。

### (c) 歯（f199_・base で 0 件・7 本）

binary 経由の歯は `crates/folio/tests/proposed.rs`（便 198 の写しの作り方と同じ・床の土台の写しを一時 dir の git の 1 commit にし、口の一時の作業場所を版管理の外に置く）と `crates/folio/tests/plan.rs`（便 183 の計画のノートの写し）。期待の字は歯の側の手書き。

1. **f199_cross_checks_outside_the_net_are_links（10 の中身）。** どの中身も、口は終了 0・止める行 0・つながりの行が期待の字の列とちょうど同じで、同じ中身を書いた置き場の素の床は終了 1 で標準出力に同じ字の行が在る。
   1. 設計ノート example.yaml の §6 の行 a の req に FR999 → `[note] design-note/example.yaml: §6 の行 a の req[1]: id「FR999」が実在しない`。
   2. example.yaml の meta に supersedes: nosuch → `[note] design-note/example.yaml: meta.supersedes「nosuch」の設計ノートが実在しない`。
   3. example.yaml の §1 の規範の印を持つ文の参照に P-99 → `[note] design-note/example.yaml: §1: 散文の参照 id「P-99」が実在しない`。
   4. 判断の記録の欄の決まり adr/schema.yaml の decided_by に ADR-99 → `[adr] adr/schema.yaml meta.decided_by の ADR-99 が実在しない（欄の決まりの出所の判断が消えている）`。
   5. 新しい判断の記録 ADR-11（ADR-10 の写し・proposed・supersedes: ADR-99）→ `[adr] ADR-11: supersedes ADR-99 の判断の記録が実在しない` と網の `[adr] ADR-11.supersedes: 判断の記録 ADR-99 が実在しない` の 2 行。
   6. 新しい ADR-11（retired・superseded_by: ADR-10）→ `[adr] ADR-11: 後継 ADR-10 の supersedes に ADR-11 が無い（双方向）` と `[adr] ADR-11: 発効しているのに封の行が無い（folio check --freeze-adrs で封を足し、commit する）` の 2 行。
   7. 新しい ADR-11（accepted）→ 封の欠けの 1 行。
   8. 入口 index.yaml の audience の short に英字の語 zqword → `[index] index.yaml audience の short: 語彙に無い英字の語「zqword」`。
   9. 相談窓口 intake.yaml の meta の title に zqword → `[intake] intake.yaml meta の title: 語彙に無い英字の語「zqword」`。
   10. 天井の正本 ceiling.yaml の meta の title に zqword → `[ceiling] ceiling.yaml meta の title: 語彙に無い英字の語「zqword」`。
2. **f199_retired_cycle_is_a_link。** 置き場に ADR-12（retired・supersedes と superseded_by が ADR-11）を先に置き、ADR-11（retired・supersedes と superseded_by が ADR-12）を口に渡すと、終了 0 で、つながりの行がちょうど ADR-11 と ADR-12 の輪の 2 行（`retired の後継の列が輪になっている（ADR-11→ADR-12→ADR-11）＝発効している後継が無い（P-7.2）` と逆向き）と ADR-11 の封の欠けの 1 行。素の床にその 3 行が在る。
3. **f199_single_file_shapes_stay_stops（3 の中身）。** どれも口は終了 1。(i) example.yaml の status を retired → 止める行がちょうど `meta: status retired なのに approval（承認欄）が無い` と `meta: retired なのに superseded_by（後継）が無い（P-7.2）` の 2 行・つながり 0。(ii) 新しい ADR-11（retired・後継なし）→ 止める行 `[adr] ADR-11: retired なのに superseded_by（後継）が無い（P-7.2）` の 1 行・つながりは封の欠けの 1 行。(iii) example.yaml の §1 の型を row-plan → 止める行が `§1: 節の型 row-plan の欄「rows」が無い` と `§6 の行 a: section「1」が同じ文書の prose の節の n でない` の 2 行・つながりは置き場の字 `§1: 節の型 row-plan は計画の名札の行（欄 key が plan-note の閾値の行）が名指す計画のノートにだけ置ける（名札の行が無い）` の 1 行。
4. **f199_plan_cross_checks_are_links_but_shapes_stop（5 の中身・`tests/plan.rs`）。** 計画のノート plan.yaml と名札の行を持つ写しで、(i) example.yaml の契約表に行 a2 を足す → 口は終了 0・つながりの行がちょうど `[note] design-note/plan.yaml: 行の索引の生成区間が契約表からの導出と違う（folio derive --write で書き直す）`。(ii) 計画だけの行 c の id を a（索引に在る）→ 終了 0・つながり `計画だけの行「a」が行の索引に在る（契約の行が在る＝計画だけの行の節から外す）`。(iii) 行 c の depends を zz → 終了 0・つながり `計画だけの行「c」の depends「zz」が行の索引にも計画だけの行にも無い（宙に浮いた依存）`。(iv) 行 c の id を b（計画だけの行に在る）→ 終了 1・止める行 `計画だけの行 id「b」が 2 度在る`。(v) 生成区間の end の印の行を消す → 終了 1・止める行 `行の索引の生成区間の印が 1 対でない（begin 1・end 0）`。5 つとも、同じ中身を書いた置き場の素の床の標準出力にその行が在る。
5. **f199_adr_pointing_at_itself_stays_a_stop（検証役の Bb199-1）。** 発効した ADR-10 を retired・superseded_by: ADR-10 にした中身を口に渡すと、終了 1・止める行がちょうど `[adr] ADR-10: 後継 ADR-10 の supersedes に ADR-10 が無い（双方向）` と `[adr] ADR-10: retired の後継の列が輪になっている（ADR-10→ADR-10）＝発効している後継が無い（P-7.2）` の 2 行・つながり 0。別の置き場で、新しい ADR-11（proposed・supersedes: ADR-11）は終了 1・止める行がちょうど `[adr] ADR-11: 置き換えた ADR-11 の superseded_by が ADR-11 でない（双方向）`・つながり 0。どちらも同じ中身を書いた置き場の素の床にその行が在る。
6. **f199_note_pointing_at_itself_stays_a_stop（検証役の Bb199-1）。** example.yaml の meta に supersedes: example を足した中身は、終了 1・止める行がちょうど `[note] design-note/example.yaml: meta.supersedes「example」の設計ノートが実在しない`・つながり 0。素の床にその行が在る。
7. **f199_superseding_before_the_old_record_is_a_link（検証役の Bb199-2）。** 新しい ADR-11（proposed・supersedes: ADR-10）を、置き換えた ADR-10 の superseded_by を書く前に渡すと、終了 0・つながりの行がちょうど `[adr] ADR-11: 置き換えた ADR-10 の superseded_by が ADR-11 でない（双方向）`（ADR-33 決定 (2)・持ち主への問い 1 の「両方向に書く決まりは止めない」）。素の床にその行が在る。
8. **RED の実測。** 歯の file だけ（`r199-teeth.patch`＝`tests/proposed.rs` と `tests/plan.rs` の差分）を base に当てると、f199_ の 7 本のうち 5 本（1〜4 と 7）が落ちる（base の口は 16 か所を止める＝つながりの行が無く終了 1）・`tests/plan.rs` の便 183 の 7 本は緑のまま。(c) の 5・6 は base でも緑（base の口はどの字も止める＝止めるままの向きを固定する歯で、変異 M22〜M25 が落とす）。log は起草の記録の `red-199d.log`。

### (d) 採らなかった形

1. **関数ごとに全部をつながりにする（16 か所でなく関数単位）。** 同じ関数の中の 1 つの file の形の字（(a) の 3）も止めなくなる。ADR-33 決定 (2) は file の形を止める側に置く。
2. **drift の戻りの型を分ける（形と中身の 2 種の型を返す）。** drift を撃つ `derive.rs` の 2 か所の書き換えが要り、write-set が広がる。理由の字の定数 1 つで分ければ drift の式は床と derive --check で 1 つのまま（条 P-15.2）。
3. **drift の全部をつながりにする。** 計画のノートの印の崩れ（1 つの file の形）を口が止めない。起草の初めの形で、検証の前に改めた（歯 (c) 4 の (v) と変異 M20）。
4. **自分自身を指す字もつながりにする（字の頭だけで分ける）。** 1 つの file の中で決まる形（ADR-33 決定 (2) の file の形）を口が止めない。起草の c8048e0 はそうなっていて、検証役の Bb199-1 で改めた（歯 (c) 5・6 と変異 M22〜M25）。
5. **口が字の頭で族を決める（床の関数を触らない）。** 字の頭と族の対応を口が別に持つことになり、床の字が変わると黙って族が変わる（条 P-6.4）。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 本便の差分を base に当てた写し（見本 85b6d84）の数は次の表（参考値・起草の記録の `verify-199d-summary.txt`・`CARGO_BUILD_JOBS=4`・`--test-threads 2`）。字の期待を直した既存の歯は無い。

| 数え | base 5b8aa9a | 見本 85b6d84 |
| --- | ---: | ---: |
| workspace の nextest（全部合格） | 1131 | 1138（+7 の歯） |
| clippy の警告 | 0 | 0 |
| 床 4 本（check・inject --check・schema --check・derive --check） | 4 本とも rc 0 | 4 本とも rc 0 |
| `folio build --write` の file 数 | 39 | 39（base と全 file が byte で同じ） |

2. **突然変異（見本の写しの src だけを 1 通りずつ変え、f199_ の歯を撃つ・`mut-199.py`・歯の番号は (c)）。** M1〜M15 は (a) の 2 の 15 か所を 1 か所ずつ止める側（violation）に戻す。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 設計ノートの別のノートの実在をつながりに数えない | 1 |
| M2 設計ノートの散文の参照 id をつながりに数えない | 1 |
| M3 設計ノートの参照 id の解決（resolve_ids）をつながりに数えない | 1 |
| M4 判断の記録の前任・後継の実在をつながりに数えない | 1 |
| M5 判断の記録の後継の双方向をつながりに数えない | 1 |
| M6 判断の記録の retired の後継の輪をつながりに数えない | 2 |
| M7 欄の決まりの出所の実在（decided_by）をつながりに数えない | 1 |
| M8 発効した判断の記録の封の欠けをつながりに数えない | 1・2・3 |
| M9 入口の英字の語をつながりに数えない | 1 |
| M10 相談窓口の英字の語をつながりに数えない | 1 |
| M11 天井の英字の語をつながりに数えない | 1 |
| M12 計画の節の型の置き場をつながりに数えない | 3 |
| M13 計画の行の索引の中身と契約表からの導出の違い（drift の中身の理由）をつながりに数えない | 4 |
| M14 計画だけの行が行の索引に在るをつながりに数えない | 4 |
| M15 計画だけの行の宙に浮いた依存をつながりに数えない | 4 |
| M16 Report::link がつながりの印を付けない | 1・2・3・4・7 |
| M17 Report::link の印が 1 つずれる（次の違反を指す） | 1・2・3・4・7 |
| M18 計画だけの行の id の重なりもつながりに数える | 4 |
| M19 設計ノートの retired に後継が無いもつながりに数える | 3 |
| M20 計画のノートの生成区間の印の崩れ（1 つの file の形）もつながりに数える | 4 |
| M21 置き換えた記録の superseded_by の双方向をつながりに数えない（Bb199-2） | 7 |
| M22 後継の supersedes の双方向で自分自身を指す字もつながりに数える（Bb199-1） | 5 |
| M23 後継の列の輪の長さ 1 もつながりに数える（Bb199-1） | 5 |
| M24 置き換えた記録の双方向で自分自身を指す字もつながりに数える（Bb199-1） | 5 |
| M25 設計ノートの meta が自分自身を指す字もつながりに数える（Bb199-1） | 6 |

   25 通りとも落ちる（生き残り 0・`mut-199d.log`・直列・85b6d84）。M16・M17 は関数 link の印を外す・ずらす。M18〜M20 と M22〜M25 は 1 つの file の形の字をつながりに数える向き（止めるままであることを落とす）。M20 は起草の中で足した（(d) の 3）。M21〜M25 は検証役の Bb199-1・Bb199-2 の直しで足した（(d) の 4）。

3. **外の置き場（tsuzuri の写し f71082f・参考値・`tz-19x.log`）。** 見本の binary で (a) の 4 の中身は終了 0・要約 `通す（新しい違反 0・つながり 1・…）` とつながりの行 `[note] design-note/surface-wavei.yaml: §2 の行 i-1 の req[0]: id「FR999」が実在しない`。素の床は base と見本で同じ（便 196 の着地の後の base では tsuzuri の設計ノートの欄の決まりの写しが古く、どちらも `[note] design-note/schema.yaml: 床の定数と違う: schema.contract_table.external_schema.known_values（欠落）` の違反 1 で終了 1＝tsuzuri の `folio schema --write` の手番・本便に依らない）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file は無い。`crates/folio/tests/check.rs` と `crates/folio/tests/adr.rs` は本文を変えない（verify の `--test` の scope）。差分 29,028 byte（`git diff 5b8aa9a 85b6d84 | wc -c`・10 file・+254 −39）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 8 本（python と awk の 2 実装で一致・`cap-19x.sh`・`cap-199d.log`）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/note.rs` | 1002 | 498 | 1005（+3） | 495 |
| `crates/folio/src/adr.rs` | 972 | 528 | 969（−3） | 531 |
| `crates/folio/src/ceiling.rs` | 576 | 924 | 576（±0） | 924 |
| `crates/folio/src/entrance.rs` | 378 | 1122 | 378（±0） | 1122 |
| `crates/folio/src/seal.rs` | 316 | 1184 | 316（±0） | 1184 |
| `crates/folio/src/intake.rs` | 290 | 1210 | 290（±0） | 1210 |
| `crates/folio/src/plan.rs` | 274 | 1226 | 279（+5） | 1221 |
| `crates/folio/src/verdict.rs` | 107 | 1393 | 113（+6） | 1387 |

3. **size は S。** src の増分は +11 で S の見積 100 の内。余地の最小（note.rs の base 498・本便の後 495）は S の 100 を超える。
4. **verify は 5 行**で、done の 5 つの塊と 1 対 1 に揃える。見本の写しで 5 行とも rc 0。
   1. `cargo nextest run -p folio --test proposed` = (c) の 1〜3 と 5〜7（f199_ の 6 本）と便 198 の f198_ の 19 本（口の形と順は変わらない・25 本）。
   2. `cargo nextest run -p folio --test plan` = (c) の 4（f199_ の 1 本）と便 183 の f183_ の 7 本（計画の床の字と derive --check は変わらない・8 本）。
   3. `cargo nextest run -p folio --test check` = 素の `folio check` の歯（出力と終了が同じ・58 本）。
   4. `cargo nextest run -p folio --test adr` = 判断の記録の床の歯（字と数が同じ・13 本）。
   5. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（proposed・plan・check・adr）は全部 write-set に在る（check と adr は本文不変）。

### (g) 門・受付・ほかのレーンとの重なり

1. **門。** 冒頭のとおり対象外・0（通す）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/guard2 の本契約で 契約に起因する断り 0（起草の記録の `precheck-199.log`）。ADR-33 と要件書 第 1.55 版は発効済み（req の FR28 は本流に在る）。
3. **重なり（2026-09-29 02:4x と 03:0x の実測・origin の各レーンの見本の枝と本便の見本を git merge-tree で重ねた）。**

| レーン（便・見本） | 重なる file | 扱い |
| --- | --- | --- |
| 写しの負債（196・本流 5b8aa9a） | 無し（196 は check.rs・floor_note.rs・ids.rs・tests/graph.rs・tests/schema.rs・tests/schema_docs.rs と設計文書） | 着地済み（base に含む・取り込みの merge は衝突なし） |
| 便 200（impl/d200 a250799） | 無し | 無し（衝突なし・どちらが先でも数は変わらない） |
| 便 201（impl/d201 8dbdd4d） | 無し | 無し（衝突なし） |

4. **着地の後。** 席は器の席へ、器の行（口を撃つ器の hook の行）を頼める（ADR-33 決定 (7)「器の行は便 199 の着地の後に器の席へ頼む」）。本便の後、口が止めるのは形と凍結と索引の節点と まだ分からない だけになる。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/guard-draft.md`、script と log は同じ dir の `guard-scripts/`。

1. 模擬: 見本 impl/d199 の 85b6d84（`git diff 5b8aa9a 85b6d84` = `c199.patch`・歯だけ = `r199-teeth.patch`）。`verify-198.sh <見本> <base> <log>`（組み立て・nextest・clippy・床 4 本・build の出力を base と byte で比べる）。
2. RED: `red-19x.sh <base> r199-teeth.patch <log> proposed f199_ plan -`。突然変異: `mut-199.py`（M1〜M25・直列）。余地: `cap-19x.sh <base> <見本> <src の file 名…>`（`lines.py` と `lines.awk`）。資源は `CARGO_BUILD_JOBS=4`・nextest `--test-threads 2`。
3. 外の置き場: tsuzuri の `.git` だけを写して clone・remote を外した写しで、base と見本の binary を撃つ（`tz-19x.sh`）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 極性一覧の口と下限の数え（便 200）・器の hook の行（器の判断の記録と便）・網の関数 5 つ・床の字の変更・設計文書の正本。
2. **言えないこと。** (1) 族は積む所ごとに決まる。これから 16 か所の関数に突き合わせの字を足す便が関数 link でなく violation で積めば、その字は止める側に残る（閉じる側に倒れる）。(2) 口の 通す は、その編集で増える違反が止める族に無いことだけを言い、置き場の床の 合格 ではない（条 P-3.3）。(3) 判断の記録の網（`link.rs`）と `adr.rs` の関数 check_between は、同じ相手の不在を 2 行で名指す（(c) 1 の 5）。
3. **撤退条件。** (1) 本便が要件書 FR28 か ADR-33 の字か憲法の条文を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で (a) の 2 の 16 か所の字か関数 drift の理由の字が base（5b8aa9a）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 見本の口がつながりに名指した行が同じ中身を書いた置き場の素の床に無い周が 1 つでも見つかったら、止めて席へ返す（ADR-33 撤退条件 (2) と同じ向き）。

## 2. 範囲

- 入れる: `verdict.rs` の関数 link・16 か所の積み方の置き換え（`adr.rs` の関数 one_or_link と `note.rs` の自分自身の分け）・`plan.rs` の定数 CONTENT_DRIFT と check_plan の理由の分け・歯の file の f199_ の 7 本（`tests/proposed.rs` 6・`tests/plan.rs` 1）。
- 入れない: 極性一覧・器の hook・網の関数・違反の字・`derive.rs`・規則の表と憲法と要件書と判断の記録の字・設計文書の正本と生成区間・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| link | つながりの積み方 | `verdict.rs` の関数 link（違反を積み、印を付ける） |
| sites | 網の外の突き合わせの字 | `note.rs`・`adr.rs`・`seal.rs`・`entrance.rs`・`intake.rs`・`ceiling.rs`・`plan.rs` の 16 か所 |
| drift | 計画の床の理由の分け | `plan.rs` の定数 CONTENT_DRIFT（中身の違いだけがつながり・形の理由は止める） |
| teeth | 歯 | `tests/proposed.rs` の f199_ の 6 本と `tests/plan.rs` の f199_ 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 本流 5b8aa9a（便 198 の着地 ca8640a と便 196 の着地の後）。ADR-33 と要件書 第 1.55 版は発効済み。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。器の席へ器の行を頼む。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gt"
title = "網の外の関数の中の突き合わせの字をつながりに数える（判断の記録 ADR-33 の便 2・決定 (2)(7)・要件書 第 1.55 版の FR28 と AC31）: crates/folio/src/verdict.rs の Report に関数 link（違反を 1 つ積み、つながりの印を付ける）を足し、設計ノートの参照 id の解決と別のノートの実在と散文の参照 id（note.rs の 3 か所）・判断の記録どうしの前任と後継の実在と後継と置き換えた記録の双方向と retired の後継の輪と欄の決まりの出所（adr.rs の 5 か所・双方向と輪は関数 one_or_link で相手が自分自身なら止める）・発効した判断の記録の封の欠け（seal.rs）・入口と相談窓口と天井の英字の語（entrance.rs・intake.rs・ceiling.rs）・計画の床の節の型の置き場と行の索引の中身の違いと索引に在る計画だけの行と宙に浮いた依存（plan.rs の 4 か所）の 16 か所を関数 link で積む（note.rs の meta の相手も自分自身なら止める）。plan.rs は関数 drift の中身の違いの理由を定数 CONTENT_DRIFT に分け、形の理由（節の数・印の崩れ・印が節の外）と計画だけの行の id の重なりは止めるまま。素の folio check の出力と終了・derive・口の出力の形は変えない（base = 本流 5b8aa9a・見本 impl/d199 85b6d84）"
req = ["FR28"]
section = "1"
write-set = ["crates/folio/src/verdict.rs", "crates/folio/src/note.rs", "crates/folio/src/adr.rs", "crates/folio/src/seal.rs", "crates/folio/src/entrance.rs", "crates/folio/src/intake.rs", "crates/folio/src/ceiling.rs", "crates/folio/src/plan.rs", "crates/folio/tests/proposed.rs", "crates/folio/tests/plan.rs", "crates/folio/tests/check.rs", "crates/folio/tests/adr.rs"]
verify = ["cargo nextest run -p folio --test proposed", "cargo nextest run -p folio --test plan", "cargo nextest run -p folio --test check", "cargo nextest run -p folio --test adr", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "crates/folio/tests/proposed.rs の f199_ の 6 本（網の外の突き合わせの字 10 の中身は口が止めずにつながりに名指し素の床が同じ字で落とす・retired の後継の輪もつながり・1 つの file の形の字は止めるまま・判断の記録と設計ノートが自分自身を指す字は止めるまま・置き換えた記録の superseded_by を後で書く順はつながり）と便 198 の f198_ の 19 本が緑、crates/folio/tests/plan.rs の f199_ の 1 本（計画の床の中身の違い・索引に在る計画だけの行・宙に浮いた依存はつながり・計画だけの行の id の重なりと生成区間の印の崩れは止める）と便 183 の f183_ の 7 本が緑、crates/folio/tests/check.rs の歯の全部（素の folio check の出力と終了が同じ）が緑、crates/folio/tests/adr.rs の歯の全部（判断の記録の床の字と数が同じ）が緑、clippy 0 警告"
<!-- contracts:end -->
