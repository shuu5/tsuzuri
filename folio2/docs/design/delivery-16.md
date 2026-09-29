# 設計: 便 16 — 入口の面を正本から `folio face --face index` で生成する（入口の正本 v0.2）

- 要件: FR4（3 面を単一の生成器と単一の design token で出す）/ NFR2（見た目の材料は 1 か所から・部品目録に無い class は rules 行 R-3 の値 0）
- 条: P-2.1・P-2.2・P-2.4（人が読むページは 1 つの生成器から・手書きのページを入口の導線に置かない・部品は閉じた一覧）/ P-6.1・P-6.2・P-6.3（正本は YAML・生成物を手で直さない・数と日付は他の正本から数えて出す）/ P-4.1・P-4.2（黙って飛ばさない）/ P-10.1・P-10.2（独立した凍結 anchor）
- 判断の記録: ADR-5（決定 (2) 入口の案内文の正本 = index.yaml・数と日付は生成のたびに他の正本から数える／決定 (1)(3)(4)(5)）。便 15（f2-648.26）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」（f2-648 notes・3 面 → walk 承認 → 組み立てと配信の順序）。入口の正本は下書き（発効は 3 面の walk 承認と同時・AC5）なので v0.2 の変更に承認は要らない。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 q が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

見本 3 面の最後、入口の面（`design-intent/preview/index.html`・2026-09-12 に持ち主が見た目を承認した手書きの HTML）を、正本から機械で生成する。便 14・便 15 の `folio face` に面 index の生成器（`crates/folio/src/face_index.rs`）を足し、同じ規律で組む: 生成物の文字列は (α) 正本の欄の値を escape して逐語で差し込んだもの／(β) 生成器が閉じた表として持つ名札／(γ) 正本から数えた数 の 3 種だけ。入口の面だけの決まり（ADR-5 決定 (2)）: 案内の文（誰のため・棚の凡例・関係の名札・読む順番・相談窓口）は入口の正本 `index.yaml` から出し、数と日付（条の数・要件の数・語彙と rules の数・判断の記録の本数・各文書の版と生成日・文書が読めるか）は index.yaml に書かず、生成のたびに他の正本から数えて出す。入口の正本は v0.2（相談窓口に命令の字面〔command〕を足した・planner の PR #63 で main に置いた・下書きのままなので承認は要らない）。読む順番の各行き先の節の id〔at〕は本便の write-set で worker が `design-intent/index.yaml` の stops 9 行に足す（値は (b) の表のとおり・行の形は `- {doc: <doc>, at: <at>, label: <label>}`）。便 12 の歯 `crates/folio/tests/entrance.rs` の 1 本（entrance_stop_doc_not_on_shelf_fails）は index.yaml の 1 行の字面を変異の当て先に持つ（`- {doc: constitution, label: 目指すことと誰のため}` の行）ので、同じ便でその当て先を at 入りの行に更新する（期待は変えない・違反 1 のまま）。planner が先に at を足したところ CI がこの歯で落ちた（2026-09-17・PR #63・main に戻して本便に載せ替えた）。床は変えない: `entrance.rs` は stops の doc と label・intake の 5 欄しか数えず、足した欄は数えない（実測・admin も確認）。部品の名札は便 13 の閉じた一覧からだけ出す。受入の機構は便 14・15 と同じ 2 つ: 生成した面を `folio parts --check` に掛けて目録外 0（AC2）と、独立した最小の凍結 fixture との byte 一致（P-10.1）。本便が着地すると 3 面が揃うので、その後に planner が生成した 3 面を tailnet の内側で見せて持ち主の walk 承認（AC5・入口の正本の発効も同時）を取る（本便の外）。

