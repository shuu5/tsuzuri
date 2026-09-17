# 設計: 便 15 — 要件書の面を正本から `folio face --face srs` で生成し、憲法面と共有する口を `face.rs` に寄せる

- 要件: FR4（3 面を単一の生成器と単一の design token で出す）/ NFR2（見た目の材料は 1 か所から・部品目録に無い class は rules 行 R-3 の値 0）
- 条: P-2.1・P-2.4（人が読むページは 1 つの生成器から・部品は閉じた一覧）/ P-6.1・P-6.2・P-6.3（正本は YAML・生成物を手で直さない・同じ内容は一方を正本に他方を導出）/ P-4.1・P-4.2（実行できなかった生成を異常なしにしない・判定できないものは「まだ分からない」）/ P-10.1・P-10.2（独立した凍結 anchor・生成物どうしの突き合わせを唯一の判定にしない）
- 判断の記録: ADR-5（決定 (1) 見た目は見本・文章と数は正本／決定 (3) 手書きの Rust・部品目録を型で閉じる／決定 (4) 図は既存 3 型だけ／決定 (5) 順序 = 憲法面 → 要件書面 → 入口面）。便 14（f2-648.25）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」（f2-648 notes・3 面 → walk 承認 → 組み立てと配信の順序）。図 3（検査の答えは 3 つ）の正本は要件書 v1.2 に無く、持ち主の裁定 2026-09-17（f2-648 notes・逐語「Aですすめてよい」）= 要件書を v1.3 に上げて verdicts の節を足す。節の追加は本便の着地の後に planner の別 PR（承認の行に逐語）で行い、本便は床の受け皿（節の閉じた一覧に verdicts を足す）と生成器（節が在れば図 3 を出し、無ければ出さない）を運ぶ。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 p が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

見本 3 面のうち要件書の面（`design-intent/preview/srs.html`・2026-09-12 に持ち主が見た目を承認した手書きの HTML）を、正本から機械で生成する。便 14 の副命令 `folio face` に面 srs の生成器（`crates/folio/src/face_srs.rs`）を足し、憲法面（便 14）と同じ規律で組む: 生成物の文字列は (α) 正本の欄の値を escape して逐語で差し込んだもの／(β) 生成器が閉じた表として持つ名札／(γ) 正本から数えた数 の 3 種だけで、見本にしか無い案内の文（表紙の副題・読者の札・各章の lead と補足・読む順番の札〔lane-chip〕・図の中の説明の小窓・対応表の補足）は正本に同じ内容の欄が在ればその欄から出し、無ければ出さない。部品の名札は便 13 の閉じた一覧（`parts.rs` の catalog の Component）からだけ出す。受入の機構は便 14 と同じ 2 つ: 生成した面を `folio parts --check` に掛けて目録外 0（AC2）と、独立した最小の凍結 fixture との byte 一致（P-10.1）。併せて、憲法面の生成器の中に在る「面に依らない口」（head と site-bar・章の帯・card・foot・部品の名札の出し方）を `face.rs` へ寄せ、2 つの面が同じ関数で組む（P-6.3 の趣旨・見た目の直しが 1 か所で済む）。寄せた後の憲法面の出力は 1 byte も変わらない（凍結 fixture の期待 `tests/fixtures/face/expected.html` と実の正本の census が固定する）。

読む正本は 4 file: `srs.yaml`（本体）・`constitution.yaml`（根拠の条の見出しと rules 行の実在）・`rules.yaml`（rules 行 id の実在）・`vocabulary.yaml`（§8 の用語の一覧）。判断の記録・凍結 anchor・入口の正本・部品目録の実行時の写しは読まない。

