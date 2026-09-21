# 設計: 便 84 — 条の機構の札にいつから動くかの意味を添え、用語集の面に欄の名前の節を出す（天井 18 周目 読みやすさ F-3・20 周目 読みやすさ F-2・FR4）

- 要件: FR4（人が読むページを 1 つの生成器から出す）
- 条: P-2.1（人が読むページはすべて 1 つの生成器から出力する）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-4.1（読めない入力を異常なしにしない）
- 出所: 一括 10 の仕分け C。天井の 18 周目 読みやすさ F-3「条のカードの機構の札に、いつから動くか（M0 など）の意味を面へ出す。憲法の schema の節そのものは改訂の範囲なので触らない」と 20 周目 読みやすさ F-2「用語集の面に欄の名前（field_terms）の節を出し、『規範文』を面から引けるようにする」。
- 位置: 便 79（歯の file crates/folio/tests/face_constitution.rs を置く）と 便 81（歯の file crates/folio/tests/face_srs.rs を置く）の後。両方の着地の後に受け付ける。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cg が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き。
- 門: 本便は design-intent の下の正本を 1 file も書き換えない（触るのは crates/folio/ と tests/fixtures/face/ だけ）ので、天井の門（folio ceiling --gate）は 通す を返す。

## 1. 目的と中身

① 憲法の面の条のカードは、機構の小窓に「機械が拒む・M0 で動く・編集時・止める」のような札を並べる。いつから動くかの値（いま動く・M0 で動く・便 0 で動く・M1 で動く・判断の記録の欄の決まりの後）は面に出ているが、M0 や 便 0 が何を指すのかは面のどこにも無いので、読み手は札を読めても意味が取れない。② 語彙の正本は語（terms・実測 54 語）と欄の名前（field_terms・実測 7 語 = 段・規範文・やさしく言うと・強度・型（いつ守るか）・機構・縛る相手）の 2 つの節を持つが、面に出るのは語だけである。憲法の面も要件書の面も、本文でいちばん多く使う「規範文」「やさしく言うと」「強度」を用語集から引けない。本便はこの 2 つを面の側だけで直す。正本 design-intent/constitution.yaml と design-intent/vocabulary.yaml は 1 byte も触らない。

orchestrator 席の実測（2026-09-22・main 81bc02c）:

- いつから動くかの名札は crates/folio/src/face.rs の mechanism_live_label（595〜604 行）で、組み立て時に憲法の正本から導いた型 ce::MechanismLive への網羅の場合分け（その他を受ける枝なし・ADR-11 決定 (4)②）。値は 5 つ。同じ形の名札が近くに 9 本並ぶ（段・強度・型・縛る相手・機構の種別・stage・polarity・根拠の種別・撤退の種別）。うち強度だけは名札（strength_label・539 行）と意味（strength_meaning・548 行）の 2 本を持つ形になっており、本便はその形に倣う。
- 機構の小窓を組むのは crates/folio/src/face_constitution.rs の item_row（510〜652 行）の 573〜600 行。種別の名札と いつから動くか の名札を中黒で繋ぎ、stage と polarity が在れば足し、note が在れば区切って足し、face.rs の hint（358〜363 行）に名札 機構 で渡す。
- 用語集の語の行を組むのは face.rs の glossary_rows（1039〜1068 行）で、引数の語彙の根から terms だけを読む。呼ぶのは 2 か所 = face_constitution.rs の glossary_chapter（1051〜1066 行・憲法の面の章 07）と crates/folio/src/face_srs_rtm.rs の glossary_chapter（100〜112 行・要件書の面の章 08）。どちらも部品 glossary-term-table の div 1 つの中に行を並べる。部品目録 design-intent/preview/parts.json は glossary-term-table の面を constitution と srs の 2 つに定めており、同じ面に 2 つ置くことを妨げる欄は持たない。
- field_terms を既に読んでいる所が 1 つある。face_constitution.rs の reading（459〜489 行）が id が tier の行の def だけを章 01 の lead に使う。行の欄は id・term・en・def の 4 つで、terms と違い short と note を持たない（glossary_rows は note を任意で読むので、そのまま通る）。
- 語の id の重なりは 0（terms の 54 個と field_terms の 7 個に同じ id は無い。実測）。面の行の id は g- を頭に付けるので、節を 2 つ並べても重複する id は出ない。
- 凍結の写し tests/fixtures/face/vocabulary.yaml は terms を 2 語、field_terms を 1 語（id が tier）持つので、②の直しは写しの面の字を変える。
- 章の見出しの数えの字は face.rs の count_word（471〜477 行・9 までは 「{n} つの{noun}」・10 以上は 「{n} の{noun}」）。見出しに数を出すときはここから出す（便 70）。
- 様式 design-intent/preview/folio.css は h3 と小窓の規則を既に持ち、部品 --check（crates/folio/src/parts.rs 233〜242 行）は面の class が folio.css に在ることだけを見る。本便は新しい class を 1 つも作らないので様式は 1 byte も触らない。