読む正本は 5 file と 1 dir: `index.yaml`（案内文・本体）・`constitution.yaml`（北極星の文・条の数と段ごとの数・版と生成日）・`srs.yaml`（要件・非機能要件・受入基準の数・版と生成日）・`vocabulary.yaml`（語の数）・`rules.yaml`（行の数）・`adr/` の下で名が ADR- で始まり .yaml で終わる file（本数だけ・中身は読まない）。凍結 anchor・部品目録の実行時の写し・設計ノートの置き場は読まない。

planner の実測（2026-09-17・main d747a9c + 本 PR の index.yaml v0.2）: 入口の正本 v0.2 の documents は 4（constitution・srs・design-note・adr の順・absent は前 2 つが null・後 2 つが文字列）・annexes 2（vocabulary・rules・inside はどちらも constitution）・relations 4（id は binds・before-build・inside・amends）・lanes の rows 3（stops は各 3・at は本便で足す・値は上から s0 / s1 / fig-rail / s5 / s3 / s5 / s2 / s6 / s7）・intake の steps 4・note 1・command 1。他の正本: 憲法 v1.0（生成日 2026-09-12・条 27 = いつも守る 17・確認してから変える 4・絶対にやらない 6）・要件書 v1.3（生成日 2026-09-12・機能 15・非機能 3・受入基準 13・制約 9）・語彙 40 語・rules 26 行（閾値 16・開発規律 10）・判断の記録 5 本。見本の入口の部品の名札は 12 種（hub-cover・figure-panel・doc-shelf・shelf-card 4・shelf-link 4・status-line・intake-line・chapter-deck-band 2・reader-lane 3・intake-callout・freshness-stamp・font-size-control）で、行内の様式は --shelf-n と --band-n の 2 種。棚の並び（見本の css の shelf-grid）は 文書の id 4 つ（constitution・srs・design-note・adr）と関係 4 つに固定の置き場（shelf-c・shelf-s・shelf-d・shelf-adr・shelf-l1・shelf-l2・shelf-branch・up）を持つ。

(a) 命令の形: 便 14 の `folio face --face <面の名> --dir --out --write|--check` のまま。`--face index` で本便の生成器へ分岐する（これで 3 面が全部生成器を持ち、「生成器はまだ無い」の分岐は消える）。終了コード・文言・`--out` に既定が無いこと・導出できないときに 1 byte も書かないことは便 14 と同じ。

(b) 入口の面の組み立て（`face_index.rs`）。骨格は見本と同じ順: head（title「folio2 — 設計文書の入口（<入口の版>）」・favicon は見本の入口と同じ data URI・folio.css と folio-ui.js）→ skip-link → site-bar（nav は入口に aria-current・here「入口 ▸ 文書の一覧」・freshness-stamp「生成 <入口の生成日> · <入口の版>（<状態の名札>）」）→ main → hub-cover → 棚（figure-panel）→ status-line → intake-line → 章 01 読む順番 → 章 02 相談窓口 → foot → doc-locator。見本の layer-line は出さない。字下げ無し・部品ごとに改行 1 つ・末尾に改行 1 つ・escape と id の規則は便 14 と同じ。便 15 で `face.rs` に寄せた共有の口（面の骨格の型 Frame の関数 head・toc・band・foot・dc と関数 card・hint・hint_q・esc・val）を使う。入口は toc と章の帯の番号付きの章を持たないので、Frame の band は slim の帯（読む順番・相談窓口）にだけ使い、toc は出さない。
- 名札の表（β・本便で足すもの・表に無い値は導出できない）:
  - 文書の id と棚の置き場・面の file・原語の札: constitution = shelf-c・constitution.html・「CONSTITUTION」／srs = shelf-s・srs.html・「SRS」／design-note = shelf-d・面なし・「DESIGN」／adr = shelf-adr・面なし・「ADR」。documents の id がこの 4 つの集合と過不足なく一致しなければ導出できない（棚の並びは見本の css に固定）。
  - 関係の id と置き場（shelf-link の class）: binds = shelf-l1／before-build = shelf-l2／inside = 「shelf-branch branch」（見本と同じ 2 つの class・branch は見本の css が縦の点線の矢印を描く様式・付録の chip を伴う）／amends = up（判断の記録の枠の上・shelf-adr の div の中）。relations の id がこの 4 つと過不足なく一致しなければ導出できない。
  - 付録の id と憲法の章: vocabulary = 憲法 §7（constitution.html#s7・数は語彙の terms の数・単位「語」）／rules = 憲法 §5（constitution.html#s5・数は閾値行 + 開発規律行・単位「行」）。annexes の id がこの 2 つと過不足なく一致しなければ導出できない。
  - 読めるか: constitution と srs は読める（面が在る）・design-note と adr は面が無い。読める数 = 入口 1 + 読める文書の数。
  - 入口の状態（meta の status）: draft「下書き・拘束力なし」・effective「発効」。
  - 読む順番の行き先（stops の at）: 憲法は s0〜s8・要件書は s1〜s8 と fig-context・fig-rail・fig-verdicts のどれか（便 14・15 の面の anchor の集合）。それ以外は導出できない。
