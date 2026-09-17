# 設計: 便 14 — 憲法の面を正本から `folio face` で生成し、`folio parts --check` で目録外 0 を固定する

- 要件: FR4（3 面を単一の生成器と単一の design token で出す）/ NFR2（見た目の材料は 1 か所から・部品目録に無い class は rules 行 R-3 の値 0）
- 条: P-2.1・P-2.2・P-2.4（人が読むページは 1 つの生成器から・手書きのページを導線に置かない・部品は閉じた一覧）/ P-6.1・P-6.2・P-6.3（正本は YAML・生成物を手で直さない・同じ内容は一方を正本に他方を導出）/ P-4.1・P-4.2（実行できなかった生成を異常なしにしない・判定できないものは「まだ分からない」）/ P-10.1・P-10.2（独立した凍結 anchor・生成物どうしの突き合わせを唯一の判定にしない）
- 判断の記録: ADR-5（決定 (1) 見た目は見本・文章と数は正本・byte 一致は求めない／決定 (3) 手書きの Rust・外部の部品を足さない・部品目録を型で閉じる／決定 (4) 図は既存 3 型だけ／決定 (5) 順序 = 部品目録の型 → 憲法面・要件書面・入口面の生成）。便 13（f2-648.24）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」（f2-648 notes・便 11 → 12 → 13 → 3 面 → walk 承認 → 組み立てと配信の順序）と ADR-5 の発効の承認（f2-648.22 notes・逐語「承認する」）。見本の置き換え（手書きの見本の退役）は本便に含めない（A-1・別に問う）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 o が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

見本 3 面のうち憲法の面（`design-intent/preview/constitution.html`・2026-09-12 に持ち主が見た目を承認した手書きの HTML）を、正本から機械で生成する副命令 `folio face` を置く。生成器は見本の部品（属性 data-component の名札）・見た目の値（様式の定義 folio.css 1 本）・構造（章の並び・章の中の部品の並び・各部品の要素と class の並び）を保ち、文章と数は今の正本から組む（ADR-5 決定 (1)）。見本の HTML との byte 一致は求めない。受入の機構は、生成した面を便 13 の `folio parts --check` に掛けて 目録外の class 0・目録外の部品 0・面に置けない部品 0・許されない行内の様式 0 を歯で固定すること（要件書 AC2 の機構を生成器の出力に当てる）と、独立した最小の凍結 fixture との byte 一致（P-10.1）である。持ち主が 3 面を開いて読む承認（AC5 の形）は 3 面が揃った後の別の段で取る。本便は入口の面と要件書の面を生成しない。

読む正本は 4 file: `constitution.yaml`（本体）・`rules.yaml`（§5 の表）・`vocabulary.yaml`（§7 の用語集と §1 の段の説明）・`srs.yaml`（条の関係の欄が指す要件 id の見出しだけ）。判断の記録（adr/）と凍結 anchor（anchors/）と入口の正本（index.yaml）と部品目録の実行時の写しは読まない。

planner の実測（2026-09-17・main f44092c）: 憲法の条は 27（いつも守る 17・確認してから変える 4・絶対にやらない 6＝meta の counts と同値）・規範文は 66・supersedes_v1 を持つ条は 1（P-2）・amended_by を持つ条は 0・retreat を持つ条は 3（P-2・P-6・P-16）・note を持つ条は 6・機構の stage と polarity を持つ条は 24（human-review の 3 条は持たない）・関係の欄の参照は reqs 25・rules 47・articles 33・sections 5（値は §6 だけ）で、reqs の 19 種はすべて要件書の 4 節（requirements・nonfunctional・acceptance・constraints）の id に在り、rules と articles の参照先も全部在る（便 1 の床 R-4 が 0 で通っている）。改訂の手続きの段（amendment の steps）は 5（担当 = AI・AI・AI・持ち主・機械・各段の what は「 — 」〔前後に半角空白 1 つずつの全角ダッシュ〕を 1 つ持つ）と発効の段（effective_step・n は 0）が 1。出所（sources）は 5。段の一覧（schema の enums の tier）は always・ask-first・never の順。rules の閾値行は 16（値の型は文字列 12・整数 3・表 1〔R-16〕）・開発規律行は 10・status の値は 凍結 16・仮 9・未定 1・kind の値は human-review 10・build-check 9・deny 7。語彙の terms は 40（en が null の語は 0）・field_terms は 7（id が tier の行を持つ）。正本の規範文・見出し・平易文に「<」「>」「&」「二重引用符」を含むものは 0。見本の憲法面の部品の名札は 15 種（chapter-deck-band 10・item-row 15・lane-chip 5・rail-node 4・section-lead-callout 3・他 1 ずつ）で、行内の様式は --band-n と --rail-n の 2 種。