planner の実測（2026-09-17・main ae9bd12）: 要件書 v1.2 の goals 4・actors 5（role が「道具」の行は folio-v2 の 1 つ）・outputs 3（from は FR1・FR4・FR5）・rail 7（who は 持ち主 2・folio 5・段 3 は reqs が空で basis に P-1 と P-12・段の what に「 — 」は 0）・requirements 15・nonfunctional 3・acceptance 13（milestone M1 が 7・note を持つのは AC5）・constraints 9（rules を持つのは CON5・CON6・basis が空なのは CON7）・meta の counts は fr 15 / nfr 3 / ac 13 / con 9 で数えた数と同値・meta の approval は 7 行（役は 作成 3・レビュー 1・承認 3・承認の 3 行は verbatim と version を持つ）・scope の build 6 / not_build 4・scope_m1 の build 5 / not_build 4 / note 1・not_frozen 1 文・sources 2・glossary_pointer 1。要件と非機能要件 18 の pattern は event 10・ubiquitous 6・unwanted 1・state 1、strength は must 15・should 3、verify の method は test 16・test+inspection 2、milestone M1 が 7、rules を持つのは 7、note を持つのは 10、verify の ac が空なのは 5。figures の値の種類は 図2-1・図2-2・図2-4・図2-5・図2-6・図2-7・図1・図3・全段 の 9 種（図3 は FR5 だけ）。goals の参照先・verify の ac の参照先・basis の条・rules 行・acceptance の verifies・outputs の from・rail の reqs と basis は全部実在（便 1 の床 R-4 が 0 で通っている）。要件・受入・制約の見出し・規範文・平易文に「<」「>」「&」「二重引用符」を含むものは 0。見本の要件書面の部品の名札は 18 種で、うち lane-chip（4）は出さず、rtm-grid・glossary-links・ac-state-chip・context-band・band-node・state-strip・state-node は本便で初めて生成器が出す。verdicts の節は要件書 v1.2 に無い（実測 0）。

(a) 命令の形: 便 14 の `folio face --face <面の名> --dir <正本の置き場> --out <出力先> --write|--check` のまま。`--face srs` で本便の生成器へ分岐し、`--face index` は引き続き「面「index」の生成器はまだ無い」（終了 2）。終了コード・文言・`--out` に既定が無いこと・導出できないときに 1 byte も書かないことは便 14 の (a) と同じ（`face.rs` の run を変えない）。

(b0) 床の受け皿（`crates/folio/src/check.rs`）: 要件書の節の閉じた一覧 SRS_TOP_LEVEL（14）に verdicts を足して 15 にし、要件書の検査（同 file の関数 check_srs）で verdicts の行を goals と同じ形で数える（行の一覧・各行の id・name・tone・cond の非空・id の重複は要件書の他の節の id と合わせて 1 つの集合で数える）。tone の値域は床では数えない（面の生成器の表引きが「まだ分からない」に倒す）。節が無い要件書（今の v1.2・凍結 fixture の 134 case・便 0 の fixture）は変わらず合格し、未知の節の fixture `tests/fixtures/check/unknown-section/srs.yaml` の節の名は extra_section なので期待は変わらない（実測）。`refs.rs`（R-4）は触らない: R-4 の走査は要件書の全節を歩く既存の形のままなので verdicts の文字列（cond）も走査されるが、verdicts の id は解決先（5 節の id）に入らず誰からも参照されない（v1.3 の cond に書く id 形の語は実在する id だけ）。`vocab.rs`（R-9）の母集団は節を名指すので verdicts は外。

(b) 共有する口の寄せ（`face.rs` に足し、`face_constitution.rs` から同じ字面を出す）:
- 面の骨格の型: 面ごとの固定値（面の名〔title と crumb と here に出す〕・favicon の字〔憲 / 要〕・nav の aria-current の位置・章の数・章ごとの帯の class と kicker の絵記号・prevnext の前後）を 1 つの表の型で持ち、head と site-bar（skip-link から main の開始まで）・章の帯（section の chapter-deck-band・crumb・kicker・h2・任意の lead）・card・toc の li・foot（prevnext・ft-plain・機械のための面の dl・doc-locator・body と html の閉じ）を面の値を引数に取る関数にする。
- 部品の名札の出し方: 面が使う部品の一覧（Component の配列）を引数に取り、一覧に無い部品は組み立て時に落ちる（今の憲法面の関数 dc と同じ・debug の確かめ）。
- 寄せた後の憲法面: `face_constitution.rs` は憲法面の一覧 PARTS・章の名と h2・各章の中身だけを持ち、上の関数を呼ぶ。出力は寄せる前と 1 byte も変わらない（(e) の歯）。
- 便 14 の名札の表（STRENGTH・PATTERN・DOC_STATUS 等）はそのまま使う。本便で足す表は (c) に列挙する。