- hub-cover: cover-eyebrow = doc-type「入口 (index)」+「folio2 — 設計文書（design-intent）」／h1 = 「folio2 — <meta の title>」／p（class north-star）= 「目指すこと: 」+ q に憲法の north_star の statement + 「 — 」+ a（href constitution.html#s0）「憲法 §0 で読む」／summary-card = ic「誰」・lab = audience の label・txt = audience の text／p（class hub-who）= 「誰のため: 」+ audience の short + 小窓「?」に audience の text。
- 棚（figure-panel・id docset・data-role diagram）: fig-title = fn「棚」+ shelf の title + 「 ── いま読めるのは 」+ b に「<読める数> 面」+ 「 ＝ このページ（入口）と、<読める文書の type を・で繋ぐ>」+ fig-tools（小窓「凡例」= fig-legend の lg を shelf の legend の行ごとに〔id readable = sw ok・absent = sw neutral・binds = sw line・inside = sw dash・それ以外の id は導出できない〕+ text／小窓「図の説明」= p に shelf の explain／zoom-btn／zoom-close）。shelf-head = span（class shelf-here）「▣ 入口 = いま見ているページ」+ span（class sub）「これから増える文書 <面が無い文書の数>（<その type を・で>）／ 付録 <annexes の数>（憲法の中）」。shelf-minimap = state ok「入口（いまここ）」→ 読める文書を a（class「state ok」・href #doc-<id>）で type → 面が無い文書を a（class state・href #doc-<id>）で type（design-note の後に「｜」で adr）「｜」a（class state・href constitution.html#s7）「付録 <annexes の数>」。doc-shelf（class shelf-grid・style --shelf-n:<読める文書の数 + 面が無い文書のうち design-note の 1>〔見本と同じ 3〕）の中は棚の置き場の順（shelf-c・l1・shelf-s・l2・shelf-d・branch・adr）:
  - shelf-card（article・class「<置き場> is-absent〔面が無いとき〕」・ただし adr は見本と同じく置き場 shelf-adr を外側の div（class shelf-adr・up の link と同じ div）に持ち article の class は「is-absent」だけ・id doc-<id>・data-doc-type=<id>）: p（class sc-type）= type + span（class sub）に原語の札／p（class sc-use）= use／p（class sc-row）= 読める文書は span（class「state ok」）「● 揃っている」+ span に数〔constitution = 「<条の数> 条（<段の名> <数> · …）」・srs = 「機能 <数> · 非機能 <数> · 受入基準 <数>」〕、面が無い文書は span（class state）「○ まだ無い」〔adr は「○ <本数> 本」〕+ span に absent の文（adr で本数が 1 以上なら absent の文の代わりに「ページはまだ無い」）／読める文書は p（class sc-row）に span（class up）「更新 <その正本の生成日>・<版>」と a（class sc-open・href 面の file）「開く →」と a（class sc-hit・href 面の file・aria-hidden・tabindex -1）、面が無い文書は p（class sc-none）「—（まだページはありません）」。
  - shelf-link（span・class は置き場）: span（class arrow）+ span（class lbl）に relation の label + 小窓「?」に hint。branch はさらに span（class annex-chips）に annexes ごとの a（href 憲法の章）「付録 <type> 」+ span（class cnt）「<数> <単位> → 憲法 §<n>」。up は shelf-adr の div の中で「<from の type> → <label>」（小窓は hint）。
  - figcaption の ver = 「棚 · <入口の版> <入口の生成日> · index.yaml」。