(a) 命令の形: `folio face --face <面の名> --dir <正本の置き場・既定 design-intent> --out <出力先> --write|--check`。
- `--face` の値は便 13 の `parts.rs` の定数 FACES と同じ 3 つの名（index・constitution・srs）。3 つのどれでもなければ「まだ分からない」（終了 2・標準エラー「folio face: まだ分からない: 面の名「<値>」は index・constitution・srs のどれでもない」）。index と srs は本便では生成器が無いので「まだ分からない」（文言「面「index」の生成器はまだ無い」・出力先に 1 byte も書かない・ADR-5 決定 (4) の「黙って飛ばさず止まる」と同じ形）。
- `--out` は必ず指定する（既定を持たない・相対なら `--dir` からの相対・絶対ならそのまま）。既定を持たないのは、`design-intent/preview/constitution.html` に在る手書きの見本を黙って上書きしないため（見本の置き換えは A-1 で持ち主に問う別の便・そのときに既定を足す）。
- `--write` と `--check` はどちらか 1 つを必ず指定する（便 2・便 11・便 13 と同じ ArgGroup の形）。
- 終了コードと文言（3 値・便 11 の `folio render` と同じ判定の形）:
  - write = 0・標準出力「folio face: 書いた（N byte）」（N = 書いた byte 数）
  - check で出力先が無い = 2・標準エラー「folio face: 面が無い（未生成）」
  - check で不一致 = 1・標準エラー「folio face: DRIFT — 面 N byte ≠ 導出 M byte（手で直したか正本が変わった）」（N = 今の file の byte 数・M = 導出の byte 数）
  - check で一致 = 0・標準出力「folio face: OK — 面は正本と一致（N byte）」
  - 導出できない（下の (d)）= 2・標準エラー「folio face: まだ分からない: <理由>」。このとき `--write` でも出力先に 1 byte も書かない。
- check の比較は byte 列の一致（改行の読み替えをしない）。