(c) 要件書の面の組み立て（`face_srs.rs`）。骨格は見本と同じ順: head（title「folio2 — 要件書（<版>）」・favicon は見本と同じ data URI〔要の字〕・folio.css と folio-ui.js）→ skip-link → site-bar（nav は要件書に aria-current・here「要件書 ▸ 全 9 章 — 目次へ」・freshness-stamp「生成 <生成日> · <版>（<状態の名札>）」）→ main → doc-cover-band → toc → 章 01〜08 と承認欄 → prevnext（前 = 憲法 constitution.html・次 = 入口 index.html）→ foot → doc-locator。見本の layer-line と reader-chip は出さない。字下げ無し・部品ごとに改行 1 つ・末尾に改行 1 つ・escape と id の規則は便 14 と同じ（要件 id の anchor は小文字: FR1 → fr1・GOAL1 → goal1・AC1 → ac1・CON1 → con1）。
- 名札の表（β・本便で足すもの・表に無い値は導出できない）:
  - 確かめ方（verify の method）: test「実際に動かして確かめる（Test）」・inspection「目で見て確かめる（Inspection）」・test+inspection = 前 2 つを「 + 」で繋ぐ。
  - 強度の色の class（prio）: must と must-not = must・should = should（見本の css の 2 つ）。
  - 図の色（tone・verdicts の節と rail の担当）: ok・bad・neutral・warn の 4 つ（class は tone-<値>）。
  - 段の担当（rail の who）: 値が「持ち主」なら持ち主の手番（tone-ok・上の枠・下の枠の lane-hint「folio は待つ」）、それ以外は道具の手番（上の枠の lane-hint「あなたの出番なし」・下の枠）。値はそのまま「担当: <who>」に出す。
  - 章: 01「ゴール」/ 02「範囲」/ 03「機能要件」/ 04「非機能要件」/ 05「受入基準」/ 06「制約」/ 07「対応表」/ 08「用語集」/ 承認欄「承認欄」。帯の class は見本と同じ（01 band-1・02 band-5・03 band-2・04 band-6・05 band-2・06 band-3・07 band-5・08 band-1・承認欄 band-3 slim）。kicker の絵記号は見本の各章の path の字面。crumb は「要件書 › <NN> <章の名> <i>/9」。
  - 図の参照（要件の figures の値）の解き方: 「図2-<n>」→ 段 n の rail-node への a（class rq-where・href #rail-<n>・字面「図 2 <n> <その段の what>」）／「図1」→ #fig-context「図 1」／「図3」→ verdicts の節が在れば #fig-verdicts「図 3」・無ければリンク無しの字面「図 3」／「全段」→ 字面「全段に掛かる」／それ以外は導出できない。「図2-<n>」の n が rail に無ければ導出できない。
  - 受入基準の状態の chip（ac-state-chip）: folio はまだ受入の結果を測らないので全部「まだ分からない」（P-4.2）。