- status-line（section・aria-label「いまの状態」）: p（class「st ok」）= mark「●」・k「揃っている」・v「<読める文書の type を・で> が読める（付録の <annexes の type を・で> は憲法の中）」／p（class st）= mark「○」・k「まだ無い」・v = 面が無い文書ごとに「<type>（<absent の文か「<本数> 本・ページはまだ無い」>）」を「／」で繋ぐ。見本の「次の一手」「まだ測っていない」の行は正本に無いので出さない。
- intake-line（p）: span（class k）「相談を始める:」+ span（class cmd）に intake の command + span に「（と AI に頼む）→ 」+ steps を「 → 」で繋ぐ + a（class more・href #s2）「くわしく →」。
- 章 01 読む順番: 帯（slim・kicker「読む順番」・h2 = lanes の title・lead = lanes の lead）。chapbody に div（class lane-grid・style --band-n:<rows の数>）の reader-lane（article）を行ごとに: p（class who）= span（class av）に mark + who／p（class why）= why／ol の li = stops ごとに a（href「<doc の面の file>#<at>」）「<doc の type> <label>」／span（class time）「約 <minutes> 分」。
- 章 02 相談窓口: 帯（slim・kicker「相談窓口」・h2 = intake の title・lead = intake の lead）。chapbody に intake-callout: div に h3 = heading・p = text・div（class steps）に steps ごとに span「<i> · <step>」／div（class cmd）= command。note が在れば details.note（summary「補足」・p = note）。
- foot: ft-plain「このページは正本 index.yaml と他の正本から folio が生成した · 入口 <版>（<生成日>）· 手で直さない」+ details.machine の dl = id / version / status / sources（読んだ正本の file 名を「 / 」で）。doc-locator「この文書の所属: 設計文書（design-intent）/ 入口 — <a 憲法へ> · <a 要件書へ>」。prevnext は出さない（入口は起点・見本にも無い）。
- 本便の入口の面が使う部品は 12 種: freshness-stamp・font-size-control・hub-cover・figure-panel・doc-shelf・shelf-card・shelf-link・status-line・intake-line・chapter-deck-band・reader-lane・intake-callout。

(c) 導出できない = 2 で出力先に 1 byte も書かない（便 14 の (d) に足すもの・ここに挙げたものだけ）: index.yaml と他の 4 file の読めない・重複キー・空の文書／adr の dir が無い・読めない／読む欄が無い・型が違う（index の meta の id・title・version・status・generated／audience の label・short・text／shelf の title・explain・legend〔各 id・text〕・documents〔各 id・type・use・absent〈null か文字列〉〕・annexes〔各 id・type・inside〕・relations〔各 id・from・to・label・hint〕／lanes の title・lead・rows〔各 id・mark・who・why・minutes・stops〈各 doc・at・label〉〕／intake の title・lead・heading・command・text・steps〔文字列の一覧〕／憲法の north_star の statement・meta の version・generated・counts・articles／要件書の meta の version・generated・counts・requirements・nonfunctional・acceptance／語彙の terms／rules の thresholds と discipline）。任意 = intake の note・meta の approval／表に無い値（文書の id の集合・関係の id の集合・付録の id の集合・legend の id・stops の doc と at・meta の status）／stops の doc が面の無い文書／憲法の counts と条の数の不一致・要件書の counts と数の不一致／`--out` の親 dir が無い。