(b) 面の組み立て（`crates/folio/src/face_constitution.rs`）。生成物の各文字列は次の 3 種のどれかで、それ以外の自由な文を生成器に置かない: (α) 正本の欄の値を escape して逐語で差し込んだもの／(β) 生成器が閉じた表として持つ名札（章の名・欄の名札・段の名・型の名・強度の名・機構の名・ボタンと凡例の文言・見本の絵記号の SVG の字面）／(γ) 正本から数えた数。見本の中にしか無い案内の文（表紙の副題の文・読者の札・各章の lead と補足・読む順番の札〔lane-chip〕・図の説明の文）は、正本に同じ内容の欄が在ればその欄から出し、無ければ出さない（構造は保つ・文だけを落とす）。
- 全体の骨格は見本と同じ順: head（文字符号・viewport・title「folio2 — 憲法（不変原則・<版>）」・見本と同じ favicon の data URI〔憲の字〕・`folio.css` と `folio-ui.js` を同じ dir からの相対で参照）→ skip-link → site-bar（brand・読める面の nav 3 つ〔憲法に aria-current〕・font-size-control・here「憲法 ▸ 全 10 章 — 目次へ」・freshness-stamp「生成 <生成日> · <版>（<状態の名札>）」）→ main → doc-cover-band → toc → 章 00〜08 と承認欄（各章 = chapter-deck-band の section + chapbody の div）→ prevnext（前 = 入口 index.html・次 = 要件書 srs.html）→ foot → doc-locator。見本の layer-line（暫定の断り書き）と reader-chip は出さない。各部品の要素の入れ子・属性の並び・class の並びは見本の同じ部品の字面を写し、値だけを正本から差し込む。字下げは付けず、部品ごとに改行 1 つで繋ぎ、末尾に改行 1 つ。
- escape は便 11 と同じ 5 字（& → `&amp;`・< → `&lt;`・> → `&gt;`・二重引用符 → `&quot;`・一重引用符 → `&#x27;`）を、正本から差し込む全部の値（属性の値を含む）に掛ける。id と href に使う正本の id は英数字と「-」と「.」だけを受け、それ以外を含めば導出できない。条 id の anchor は id を ASCII 小文字にしたもの（P-1 → p-1）・rules 行は r-1 / d-1・語彙は g-<id>・章は s0〜s8 と approval・図の段は a-step-<n>。
- 名札の表（β・値は全部この字面）。表に無い値は導出できない:
  - 段（tier・順は正本の schema の enums の tier の順・3 つ全部が表に在ること）: always = 名「いつも守る」・原語「Always」・class「tier-always」・色の class「ok」・意味「道具も AI も、毎回これに従う」・外すのに要るもの「憲法の改訂（§6: 判断の記録 + 持ち主の承認）」／ask-first = 「確認してから変える」・「Ask-first」・「tier-askfirst」・「warn」・「やってよいが、実行前に持ち主へ確認する」・「その場の持ち主の確認」／never = 「絶対にやらない」・「Never」・「tier-never」・「bad」・「確認があってもやらない」・「憲法の改訂（確認では解けない）」。
  - 強度（strength）: must と should は原語を ASCII 大文字にした 1 語・must-not は原語の 2 語を ASCII 大文字にして半角空白 1 つで繋いだもの（見本の kw の字面）。
  - 型（pattern）: ubiquitous「つねに」・event「〜のとき」・state「〜のあいだ」・unwanted「〜になったら」・optional「〜ならば」。縛る相手（binds）: tool「道具」・practice「作法」・both「両方」。
  - 機構: kind = reject「機械が拒む」・build-check「生成時の検査」・human-review「人が目で確かめる」・none「なし」／live = now「いま動く」・M0「M0 で動く」・delivery-0「便 0 で動く」・M1「M1 で動く」・adr「判断の記録の欄の決まりの後」／stage = in-loop「編集時」・post「事後」／polarity = fail-open「開く」・fail-closed「閉じる」。
  - 根拠の種別（rationale の kind）: v1-incident「v1 の実害」・scribe2-article「scribe2 の条」・folio2-ruling「持ち主の裁定」。撤退条件の種別（retreat の kind）: spike「試して測る」・measure「測る」・ruling「持ち主に問う」。
  - 文書の状態（meta の status）: effective「発効・拘束力あり」・draft「未承認・拘束力なし」。
  - rules 行の状態（status）と色の class: 凍結 = ok・仮 = warn・未定 = 色なし。rules 行の種別（kind）: deny「測って落とす」・build-check「生成時の検査」・detect「記録のみ」・human-review「人が守る作法」。
  - 改訂の段の担当（who）: 値が「持ち主」なら持ち主の手番（tone-warn・上の枠）、それ以外は AI と機械の手番（下の枠）。値はそのまま出す。
  - 章: 00「北極星」/ 01「読み方」/ 02〜04 = 段の名（正本の順）/ 05「数値の表（rules）」/ 06「改訂」/ 07「用語集」/ 08「出所」/ 承認欄「承認欄」。帯の class は見本と同じ（00 band-1・01 band-5・02 band-2・03 band-3・04 band-4・05 band-6・06 band-5・07 band-1・08 band-5・承認欄 band-3 slim）。kicker の絵記号（class ico の SVG）は見本の各章の path の字面をそのまま持つ（承認欄は無し）。crumb は「憲法 › <NN> <章の名> <i>/10」。