- 表紙（doc-cover-band）: cover-eyebrow = doc-type「要件書 (SRS)」+「folio2 — <meta の title>」／h1 = meta の title／summary-card = ic「要」・lab「この文書が約束すること（1 文）」・txt = meta の promise／cover-meta = 「機能要件」<数> 件（<最初の id>–<最後の id>）→ #s3・「非機能要件」→ #s4・「受入基準」→ #s5・「制約」→ #s6・「図」= 図 1（#fig-context）· 図 2（#fig-rail）· 図 3（#fig-verdicts・verdicts の節が在るときだけ）・「版」<version> / <generated>／cover-status = k「状態」+ effective なら「発効・拘束力あり（承認 <承認の最後の行の when>）」+ 承認欄へのリンク、draft なら「未承認のため拘束力なし → 持ち主の承認で発効」+ 承認欄へのリンク。meta の counts（fr・nfr・ac・con）は数えた数と一致しなければ導出できない。
- 目次（toc）: 章ごとに n（01〜08・承認欄は「—」）・k（章の名）・t（h2 の字面）。
- 01 ゴール: h2「「これができたら成功」を先に決める」・lead 無し。callout（--band-n:<goals の数>）の card（class「card accent」・id = goal の anchor）= cid「<id>」・ct = title・cd = text。
- 02 範囲: h2「誰が登場し、何を作り、何を作らないか」・lead 無し。figure-panel（id fig-context・data-role diagram）: fig-title = fn「図 1」+「誰が使い、何が出るか」+ fig-tools（zoom-btn「拡大」・zoom-close「✕ 閉じる」・小窓は出さない）。図の本体は context-band（--band-n:3）: band 1（h4「入れる側」）に role が「道具」でない actor を正本の順で band-node（id c-<actor の id>・p.nt = name・span〔class edge〕に span〔class arrow〕+ role）／band-arrow／band 2（class「band tool」・h4「道具」）に role が「道具」の actor を同じ形で（edge は出さない）／band-arrow／band 3（h4「出る側」）に outputs を正本の順で band-node（id c-<id>・p.nt = name・p.fig-reqs に from の要件への a〔class fig-req・span〔class no〕に id + その要件の title〕）。role が「道具」の actor が 0 か 2 以上なら導出できない。入れる側（道具でない actor）と出る側（outputs）の各帯が 4（部品目録の context-band の max_per_band）を超えれば導出できない。figcaption の ver = 「図 1 · <version> <generated> · srs.yaml」。図の後に h3「作るもの / 作らないもの」と callout（--band-n:2）2 段: 1 段目 = card accent ok cid「M0 で作る」cd = scope の build を「 ／ 」で繋ぐ・card accent cid「M0 では作らない」+ span〔class pill〕「対象外」cd = not_build を「 ／ 」で繋ぐ／2 段目 = 同じ形で scope_m1（cid「M1 で作る」「M1 では作らない」・note が在れば 2 枚目の cd の後に小窓「注」）。
- 03 機能要件: h2「<requirements の数> の機能要件 — いつ・何をするか」・lead 無し。chapbody の先頭に legend-line「凡例:」+ 小窓「言葉」（強度の名札 3 つ「<原語の大文字> = <意味>」〔must「必ず守る」・must-not「決してしない」・should「強い推奨（外すなら理由が要る）」〕と型の名札 5 つを「／」で繋ぐ）+ 小窓「確かめ方」（確かめ方の名札 2 つ）。次に figure-panel（id fig-rail）: fig-title = fn「図 2」+「folio が 1 回で通す <rail の数> 段 — 各段を決めている要件」+ fig-tools（小窓「凡例」= fig-legend の sw 3 つ「あなたの手番」ok・「folio がやる」neutral・「番号の順に進む」line + lg「枠線が点線 = 定める要件が無い段」／zoom-btn／zoom-close）。図の本体は pipeline-rail（--rail-n:<段の数>・7 を超えれば導出できない）で、段ごとに li.rail-col: rail-step の no = n、rail-slot 2 つ（担当の表のとおり）。rail-node = article・id rail-<n>・actor「担当: <who>」・p.nt = what（note が在れば小窓「?」に note）・p.fig-reqs = reqs の各 id への a（class fig-req・span〔class no〕に id + その要件の title）。reqs が空なら span（class「fig-req none」）に「この段を定める要件は無い」+ basis が在れば「（<憲法の各条への a〔class xref・href constitution.html#<条の anchor>・字面「憲法 <id>」〕を「・」で繋ぐ〉）」。figcaption の ver。次に stack: requirements を正本の順で item-row（id = anchor・class 無し）。item-row の中身は順に:
  - ir-head: rid = id・h3 rt = title・badges = span（class「prio <色の class>」）に強度の名札 + span（class ears）に型の名札 + milestone が在れば span（class pill）に milestone の値。
  - p.norm: pattern が ubiquitous なら shall だけ・それ以外は span（class ew）に when + 半角空白 + shall。
  - div.plain: pk「やさしく言うと」+ plain。
  - p.meta-chips の小窓（在るものだけ・この順）: 「確かめ方」= span（class vbadge）に method の名札 + 半角空白 + how + ac が空でなければ「（」+ 各 AC への a〔class xref・href #<anchor>・字面 id〕を「・」で繋ぐ +「）」／「根拠」= goals の各 id への a（href #<anchor>・字面「<id>（<title>）」）と basis の各条への a（href constitution.html#<anchor>・字面「憲法 <id>（<条の title>）」）と rules の各行への a（href constitution.html#<anchor>・字面「rules <id>」）をこの順で「・」で繋ぐ／「注」= note。小窓の後に figures の各値を図の参照の解き方で（a〔class rq-where〕か字面）。
  - details.machine（data-audience=machine）: dl = 「pattern」<pattern>／「strength」<strength>／「when」<when>／「verify」<method>／「milestone」<milestone>（在れば）。値は正本のまま。
  stack の後、verdicts の節が在れば figure-panel（id fig-verdicts）: fig-title = fn「図 3」+「検査の答えは <数> つ — どういうときにその答えになるか」+ fig-tools（小窓「凡例」= 各 verdict の sw〔class sw <tone>〕+ name／zoom）。図の本体は state-strip（--state-n:<数>・4〔部品目録の state-strip の max_nodes〕を超えれば導出できない）の state-node（li・id v-<id>・class tone-<tone>・p.nt = name・span〔class cond〕「ここへ来る条件」・p.np = cond）。figcaption の ver「図 3 · …」。verdicts の節の形は 一覧・各行に id・name・tone・cond の 4 欄（無い欄・表に無い tone は導出できない）。