(d) 歯 `crates/folio/tests/face.rs`（既存の歯の file に足す・関数名はすべて face を含める＝verify の filter 語・便 14・15 の歯は変えない）。filter 語 entrance が base で解く歯は `tests/entrance.rs` 12 本と `src/entrance.rs` の unit 2 本（実測）で、`src/entrance.rs` は write-set に在るが本文も期待も変えない。filter 語 face が base で解く歯は `tests/face.rs` 30・`src/face.rs` 5・`src/face_constitution.rs` 2・`src/face_srs.rs` 5・`tests/parts.rs` 3（実測・main d747a9c）で、`src/face_constitution.rs`・`src/face_srs.rs`・`tests/parts.rs` は write-set に在るが本文も期待も変えない（`src/face_srs.rs` の幅 120 で正規化した行数は 1,370 で余地 130＝size は S で申告する）。
- 便 12 の歯の当て先の更新: `tests/entrance.rs` の entrance_stop_doc_not_on_shelf_fails の変異の当て先を at 入りの行（`- {doc: constitution, at: s0, label: 目指すことと誰のため}` → doc を nowhere に）に替える。期待（違反 1・行き先が棚に無い）は変えない。他の 10 本は触らない。
- 凍結 fixture との byte 一致（P-10.1）: `--face index --dir tests/fixtures/face --out <一時 file> --write` = 0 ∧ 出力が `tests/fixtures/face/expected-index.html` と byte 一致。憲法面と要件書面の期待は変えない。
- 実の正本で部品目録に合う（AC2）: `--dir design-intent` で書き `folio parts --check --dir design-intent --page index=<一時 file>` = 0 ∧「違反 0」。3 面まとめて: 3 つの一時 file を `--page` 3 つで渡して = 0（3 面が揃った証拠）。
- 逐語と件数の census（実の正本・yaml-rust2 で直に読む）: 出力に audience の text・documents の use・relations の label・lanes の who と why・stops の label・intake の text と steps が含まれる／shelf-card 4・shelf-link 4・reader-lane = rows の数／「<条の数> 条」「機能 <数>」「<語の数> 語」「<行の数> 行」「<判断の記録の本数> 本」の字面／stops の a の href が「<面の file>#<at>」の形で rows × stops の数だけ／data-component の値は 12 種の中。
- check の 3 値: 一致 0 ／ 1 byte 変えて 1 ／ 無くて 2。
- 導出できない入力 8 つ（fixture の写しに変異・どれも 2 で出力先が出来ていない）: documents から adr を消す／relations の id を 1 つ別の名に／annexes に 3 つ目を足す／stops の at を s9 に／stops の doc を design-note に／legend の id を別の名に／constitution.yaml の counts を 1 ずらす／adr の dir を消す。
- unit（`src/face_index.rs` の中・名に face を含む）: 文書の id の表（4 つ・他は Err）・stops の at の解き方（憲法 s0〜s8・要件書 s1〜s8 と 3 つの図・他は Err）・読める数の数え・使う部品 12 種の faces に index が入っている。
- 凍結 fixture（`tests/fixtures/face/`・手書き・最小）: `index.yaml`（新規・meta・audience・shelf〔legend 4・documents 4・annexes 2・relations 4〕・lanes 1 行〔stops 2 = constitution s0・srs fig-rail〕・intake〔command・steps 2・note〕）と `adr/ADR-1.yaml`（新規・本数を数えるためだけの最小の file・中身は id と status の 2 欄）・`expected-index.html`（新規・(b) の規則で組んだ面・最初の 1 回は生成器の出力を写して置いてよく、置いた後は歯が固定する）。憲法・要件書・語彙・rules の fixture はそのまま読む（変えない）。独立性の分担は便 14 と同じ。
- 実装の下書き: 前の run の commit 6dff0f8（branch scribe2/f2-648.27-20260917T145433Z・verify 6 段 rc 0・gate の不通過は契約の字面 2 か所〔adr の card の置き場・inside の link の class〕が見本と食い違っていたためで、実装は見本どおり）を写してよい。写すときに、north_star の statement と meta の title の検査を 1 か所に畳み、「<本数> 本・ページはまだ無い」の分岐を 1 つの関数にまとめる。