- 表紙（doc-cover-band）: cover-eyebrow = doc-type「憲法 (Constitution)」+「folio2 — 不変原則」／h1 = 「folio2 の憲法 — <条の数> の約束を「<段の名を・で繋いだもの>」の <段の数> 段で」／cover-sub = north_star の for_whom／summary-card = ic「北」・lab「北極星（1 文）」・txt = north_star の statement／cover-meta = 「原則の総数」<条の数> 件・「内訳」= 段ごとに章へのリンク「<段の名>&nbsp;<数>」を「 · 」で繋ぐ・「版」<meta の version> / <meta の generated>／cover-status = k「状態」+ effective なら「発効・拘束力あり（承認 <approval の date>）」+ 承認欄へのリンク、draft なら「未承認のため拘束力なし → 持ち主の承認で発効」+ 承認欄へのリンク。
- 目次（toc）: 章ごとに n（00〜08・承認欄は「—」）・k（章の名）・t（その章の h2 の字面）。
- 00 北極星: h2「何のために・誰のために・何をあきらめるか」・lead「達成の判定: <north_star の judged_by>」。chapbody に section-lead-callout（--band-n:3）の card 3 枚: 「北極星（1 文）」= statement（card accent brand）／「誰のため」= for_whom／「迷ったらどちらへ倒すか（前文）」= precedence の text を ct に・plain を cd に・rationale を根拠の小窓（hint・各項を「<種別の名札>: <ref>」にして「／」で繋ぐ）に。
- 01 読み方: h2「<段の数> 段の意味 — <段の名を・で繋いだもの>」・lead = 語彙の field_terms のうち id が tier の行の def（無ければ lead を出さない）。card 3 枚（段の順）: cid「<段の名>（<原語>）」・class「card accent <色の class>」・ct = 意味・cd-req「<b>外すのに要るもの:</b> <外すのに要るもの>」（always と never は §6 への xref を含む）。
- 02〜04 段の章（正本の段の順）: h2「<その段の条の数> つの原則」・lead = 段の意味。chapbody の stack に、その段の条を正本の順で item-row として置く。item-row = article 要素・id は条 id の小文字・class「tier-<段の class の後半>」。中身は順に:
  - ir-head: rid = 条 id・h3 rt = title・badges = 段の小窓（hint tier・tier-badge・hint-body「この段 · <段の名>」+ 意味 + 「外すのに要るもの: 」+ 外すのに要るもの + §1 への xref）。
  - 規範文ごとに p.norm 1 つ: span（class ew）に規範文 id + 半角空白 + text + 半角空白 + span（class kw）に強度の名札。
  - div.plain: pk「やさしく言うと」+ plain。
  - p.meta-chips の小窓（在るものだけ・この順）: 「根拠」= rationale の各項を「<種別の名札>: <ref>」にして「／」で繋ぐ／「関係」= relations の reqs（`srs.html#<id の小文字>` へ「要件書 <id>（<要件書のその id の title>）」）・rules（`#<行 id の小文字>` へ「rules <id>」）・articles（`#<条 id の小文字>` へ「<id> <その条の title>」）・sections（`#s<n>` へ「§<n>」）を各群この順・群の中は正本の順で「・」で繋ぐ／「撤退条件」= 「<種別の名札>: <condition>」／「機構」= 「<kind の名札>・<live の名札>」+ stage と polarity が在れば「・<stage の名札>・<polarity の名札>」+ note が在れば「 — <note>」。
  - principle-amendment-history（supersedes_v1 か amended_by が在る条だけ）: am-kick「改訂来歴」+ supersedes_v1 は am-row に「<ruling> 」+ span（class am-meta）に「<doc> の <article> を置換（<rationale>）」+ amended_by の各項は am-row に「<adr> 」+ span（class am-meta）に「<date> · <ruling> · 承認 <approved_by>」（読む欄は adr・date・approved_by・ruling・previous_text・rationale の 6 つ＝便 6 の `link.rs` が数える欄と同じ・無ければ導出できない）。
  - details.machine（data-audience=machine・全条に置く）: dl = 「binds」<binds の値>／「mechanism」<kind> · live: <live>（stage と polarity が在れば · stage: <stage> · polarity: <polarity>）／「patterns」規範文ごとに「<id>: <pattern> / <strength>」を「 · 」で繋ぐ／「note」<note>（在る条だけ）。機械のための面の値は名札に写さず正本の値のまま。