- 04 非機能要件: h2「<nonfunctional の数> の非機能要件 — 数で測れる約束」・lead 無し。legend-line は 03 と同じ字面。stack に nonfunctional を item-row（class kind-nfr）で 03 と同じ形。
- 05 受入基準: h2「<acceptance の数> の受入基準 — 何を見せられたら「できた」か」・lead 無し。callout（--band-n:2）の card（class「card accent」・id = anchor）= cid「<id> 」+ span（data-component は ac-state-chip）「まだ分からない」+ milestone が在れば span（class pill）に値・ct = title・div.plain（pk「やさしく言うと」+ plain）・p.cd = verifies の各要件への a（class xref・href #<anchor>・字面 id）を「・」で繋ぎ「 を確かめる。」+ 小窓「RED の歯」= red_test の sentence +「／fixture: 」+ fixture + note が在れば小窓「注」。card の後に callout（class「callout warn」）: span（class ck）「凍結しないもの」+ p = not_frozen。
- 06 制約: h2「設計の前に決まっていること」・lead 無し。tbl-wrap の表: thead「id・制約・中身・根拠・出所」・tr の id = anchor・td.id = id／td = title／td = text／td = basis の各条への a（href constitution.html#<anchor>・字面 id）と rules の各行への a を「・」で繋ぐ（両方空なら「—」）／td = source。
- 07 対応表: h2「どの要件が、どのゴールのためにあり、どの受入基準で確かめ、図のどこにあるか」・lead 無し。legend-line「凡例:」+ 小窓「読み方」（「● = この要件はそのゴールのためにある ／ AC = 受入基準で確かめる ／ — = 受入基準では直接確かめない」）。rtm-grid の table（class rtm）: thead = th「要件」+ goal ごとに th（class grp）「<id> <title>」+ th「受入で確かめる」+ th「図のどこ」。tbody = requirements と nonfunctional をこの順で tr: th に a（href #<anchor>・id）+ span（class lbl）に title／goal ごとに td（その要件の goals に在れば class hit + span〔class dot〕「●」・無ければ空）／td = verify の ac が空でなければ class hit + 各 AC を span（class「dot ac」）に id・空なら「—」／td = figures を図の参照の解き方で（a は class fig・href は同じ・字面は「図 2 <n>」「図 1」「図 3」・全段は字面「全段」）を「 · 」で繋ぐ（figures が空なら「—」）。
- 08 用語集: h2「この文書に出てくる専門語 → 説明は憲法 §7 で」・lead = glossary_pointer。glossary-links に語彙の terms を正本の順で span（class hint）: a（href constitution.html#g-<id>）に term + en が在れば span（class en）に en ／ label と checkbox の小窓（hint-btn q「?」）／hint-body = short + `<br>` + a（href 同じ）「憲法 §7 の定義へ」。
- 承認欄: 帯（slim・kicker「承認欄」・h2「作成 / レビュー / 承認」・lead = 文書の状態の名札 + status_note が在れば「 — <status_note>」）。approval-block に meta の approval の各行を正本の順で sign: role・who・when・version が在れば span（class when）「版 <version>」・verbatim が在れば span（class when）「逐語「<verbatim>」」・stamp = role が「承認」なら span（class stamp）に stamp の値・それ以外は span（class「stamp self」）に stamp の値。
- foot: ft-plain「このページは正本 srs.yaml から folio が生成した · 要件書 <version>（<generated>）· 手で直さない」+ details.machine の dl = id / version / status / effective_version（在れば）/ items（要件・非機能要件・受入基準・制約の id 全部を「 / 」で繋ぐ）。doc-locator「この文書の所属: 設計文書（design-intent）/ 要件書 — 入口へ戻る」。
- 本便の要件書面が使う部品は 17 種: freshness-stamp・font-size-control・doc-cover-band・chapter-deck-band・section-lead-callout・figure-panel・context-band・band-node・pipeline-rail・rail-node・state-strip・state-node・item-row・ac-state-chip・rtm-grid・glossary-links・approval-block。lane-chip は出さない（便 14 と同じ理由）。
- 面の中のリンクの先: 同じ面の anchor・`index.html`・`constitution.html#<条 / rules 行 / 語彙の anchor>`（便 14 の憲法面がこの id を持つ）。