(e) 便 15 までの形との接続: 新規は `crates/folio/src/face_index.rs`・fixture 3 本。`crates/folio/src/main.rs` は `mod face_index;` の 1 行だけ。`crates/folio/src/face.rs` は run の分岐に「index」を足し（「生成器はまだ無い」の分岐は消える・その文言を確かめる便 14 の歯 face_index_has_no_generator_yet は期待が変わるので、本便で「index の面が書ける」に書き換える＝便 14 の歯のうちこの 1 本だけ変える）、本便の名札の表を足す（他は変えない）。`face_constitution.rs`・`face_srs.rs`・`parts.rs`・`entrance.rs`・`check.rs`・他の src・`build.rs`・`Cargo.toml`・`Cargo.lock`・`scripts/`・`.github/workflows/` は触らない。`design-intent/index.yaml` は stops の 9 行に at を足すだけ（他の欄は変えない・`folio check` は合格のまま）。`crates/folio/tests/entrance.rs` は 1 本の当て先の字面だけ。`crates/folio/src/entrance.rs` は触らない。外部 crate は増えない。正規表現は使わない。size は S = 中身を変える既存の file 1 本あたりの増分の見積（`face.rs` の増分は run の分岐 1 行と名札の表で 100 行未満・`main.rs` は 1 行・新規の `face_index.rs` と歯は増分に数えない）。生成した面を `design-intent/preview/index.html` へ書く（見本の置き換え）のは本便の外（A-1）。

## 2. 範囲

- 入れる: 入口の正本 v0.2（stops の at・intake の command）・入口の面の生成器・歯・凍結 fixture。
- 入れない: 読む順番の札（lane-chip・憲法面と要件書面に置く側・3 面の walk の後の直しの便で入口の正本の stops を読んで出す）・見本の置き換えと退役（A-1）・walk 承認（本便の後・planner が tailnet で見せる）・組み立てと配信（build / serve）・見た目だけの直し・入口の正本の発効（walk 承認と同時）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| labels | 名札の表 | 文書の id と置き場・関係の id と置き場・付録・読めるか・stops の at |
| page | 入口面の導出 | hub-cover・棚・状態の行・相談の行・読む順番・相談窓口 |
| counts | 数と日付 | 他の正本から数える（条・要件・語・行・判断の記録・版・生成日） |
| fixture | 凍結 fixture | index.yaml と adr 1 本と期待の面 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 15 の歯は期待不変で全部回る・便 14 の歯 1 本だけ (e) のとおり書き換える）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "q"
title = "入口の面を正本から folio face --face index で生成する（入口の正本 v0.2）"
req = ["FR4", "NFR2"]
section = "1"
write-set = ["+crates/folio/src/face_index.rs", "crates/folio/src/face.rs", "crates/folio/src/main.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/entrance.rs", "crates/folio/tests/face.rs", "crates/folio/tests/parts.rs", "crates/folio/tests/entrance.rs", "design-intent/index.yaml", "+tests/fixtures/face/index.yaml", "+tests/fixtures/face/adr/ADR-1.yaml", "+tests/fixtures/face/expected-index.html"]
verify = ["cargo nextest run -p folio face", "cargo nextest run -p folio entrance", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face の歯（入口面の凍結 fixture との byte 一致・実の正本で parts --check 合格〔3 面まとめても〕・逐語と件数の census・check の 3 値・導出できない入力 8 つ・unit）が緑、便 14・15 の歯が期待不変で緑（便 14 の 1 本は index の面が書ける形に書き換え）、便 12 の歯 entrance が期待不変で緑（1 本は当て先の字面だけ更新）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