- 05 数値の表: h2「数値と作法の表 — 閾値行 <R の数>・開発規律行 <D の数>」・lead = constitution の rules_pointer。chapbody に legend-line 2 本: 「状態: 」+ state の chip 3 つ（凍結 = 値は動かさない（変えるなら裁定が要る）／仮 = 値は入っているが確定前／未定 = まだ値が無い）／「種別: 」+ rules の schema の kind_meaning の各項を「<kind>: <意味>」で「／」で繋ぐ。次に tbl-wrap の表 2 つ。閾値行の表 = thead「id・何の数値か・値・種別・裁定・状態」・tr の id は行 id の小文字・td.id = id／td = what + 半角空白 + 条への xref「→ <article>」+ 在れば小窓（「注」= note・「母集団」= population・「射影」= projection・「根拠」= basis・「同じ種類」= same_failure・この順）／td.val = 値の読める形（便 11 の render の関数 val と同じ: 表は欄ごとに `<br>` 区切りで全角空白の字下げ + 太字のキー + 値・一覧は文字列の要素を「」で括り「・」で繋ぐ・null は `<code>null</code>`・それ以外は文字列化して escape）／td = kind の名札 + 「・」+ stage の名札／td = ruling + 「（<ruled_at>）」／td = state の chip（status の字面・色の class）。開発規律行の表 = thead「id・作法・種別・裁定・状態」・td.id / td = what + 条への xref + 在れば小窓「注」/ td = kind の名札 / td = ruling（ruled_at）/ td = state。
- 06 改訂: h2「変えるときの手続き — <段の数> 段」・lead = amendment の declaration。chapbody に figure-panel（id fig-amend-flow・data-role diagram）: fig-title = fn「図 1」+「憲法が変わるときに通る道 — <段の数> 段・誰がやるか」+ fig-tools（凡例の小窓 = fig-legend の sw 3 つ「持ち主の手番」warn・「AI・機械がやる」neutral・「番号の順に進む」line／zoom-btn「拡大」／zoom-close「✕ 閉じる」）。図の本体は pipeline-rail（ol・style は --rail-n:<段の数>）で、段ごとに li.rail-col: rail-step の no = n、rail-slot 2 つ（担当が持ち主なら上の枠に rail-node〔tone-warn〕・下の枠に lane-hint「機械は待つ」／それ以外は上の枠に lane-hint「あなたの出番なし」・下の枠に rail-node）。rail-node = article 要素・id a-step-<n>・actor「担当: <who>」・p.nt = what の最初の「 — 」より前（「 — 」が無ければ what 全体）+「 — 」より後が在れば小窓「?」にその後ろ・p.fig-reqs = article 欄の各条 id を a（class fig-req・href はその条の anchor）にして、中は span（class no）に条 id + その条の title。figcaption の ver = 「図 1 · <version> <generated> · constitution.yaml」。段の数が 7（部品目録の pipeline-rail の max_nodes）を超えれば導出できない。図の後に stepper（ol）: 最初の li は effective_step（no は「0」・body の b = what の「 — 」より前 + who の chip〔class who・持ち主なら owner を足す〕・b の後に「 — 」より後 + 「（」+ article の各条 id への xref を「・」で繋ぐ + 「）」）、続けて steps を同じ形で。次に amendment-example を supersedes_v1 を持つ条ごと・amended_by の項ごとに置く（正本の順）: supersedes_v1 = ax-k「改訂の例 — <条 id>（<ruling>）」・ax-text = lbl「消す文」+ del に <article>・`<br>`・lbl「足す文」+ ins に「<条 id>: <title>」・dl = 「出所」<doc>／「裁定」<ruling>／「理由」<rationale>／「撤退条件」<その条の retreat の condition>（在れば）。amended_by = ax-k「改訂 — <条 id>（<adr>）」・del に previous_text・ins に現行の条への xref「現行の <条 id>」・dl = 「判断の記録」<adr>／「承認」<approved_by>（<date>）／「裁定」<ruling>／「理由」<rationale>。最後に details.note を meta の changes_from_ で始まる欄ごとに（欄の名の昇順・便 11 と同じ）: summary「<接頭辞を除き _ を . に替えた版> からの変更（<件数> 件）」+ ul の li に各項。
- 07 用語集: h2「本文に出てくる専門語のやさしい説明」・lead = constitution の glossary_pointer。chapbody に glossary-term-table: 語ごとに div.grow（id g-<語の id>）= gword に term（en が null でなければ span〔class en〕に en を添える）・div に p.gdef = def（note が在れば 2 つ目の p.gdef に）+ a（class back・href は目次の anchor）「目次へ」。field_terms と identifiers は出さない（機械層）。
- 08 出所: h2「この憲法はどこから来たか」・lead 無し。chapbody に section-lead-callout（--band-n:<出所の数>）の card = cid「出所 <n>」・ct = name・cd = ref。
- 承認欄: chapter-deck-band（slim・kicker「承認欄」・h2「作成 / 承認」・lead = 文書の状態の名札）。chapbody に approval-block の sign 2 つ: 「作成」= who <meta の author>・when <meta の generated>・stamp self「起草」／「承認」= who <approval の who>・when「<approval の date> · 逐語「<verbatim>」」・2 つ目の when「裁定: <ruling>／対話面: <surface>」・stamp = effective なら stamp「発効」・draft なら stamp todo「未承認」。
- foot: ft-plain「このページは正本 constitution.yaml から folio が生成した · 憲法 <version>（<generated>）· 手で直さない」+ details.machine の dl = id / version / status / binding / baseline / items（条 id 全部を「 / 」で繋ぐ）。doc-locator「この文書の所属: 設計文書（design-intent）/ 憲法 — <入口へのリンク「入口へ戻る」>」。
- 部品の名札（属性 data-component の値）は便 13 の `parts.rs` の catalog の Component の関数 name からだけ出す（生成器の中に名札の字面を書かない＝目録に無い部品を書けない作り・ADR-5 決定 (3)）。本便の憲法面が使う部品は 14 種: freshness-stamp・font-size-control・doc-cover-band・chapter-deck-band・section-lead-callout・item-row・principle-amendment-history・figure-panel・pipeline-rail・rail-node・stepper・amendment-example・glossary-term-table・approval-block。見本が使う lane-chip は出さない（読む順番の札の行き先は入口の正本の stops に節の id が無いので出せない・入口面の便で足す）。
- 面の中のリンクの先: 同じ面の anchor（上の id の付け方）・`index.html`・`srs.html#<要件 id の小文字>`（要件書の面は後続の便がこの id を持つ）。