(d) 導出できない = 2 で出力先に 1 byte も書かない（便 14 の (d) に足すもの・ここに挙げたものだけ）: 4 file の読めない・重複キー・空の文書（便 14 と同じ）／読む欄が無い・型が違う（meta の id・title・version・status・generated・promise・counts〔fr・nfr・ac・con〕・approval〔各 role・who・when・stamp〕／goals〔各 id・title・text〕／scope と scope_m1〔build・not_build〕／actors〔各 id・name・role〕／outputs〔各 id・name・from〕／rail〔各 n・who・what・reqs〕／requirements と nonfunctional〔各 id・title・pattern・strength・when・shall・plain・basis・verify〔method・how・ac〕・goals・figures〕／acceptance〔各 id・title・plain・verifies・red_test〔sentence・fixture〕〕／not_frozen／constraints〔各 id・title・text・basis・source〕／glossary_pointer／constitution の articles〔各 id・title〕／rules の thresholds と discipline〔各 id〕／vocabulary の terms〔各 id・term・en〈null 可〉・short〕。任意の欄 = status_note・effective_version・approval 行の version と verbatim・scope_m1 の note・rail 行の basis と note・要件の rules・note・milestone・受入の note と milestone・制約の rules・verdicts の節）／表に無い値（pattern・strength・method・tone・meta の status）／meta の counts の不一致／参照の先が無い（goals → goals の id・verify の ac と verifies → acceptance の id・basis → 条 id・rules → rules 行 id・outputs の from と rail の reqs → 要件と非機能要件の id・figures の「図2-<n>」→ rail の n）／figures の値が 9 種の形のどれでもない／rail が 7 を超える・verdicts が 4 を超える・入れる側か出る側の帯が 4 を超える／role が「道具」の actor が 1 つでない／id が英数字と「-」「.」以外を含む／`--out` の親 dir が無い。