### (a) いつから動くかの意味

face.rs に名札の隣の意味を 1 本足し、憲法の面の小窓で使う。

1. 関数 mechanism_live_meaning(l: ce::MechanismLive) -> &'static str。mechanism_live_label と同じ位置（595 行の隣）に置き、同じ形の網羅の場合分けで持つ（その他を受ける枝を置かない＝憲法の側で値が足されても消えても組み立てが通らない）。字は次の 5 つ。
   - いま動く → 今の folio に在る
   - M0 で動く → M0 = 要件書の scope の 作る の側に在る段
   - 便 0 で動く → 便 0 = 最初の便の段
   - M1 で動く → M1 = 要件書の scope_m1 の 作る の側に在る段
   - 判断の記録の欄の決まりの後 → 判断の記録の欄の決まりが定まった後
   段の中身そのものは要件書の正本が持つので、面は段の名と、その定義が要件書のどの節に在るかを指すだけにする（正本の一覧を面へ写さない・P-6.3）。
2. face_constitution.rs の item_row の 573〜578 行で、いつから動くか の名札に続けて括弧で意味を添える形にする（「{名札}（{意味}）」）。種別・stage・polarity・note の並びと区切りは変えない。機械のための面の行（machine・579 行〜）は正本の値のままなので触らない。

### (b) 用語集の欄の名前の節

1. face.rs の glossary_rows に、読む節の名を受ける引数を 1 つ足す（glossary_rows(o, v, section)）。中身は今までどおり（en が null なら英字を出さない・note が在れば 2 つ目の段落）。節が無い・一覧でないときの断り方は今までどおり（f が Err を返す）。
2. face_constitution.rs の glossary_chapter（章 07）で、今の div（terms の表）の後ろに、語彙の根が field_terms を持ち、その一覧が空でないときだけ次を足す。
   - 見出し 1 行: h3 の中に 「欄の名前 — {count_word(n, 語)}」（n は field_terms の数）
   - 部品 glossary-term-table の div 1 つと、その中に glossary_rows(o, v, field_terms) の行
   field_terms が無い・空のときは 1 行も足さない（面の字は本便の前と同じ）。
3. face_srs_rtm.rs の glossary_chapter（章 08）に、2 と 1 字も違わない同じ字面を置く（便 36 からの「2 面で同じ字面」を保つ）。
4. 章の見出し（h2）と目次の字は触らない。

### (c) 歯（関数名は f84_ で始める。`grep -rn 'fn f84_' crates/folio/tests` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f78_）

crates/folio/tests/face_constitution.rs（便 79 が置く）に 3 本。

1. f84_mechanism_chip_explains_the_stage: 実の置き場の写しで憲法の面を書く → 0 ∧ 機構の小窓の本体に 「M0 で動く（M0 = 要件書の scope の 作る の側に在る段）」 の形が 1 回以上在る ∧ 面に出るいつから動くかの名札のそれぞれに括弧の意味が付く（歯の側で、面に現れた 5 つの名札のどれについても、名札の直後が括弧であることを見る）。本便の前の main では括弧が付かないので赤い歯。
2. f84_glossary_has_the_field_terms_section: 同じ面の章 07 に、部品 glossary-term-table の div が 2 つ在る ∧ 2 つ目の前に h3 の 「欄の名前」 が在る ∧ 2 つ目の中に正本 design-intent/vocabulary.yaml の field_terms の 7 語の term が全部在る（数と語は歯の側で正本から直に読む）∧ 面に 「規範文」 の行の id（g- を頭に付けたもの）が在る。本便の前の main では div が 1 つなので赤い歯。
3. f84_glossary_heading_counts_from_the_source: h3 の数えの字が、歯の側で正本の field_terms の数と count_word の規則から独立に組んだ字と一致する。

crates/folio/tests/face_srs.rs（便 81 が置く）に 2 本。

4. f84_srs_glossary_has_the_same_field_terms_section: 実の置き場の写しで要件書の面を書く → 0 ∧ 章 08 に部品 glossary-term-table の div が 2 つ在り、2 つ目の中身が憲法の面の 2 つ目の中身と byte 一致する（便 36 からの「2 面で同じ字面」）。
5. f84_missing_field_terms_leaves_the_face_unchanged: field_terms を持たない写しの語彙で 2 面を書く → 0 ∧ どちらの面にも h3 の 「欄の名前」 が無く、部品 glossary-term-table の div が 1 つずつ。
6. 回帰（期待不変・verify の 2 行目）: crates/folio/tests/face_constitution.rs の f79_ と f80_・tests/face_srs.rs の f81_・tests/face.rs・tests/site.rs・tests/badge.rs の既存の歯すべて。tests/face.rs の census の歯が部品の数を数えているときは、用語集の表が 2 つになる分を歯の側で正本から数え直す（歯の期待を生成器の字から写さない）。