(c) 命令の口（`crates/folio/src/face.rs`）: `--face` の解決・4 file の読み（便 7 の型付きの読み手 parse_typed・重複キーは parse の duplicates で数え 1 つでも在れば導出できない・空の文書も導出できない）・escape・値の読める形・3 値の判定と文言。正本の木を辿る口は便 11 の `render.rs` の型 X と同じ形（欄の道つき・無い欄と型違いを文言つきで Err に）を本 file に持つ（`render.rs` の X は非公開で、`render.rs` は行数の上限に近いので触らない。写しの畳み込みは day-1 の読み物を退役する便で行う）。(b) の名札の表（β）と escape・値の読める形・小窓の組み立てのような共通の口は `face.rs` の側に置いてよく、`face_constitution.rs` は憲法面の骨格と各章の導出だけを持つ（2 file の分担は runner に任せる・上限は write-set の 2 file のどちらも新規なので掛からない）。

(d) 導出できない = 2「まだ分からない」で出力先に 1 byte も書かない（P-4.1・ここに挙げたものだけ）:
- 4 file のどれかが無い・読めない・UTF-8 でない・重複キー・空の文書（注釈だけの file を含む）
- 読む欄が無い・型が違う（読む欄はすべて必須: meta の id・version・status・generated・author・counts・approval〔who・date・ruling・verbatim・surface〕／north_star の statement・for_whom・judged_by／precedence の text・plain・rationale／schema の enums の tier／articles の各条の id・title・tier・binds・statements〔各 id・pattern・strength・text〕・plain・rationale〔各 kind・ref〕・mechanism〔kind・live〕／amendment の declaration・steps〔各 n・who・what・article〕・effective_step〔同じ 4 欄〕／rules_pointer／glossary_pointer／sources〔各 n・name・ref〕／rules の schema の kind_meaning・thresholds〔各 id・article・what・value・kind・status・ruling・ruled_at・stage〕・discipline〔各 id・article・what・kind・status・ruling・ruled_at〕／vocabulary の terms〔各 id・term・en・def〕／srs の goals・requirements・nonfunctional・acceptance・constraints〔各 id・title〕）。任意の欄（relations・retreat・supersedes_v1・amended_by・note・changes_from_*・rules 行の basis 等・語彙の note・field_terms）は在るときだけ読み、在れば型と中の欄を同じく確かめる
- 名札の表に無い値（tier・binds・pattern・strength・機構の kind・live・stage・polarity・rationale の kind・retreat の kind・meta の status・rules 行の status と kind と stage）
- meta の counts のキーの集合が段の一覧と違う・値が数えた数と違う
- 参照の先が無い（relations の reqs → srs の 5 節の id・rules → rules の行 id・articles と steps の article と effective_step の article → 条 id・sections → 「§」+ 0〜8 の数字 1 つ）
- 改訂の段の数が 7 を超える
- id と href に使う正本の id が英数字と「-」「.」以外を含む
- `--out` の親 dir が無い