(e) 歯 `crates/folio/tests/face.rs`（既存の歯の file に足す・関数名はすべて face を含める＝verify の filter 語・便 14 の歯は本文も期待も変えない）。base で名に face を含む歯は `tests/face.rs` の 16 本・`src/face.rs` の unit 5 本・`src/face_constitution.rs` の unit 2 本・`tests/parts.rs` の 3 本で、`tests/parts.rs` は write-set に在るが本文も期待も変えない（実測: 2026-09-17・main ae9bd12）。便 0 の歯 `tests/check.rs`（5 本・(b0) の受け皿の回帰）は verify に `check` の filter で名指し、その filter が base で解く歯の file は `tests/check.rs` 5・`tests/gitcheck.rs` 4・`src/gitcheck.rs` 1・`tests/parity.rs` 1・`tests/render.rs` 1・`tests/parts.rs` 14・`tests/face.rs` 2（実測）で、`tests/gitcheck.rs`・`src/gitcheck.rs`・`tests/parity.rs`・`tests/render.rs` は write-set に在るが本文も期待も変えない。
- 床の受け皿: `folio check --dir design-intent` = 0（今の v1.2 は verdicts を持たない）／`tests/fixtures/face/` の srs.yaml（verdicts 3 行）を含む写しは `folio check` の入力にしない（4 file の写しは床が受けない）ので、受け皿の歯は unit（`src/check.rs` の中・名に check を含む）: verdicts 3 行の要件書 = 違反 0／verdicts の 1 行の cond が空 = 違反 1（種別は空の欄の既存の種別）／verdicts の id が acceptance の id と重複 = 違反 1。
- 凍結 fixture との byte 一致（P-10.1）: `--face srs --dir tests/fixtures/face --out <一時 file> --write` = 0 ∧ 出力が `tests/fixtures/face/expected-srs.html` と byte 一致。憲法面の期待 `expected.html` は便 14 のまま 1 byte も変えず、便 14 の歯 face_write_matches_the_frozen_fixture がそれを固定する（口を寄せた後も出力が変わらない証拠）。
- 実の正本で部品目録に合う（AC2 の機構）: `--dir design-intent` で一時 file へ `--write` = 0 → `folio parts --check --dir design-intent --page srs=<一時 file>` = 0 ∧「違反 0」。
- 逐語と件数の census（実の正本・yaml-rust2 で正本を直に読む）: 出力に 全要件と非機能要件の title・shall・plain・全受入基準の title・plain・全制約の text が含まれる／item-row の出現数 = requirements + nonfunctional の数／band-node = actors + outputs の数／rail-node = rail の数／rtm の tbody の tr = requirements + nonfunctional の数／glossary-links の中の a の数 = terms の数 × 2／ac-state-chip = acceptance の数／data-component の値は 17 種の中で lane-chip を含まない／state-strip は verdicts の節が無い今の正本では 0 で「図 3」の字面がリンク無しで 1 つ（FR5）。
- 図 3 の両側: fixture（verdicts 3 行を持つ）で state-node 3 と #fig-verdicts へのリンクが出る／fixture の写しから verdicts の節を消すと state-strip 0・「図3」の参照は字面だけ・それ以外の出力は同じ。
- check の 3 値と面の名: 便 14 の歯が `--face constitution` で固定しているので、srs では 一致 0 ／ 1 byte 変えて 1 ／ 無くて 2 の 3 つだけ。
- 導出できない入力 8 つ（fixture の写しに変異・どれも `--write` = 2 ∧「まだ分からない」∧ 出力先が出来ていない）: counts の fr を 1 ずらす／要件の figures に「図2-9」／要件の figures に「図4」／rail の 1 段の who を変えずに reqs に無い id／要件の basis に無い条 id／verdicts の 1 行の tone を表に無い値に／actors の「道具」の行の role を変えて 0 にする／pattern を表に無い値に。（8 つ。帯の上限 4 の超過は unit の歯で確かめる。）
- 憲法面の口の寄せの回帰: 実の正本で `--face constitution` を書き、便 14 と同じ census（部品 14 種・item-row 27 等は yaml-rust2 の数え）が通る（便 14 の歯 face_census_on_the_real_sources_counts_and_verbatims と face_on_the_real_sources_passes_parts_check がそのまま固定する）。
- unit（`src/face_srs.rs` の中・名に face を含む）: 図の参照の解き方（図2-1・図1・図3〔在る／無い〕・全段・図4 は Err・図2-0 は Err）・担当の表（持ち主／folio）・使う部品 17 種の faces に srs が入っている・rail の上限 7 と state-strip の上限 4 と context-band の帯の上限 4 が部品目録の max_nodes / max_per_band と同値。
- 凍結 fixture（`tests/fixtures/face/`・手書き・最小）: `srs.yaml` を本便で広げる（便 14 は requirements の FR1 の title だけを読むので、FR1 の title を変えなければ憲法面の期待は変わらない）。中身 = meta（title・version・status effective・promise・counts・approval 3 行〔作成・レビュー・承認〈verbatim・version〉〕・status_note）・goals 2・scope（build 2・not_build 1）・scope_m1（build 1・not_build 1・note）・actors 3（うち 1 つが role「道具」）・outputs 2・rail 3（段 2 が 持ち主 で reqs 空・basis に P-1）・requirements 2（FR1 = event・must・figures〔図2-1, 図1〕・basis〔P-1〕・goals〔GOAL1〕・ac〔AC1〕・rules〔R-1〕・note／FR2 = ubiquitous・should・figures〔全段, 図3〕・milestone M1・ac 空）・nonfunctional 1（NFR1・rules〔R-1〕）・acceptance 1（AC1・verifies〔FR1〕・red_test）・not_frozen・constraints 1（CON1・basis〔P-1〕・rules〔D-1〕）・verdicts 3（tone ok・bad・neutral）・glossary_pointer。`expected-srs.html` は (c) の規則で組んだ面（最初の 1 回は生成器の出力を写して置いてよく、置いた後は歯が固定する）。`vocabulary.yaml` は 2 語（seihon・escape）に short を足す（便 14 の憲法面の生成器は語彙の term・en・def・note だけを読み short を読まないので、憲法面の期待 expected.html は不変＝実測）。憲法・rules の fixture はそのまま読む（変えない）。独立性の分担は便 14 と同じ: 独立した物差しは `folio parts --check` と census、期待の面は回帰の anchor（P-10.2）。