### (d) 凍結の写し

面の字が変わるのは憲法の面と要件書の面なので、次の 2 本を同じ着地で生成し直して置き換える（手で直さない・歯と同じ経路で binary に face を当て、その出力を写す）。

- tests/fixtures/face/expected.html（(a) と (b)）
- tests/fixtures/face/expected-srs.html（(b)）

ほかの 5 本（expected-index.html・expected-index-sheet.html・expected-adr.html・expected-note.html・expected-site-adr-2.html）は 1 byte も変わらない。写しの正本 tests/fixtures/face/vocabulary.yaml と constitution.yaml は触らない。

### (e) 大きさ

src は 3 本。crates/folio/src/face.rs（幅 120 で正規化して 1306 行・余地 194・見積は + 約 15 行。便 83 が先に着地しても余地は約 170）／face_constitution.rs（1130 行・余地 370・便 79 と 80 が先に着地すると余地は約 270・見積は + 約 15 行）／face_srs_rtm.rs（109 行・余地 1391・見積は + 約 10 行）。歯は tests/face_constitution.rs（+ 約 70 行）と tests/face_srs.rs（+ 約 40 行）。size **S**（src の各 file の余地は 100 以上）。外部 crate は増やさない。face_srs.rs・face_adr.rs・face_note.rs・face_index.rs・図の道具・部品目録・様式・design-intent の下の正本・tests/floor_cases.yaml・CI の yml は触らない。

### (f) 本便が運ばないもの

正本 design-intent/constitution.yaml（条の機構の欄・schema の節・値域）と design-intent/vocabulary.yaml（語と欄の名前の中身・定義の字）。憲法の schema の節はどれも改訂の範囲なので触らない（この直しは面の側だけで済む）。要件書の scope と scope_m1 の中身を面へ写すこと。入口・判断の記録・設計ノートの 3 面（用語集の章を持たず、脚の札から憲法の章 07 へ導く形は便 74 で着地済み）。用語集への札（glossary_chip）の数えの字（今までどおり terms の数を出す）。部品目録と様式。要件書 FR4 の規範文と版。

## 2. 範囲

- 入れる: いつから動くかの意味の名札 1 本・機構の小窓の字・用語集の共有の口の引数 1 つ・2 面の欄の名前の節（見出し 1 行と表 1 つ）・歯 5 本・凍結の写し 2 本の生成し直し。
- 入れない: 正本の欄と定義・憲法の schema の節・用語集の札の数え・新しい class と新しい部品・部品目録・様式・ほかの 3 面。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| meaning | 意味の名札 | face.rs の mechanism_live_meaning（網羅の場合分け・5 つ） |
| chip | 機構の小窓 | face_constitution.rs の item_row の 名札（意味） |
| rows | 語の行 | face.rs の glossary_rows に読む節の名を渡す |
| fields | 欄の名前の節 | 2 面の用語集の章の h3 1 行と glossary-term-table 1 つ |
| teeth | 歯 | tests/face_constitution.rs に f84_ 3 本・tests/face_srs.rs に f84_ 2 本 |
| frozen | 凍結 | tests/fixtures/face/expected.html と expected-srs.html の生成し直し |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cg"
title = "憲法の面の機構の小窓で、いつから動くかの名札に意味を括弧で添え（値域の型への網羅の場合分けで持ち、段の中身は要件書の節を指すだけにする）、憲法の面の章 07 と要件書の面の章 08 の用語集に、語彙の正本の欄の名前（field_terms）の節を 2 面で同じ字面で足す（正本は 1 byte も触らない・節が無い正本では字が不変・凍結の写し 2 本を生成し直す・天井 18 周目 読みやすさ F-3 と 20 周目 読みやすさ F-2）"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs_rtm.rs", "crates/folio/tests/face_constitution.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "crates/folio/tests/badge.rs", "tests/fixtures/face/expected.html", "tests/fixtures/face/expected-srs.html"]
verify = ["cargo nextest run -p folio --test face_constitution --test face_srs f84_", "cargo nextest run -p folio --test face_constitution --test face_srs --test face --test site --test badge", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f84_ の歯 5 本（機構の小窓に段の意味が括弧で付く・憲法の面の章 07 に欄の名前の節と 7 語・見出しの数えが正本と一致・要件書の面の章 08 の同じ節が憲法の面と byte 一致・節を持たない正本では面の字が不変）が緑、tests/face_constitution.rs の f79_ と f80_ と tests/face_srs.rs の f81_ と tests/face.rs と tests/site.rs と tests/badge.rs の既存の歯が全部緑（生成し直した凍結の写し 2 本との byte 一致の歯を含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