(e) 歯 `crates/folio/tests/face.rs`（binary 経由・歯の関数名はすべて face を含める＝verify の filter 語）。base で名に face を含む歯は `crates/folio/tests/parts.rs` の 3 本（parts_check_passes_on_the_three_sample_faces・parts_check_fails_on_a_component_not_allowed_on_the_face・parts_check_is_unknown_when_the_face_name_is_not_one_of_three）だけで、`src/` の unit の歯には無い（実測）＝`tests/parts.rs` は write-set に在るが本文も期待も変えない。
- 凍結 fixture との byte 一致（P-10.1）: `--face constitution --dir tests/fixtures/face --out <一時 file> --write` = 終了 0 ∧ 標準出力「書いた」∧ 出力が `tests/fixtures/face/expected.html` と byte 一致。
- 実の正本で部品目録に合う（AC2 の機構）: `--dir design-intent` で一時 file へ `--write` = 0 → `folio parts --check --dir design-intent --page constitution=<一時 file>` = 終了 0 ∧ 標準出力「違反 0」。
- 逐語と件数の census（実の正本・歯は folio の読み手を経ず yaml-rust2 で正本を直に読む＝生成側から独立）: 出力に 全条の title・plain・全規範文の text（escape 後の字面）が含まれる／item-row の名札の出現数 = 条の数／rail-node の出現数 = 改訂の段の数／grow の出現数 = 語彙の terms の数／id が r- で始まる tr の数 = 閾値行の数・d- で始まる tr の数 = 開発規律行の数／data-component の値はすべて部品目録の 14 種の中で lane-chip を含まない。
- check の 3 値: fixture の写しに `--write` した後 `--check` = 0 ∧「OK」／出力先の 1 byte を変えて `--check` = 1 ∧「DRIFT」／出力先を消して `--check` = 2 ∧「面が無い」／その後 `--write` = 0 → `--check` = 0。
- 面の名: `--face index` = 2 ∧「生成器はまだ無い」∧ 出力先が出来ていない／`--face foo` = 2 ∧「どれでもない」。
- 導出できない入力 8 つ（fixture の写しに変異を 1 つずつ・どれも `--write` = 2 ∧ 標準エラー「まだ分からない」∧ 出力先が出来ていない）: 条の tier を表に無い値に／meta の counts の値を 1 ずらす／relations の articles に無い条 id を足す／steps の article に無い条 id を足す／rules.yaml の行の status を表に無い値に／srs.yaml を消す／constitution.yaml に重複キーを足す／`--out` の親 dir が無い path。
- escape: fixture の語彙の def に「<b>」「&」「二重引用符」を含む文を置き、出力に `&lt;b&gt;`・`&amp;`・`&quot;` が現れ、生の `<b>` が現れない。
- unit の歯（`src/face_constitution.rs` と `src/face.rs` の中・名に face を含む）: 段の表引き（3 値と表に無い値の Err）・「 — 」での分割（在る・無い）・sections の解釈（§6 → s6・§9 と「§」無しは Err）・使う部品 14 種の faces に constitution が入っている（Component の関数 faces で確かめる）・改訂の段の上限 7 が部品目録 `design-intent/preview/parts.json` の pipeline-rail の max_nodes と同値。
- 独立した最小の凍結 fixture（P-10.1・手書き・新しい置き場 `tests/fixtures/face/`）: constitution.yaml（schema の enums の tier 3 つ・meta〔status draft・binding false・counts 1/1/1・approval〕・north_star・precedence・条 3 つ〔段ごとに 1 つ・P-1 は規範文 2 つ〔must と must-not〕・relations〔reqs FR1・rules R-1・articles A-1・sections §6〕・retreat・supersedes_v1・note を持つ／A-1 と N-1 は必須の欄だけ〕・amendment〔steps 2 つ = AI と 持ち主・effective_step〕・rules_pointer・glossary_pointer・sources 1 つ・changes_from_v0_1 1 項）・rules.yaml（schema の kind_meaning・閾値行 1〔値は文字列〕・開発規律行 1）・vocabulary.yaml（terms 2 つ〔1 つは def に「<b>」「&」「二重引用符」を含む〕・field_terms に id tier の行）・srs.yaml（goals 0・requirements に FR1・他の 3 節は空の一覧）・expected.html（上の正本から (b) の規則で組んだ面・最初の 1 回は生成器の出力を写して置いてよいが、置いた後は歯が固定する期待であり、変えるときは判断の記録の帰結として直す）。独立性の分担（P-10.1・P-10.2）: 生成側からも検査側からも独立した物差しは 部品目録と様式の定義を読む `folio parts --check`（便 13 の検査器）と yaml-rust2 で正本を直に読む census の 2 つで、expected.html は回帰の anchor（生成物どうしの突き合わせ）であり唯一の判定にしない。4 つの正本は各 30〜60 行の最小形で、`design-intent/` の写しにしない（gate の diff の上限・便 1 の教訓）。