(f) 便 14 までの形との接続: 新規は `crates/folio/src/face_srs.rs`・fixture `tests/fixtures/face/expected-srs.html`。`crates/folio/src/main.rs` で変えるのは `mod face_srs;` の 1 行だけ（副命令の枝は変えない）。`crates/folio/src/check.rs` で変えるのは SRS_TOP_LEVEL の 1 要素と check_srs の verdicts の行の数え（(b0)）だけで、他の関数と定数は変えない。`crates/folio/src/face.rs` は run の分岐に「srs」を足し、(b) の共有する口と (c) の名札の表を足す（run の他の分岐・文言・load・X・esc・val・hint・便 14 の表は変えない）。`crates/folio/src/face_constitution.rs` は共有する口を呼ぶ形に直す（出力は 1 byte も変えない・PARTS と章の中身は変えない）。`crates/folio/tests/face.rs` は歯を足すだけで便 14 の歯は変えない。`tests/fixtures/face/srs.yaml` は広げる（FR1 の title は変えない）・`tests/fixtures/face/vocabulary.yaml` は 2 語に short を足すだけ・`constitution.yaml`・`rules.yaml`・`expected.html` は 1 byte も変えない。`parts.rs`・`yaml.rs`・`verdict.rs`・`render.rs`・`refs.rs`・`vocab.rs`・`gitcheck.rs`・他の src・`build.rs`・`Cargo.toml`・`Cargo.lock`・`design-intent/` の下・`scripts/`・`.github/workflows/` は触らない。外部 crate は増えない。正規表現は使わない。size は M = 中身を変える既存の file 1 本あたりの増分の見積（`face.rs` が最大で 300 行未満・`face_constitution.rs` は減る＝write-set では縮む面の形〔先頭に -〕で申告し、受付はその余地を測らない・`main.rs` は 1 行・`check.rs` は 20 行未満・`tests/face.rs` は 300 行未満）。生成した面を `design-intent/preview/srs.html` へ書く（見本の置き換え）のは本便の外（A-1）。

## 2. 範囲

- 入れる: 要件書の面の生成器（図 1・図 2・verdicts の節が在れば図 3・対応表・用語の一覧）・共有する口の `face.rs` への寄せ・歯・凍結 fixture の拡張。
- 入れない: 入口の面（後続の便）・読む順番の札（lane-chip）・規範文の中の用語の小窓（hint term）・要件書 v1.3 の正本の変更そのもの（verdicts の節の追加は本便の着地の後に planner の PR・承認の行に裁定の逐語）・見本の置き換えと退役（A-1）・`folio check` への組み込み・CI・組み立てと配信・受入基準の結果の測定（chip は「まだ分からない」固定）・見た目だけの直し（憲法面の h2 の数え方の字面等・walk の後に 1 便で）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| shared | 共有する口 | 面の骨格の型・head と site-bar・帯・card・toc・foot・部品の名札の出し方 |
| labels | 名札の表 | 確かめ方・強度の色・図の色・担当・章・図の参照の解き方 |
| page | 要件書面の導出 | 図 1（actors / outputs）・図 2（rail）・図 3（verdicts）・要件の item-row・対応表・用語の一覧 |
| fixture | 凍結 fixture | srs.yaml の拡張と期待の面 |
| floor | 床の受け皿 | 要件書の節の閉じた一覧に verdicts・行の非空と id の重複 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 14 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "p"
title = "要件書の面を正本から folio face --face srs で生成し、憲法面と共有する口を face.rs に寄せる"
req = ["FR4", "NFR2"]
section = "1"
write-set = ["+crates/folio/src/face_srs.rs", "crates/folio/src/face.rs", "-crates/folio/src/face_constitution.rs", "crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/tests/face.rs", "crates/folio/tests/parts.rs", "crates/folio/tests/check.rs", "crates/folio/tests/gitcheck.rs", "crates/folio/src/gitcheck.rs", "crates/folio/tests/parity.rs", "crates/folio/tests/render.rs", "tests/fixtures/face/srs.yaml", "tests/fixtures/face/vocabulary.yaml", "+tests/fixtures/face/expected-srs.html"]
verify = ["cargo nextest run -p folio face", "cargo nextest run -p folio check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "face の歯（要件書面の凍結 fixture との byte 一致・実の正本で parts --check 合格・逐語と件数の census・図 3 の両側・check の 3 値・導出できない入力 8 つ・unit）が緑、便 14 の歯（憲法面の凍結 fixture との byte 一致・census）が期待不変で緑、便 0 の歯 check が期待不変で緑（受け皿の unit 3 つを含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