(f) 便 13 までの形との接続: 新規は `crates/folio/src/face.rs`・`crates/folio/src/face_constitution.rs`・歯 `crates/folio/tests/face.rs`・fixture 5 本。`crates/folio/src/main.rs` で変えるのは `mod face;` と `mod face_constitution;` の 2 行と、副命令 Face の枝（引数の型と、Outcome の標準出力・標準エラーを 1 行ずつ出して終了コードを返す部分・便 11 の Render の枝と同じ形）だけで、他の副命令の枝は 1 字も変えない。使う既存の口は `yaml.rs` の parse と parse_typed と Value（py_str・as_map・as_seq・as_str・get）・`verdict.rs` の Verdict・`parts.rs` の catalog の Component（関数 name と faces・`pub mod catalog` で既に crate の中から呼べる）と定数 FACES で、どれも可視性の変更は要らず、`yaml.rs`・`verdict.rs`・`parts.rs`・`render.rs`・`check.rs`・`entrance.rs`・他の src と `build.rs` は中身を変えない。`folio check` に面の検査を組み込まない（凍結 fixture の 134 case と既存の fixture は preview/ を持たない）。`design-intent/` の下（正本・見本 3 面・folio.css・parts.json）・`scripts/`・`.github/workflows/`・`Cargo.toml`・`Cargo.lock` は触らない（外部 crate は増えない）。正規表現は使わない。size は S = 中身を変える既存の file（main.rs）1 本あたりの増分の見積で、新規の file は増分に数えない。生成した面を `design-intent/preview/constitution.html` へ書く（見本の置き換え）のは本便の外（A-1・退役は可逆な移動 N-1.2）。

## 2. 範囲

- 入れる: 副命令 `folio face`（write / check・3 値）・憲法面の生成器（部品目録の閉じた一覧だけを使う）・歯（凍結 fixture との byte 一致・実の正本で parts --check 合格・census・check の 3 値・面の名・導出できない入力・escape・unit）・独立した最小の凍結 fixture。
- 入れない: 入口の面と要件書の面の生成（後続の便）・読む順番の札（lane-chip・入口の正本の stops に節の id を足す便で）・規範文の中の用語の小窓（hint term）と読者の札（reader-chip）・見本 3 面の置き換えと退役（A-1）・`folio check` への組み込み・CI の段の変更・組み立てと配信（build / serve・N-6）・密度 profile・field_terms と identifiers の表示・持ち主の walk 承認（3 面が揃ってから）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cmd | 命令の口 | --face の解決・正本の読み・3 値と文言・書く / 比べる |
| labels | 名札の表 | 段・型・強度・機構・種別・章・状態の閉じた表（表に無い値は導出できない） |
| page | 面の導出 | 見本の骨格に正本の値を差し込み、部品の名札は Component からだけ出す |
| fixture | 凍結 fixture | 最小の正本 4 file と期待の面 1 枚 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 13 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。歯の census は yaml-rust2（既存の依存）で正本を直に読む。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "o"
title = "憲法の面を正本から folio face で生成し、folio parts --check で目録外 0 を固定する"
req = ["FR4", "NFR2"]
section = "1"
write-set = ["+crates/folio/src/face.rs", "+crates/folio/src/face_constitution.rs", "+crates/folio/tests/face.rs", "crates/folio/src/main.rs", "crates/folio/tests/parts.rs", "+tests/fixtures/face/constitution.yaml", "+tests/fixtures/face/rules.yaml", "+tests/fixtures/face/vocabulary.yaml", "+tests/fixtures/face/srs.yaml", "+tests/fixtures/face/expected.html"]
verify = ["cargo nextest run -p folio face", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face の歯（凍結 fixture との byte 一致・実の正本で parts --check 合格・逐語と件数の census・check の 3 値・面の名 2 つ・導出できない入力 8 つ・escape・unit）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
