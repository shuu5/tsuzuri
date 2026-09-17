# 設計: 便 7 — `folio check` に凍結 anchor の列（型付きの読み・正規化・sha256・索引・現行との一致）を足す

- 要件: FR5（3 値・実行できなかった検査を合格にしない）/ NFR3（正本の内部の整合）
- 条: A-2.1（条文の改訂は判断の記録と承認を伴う）/ N-4.1（記録を欠く改訂を拒む）/ P-7.1（id を再利用・改番しない）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-4.2（判定できないものは「まだ分からない」として表に出す）
- 判断の記録: ADR-2（凍結 anchor の列・digest・索引・列の根）。ADR-1 の帰結「M0 で folio が判断の記録の型を実装の側で持つ」の続き。便 6（f2-648.16）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes）。sha256 は外部 crate を足さず folio の中に手書きする（A-3.1・持ち主 2026-09-17「2は推奨で進めて良い」・f2-648 notes）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 h が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`・新規 dir は file を 1 本ずつ列挙）。fixture は最小の手書き（正本の写しは使わない）。

## 1. 目的と中身

便 6 の `folio check` に、凍結 anchor の列（`design-intent/anchors/`）の検査のうち **版管理（git）を見ない部分**を足す。day-1 の床 `scripts/check_draft.py` の anchor の節と同じ式で、版管理との照合・列の区間の amends の消し込み・`--freeze-anchor`・`--emit-amends` は便 8。この便では床の script を触らない。

(a) 判定の 3 値の並びを day-1 の床に揃える。床は「読めない（止まる）」と「測れない（続ける）」を分け、終了コードは 違反が 1 つでも在れば 1、違反が無く測れないものが在れば 2、どちらも無ければ 0（床の script の末尾の式・docstring「1 と 2 が同時に立つときは 1」）。便 0 の `crates/folio/src/verdict.rs` は「読めない・測れない」を 1 つの一覧で持ち、それが在れば違反より先に「まだ分からない」を立てていた（4 file が読めない場合は床も止まるので同じだが、anchor が 0 本のような「測れない」では床と食い違う）。便 7 は Report に **測れない（pending）の一覧**を足し、判定を 読めない（unknowns）が在れば まだ分からない ／ そうでなく違反が在れば 不合格 ／ そうでなく測れない（pendings）が在れば まだ分からない ／ どれも無ければ 合格 に改める。測れないは Report の**別の一覧**（pendings）に積み、読めないの一覧（unknowns）には積まない（積むと判定が「まだ分からない」に倒れて既存の歯の終了コードが 2 になる）。命令の入口は測れないの各行を読めないと同じ形（標準エラーへ「# まだ分からない: 」+ 文言）で出し、要約行の「まだ分からない N」は読めない + 測れない の合計にする。既存の歯（check / refs / vocab / adr / link）は標準出力の「[」で始まる行だけを違反として数え、終了コードと「不合格」の字面を見て、要約行の「まだ分からない N」を assert する歯は 0 本（admin の実測 2026-09-17 main 65a0dbe）。便 0〜便 6 の検査が積む「読めない」は変えない（既存の歯の期待は不変）。

(b) 型付きの読み。凍結 anchor の digest は day-1 の床が「anchor の digest 以外の全欄を json（キー順固定・空白なし・ensure_ascii なし・日付は文字列）に直列化した sha256」で作る（adr/schema.yaml anchor_note）。同じ byte 列を作るには yaml の scalar の型（真偽・整数・浮動小数・日付・null・文字列）が要るので、`crates/folio/src/yaml.rs` に型付きの木（Value = Null / Bool / Int / Float / Date / Str / Seq / Map）を返す読みを足す（既存の Node と parse は変えない・同じ読み手の事象を使う）。plain（引用符無し）の scalar だけを型に解き、引用符付きは文字列。解き方（day-1 の床の読み手 PyYAML の既定の解決のうち、正本と anchor に出る部分集合・それ以外は fail-closed）: 空・~・null・Null・NULL → Null／true・True・TRUE・yes・Yes・YES・on・On・ON → Bool 真、false・False・FALSE・no・No・NO・off・Off・OFF → Bool 偽／任意で「-」か「+」の後に 0 か 1〜9 で始まる数字列だけ → Int（先頭の「+」は落とす）／任意で「-」か「+」の後に 数字列 + 「.」+ 数字列 だけ → Float／数字 4 桁 + 「-」+ 数字 2 桁 + 「-」+ 数字 2 桁 だけ → Date（字面を持つ）／それ以外 → Str。**先頭が 0 で 2 文字以上の数字列（八進）・「_」を含む数・0x / 0o / 0b・「:」で区切る数（六十進）・指数付きや「.」で始まる／終わる浮動小数・.inf / .nan・日付の後に時刻が続く形（時刻付きの timestamp）**は PyYAML では型付きに解けるのにこの便では解かないので、plain の scalar がこれらの形に当たれば「まだ分からない（正規化できない scalar・欄の道）」（読めない側）に落とす。

(c) 正規化（json の字面）。Null → null／Bool → true・false／Int → 数字列（先頭の「-」はそのまま）／Float → Rust の既定の表示（最短の往復表示）に「.」も「e」も無ければ「.0」を足したもので、絶対値が 10 の 16 乗以上か（0 でなく）10 の -4 乗未満なら「まだ分からない（正規化できない scalar）」（Python の repr は指数表記になるため）／Date → その字面を文字列として／Str → 二重引用符で囲み、中の二重引用符と逆斜線（バックスラッシュ）は逆斜線を前置し、U+0000〜U+001F は \n・\r・\t・\b・\f はその 2 文字に、他は \u + 4 桁の 16 進（小文字）に、それ以外（非 ASCII を含む）はそのまま／Seq → 「[」+ 要素を「,」で繋ぐ +「]」／Map → キーを符号位置の昇順（byte 列の昇順と同じ）に並べ「{」+ 二重引用符で囲んだキー + 「:」+ 値 を「,」で繋ぐ +「}」。空白は入れない。main の anchor v1.0 の実測（2026-09-17・day-1 の床の式）: 正規化した本文は 12,649 byte・scalar の型は 文字列 516・真偽 3・null 2・日付 2・整数 1（浮動小数は 0）・二重引用符・逆斜線・制御文字を含む文字列は 0。

(d) sha256。FIPS 180-4 の SHA-256 を `crates/folio/src/sha256.rs` に手書きする（外部 crate を足さない・A-3.1 の裁定）。歯（同じ file の unit の歯）: 空文字列 → e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855／abc → ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad／abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq（56 byte）→ 248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1／1,000,000 個の a → cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0。加えて `tests/anchor.rs` の歯が main の `design-intent/anchors/constitution-v1.0.yaml` を (b)(c) で正規化して sha256 を取り、file の digest 欄と床の定数 root_digest（acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed）に一致することを見る（凍結 anchor が生成側からも検査側からも独立した固定の物差しになる・P-10.1）。

(e) 現行の写し（projection）。憲法を型付きで読み、範囲 = 憲法 schema.amendment_scope の各節（articles 以外はその節の木をそのまま・articles は各条を id・title・tier・binds・statements の 5 欄に絞り、statements の各規範文を id・text・pattern・strength の 4 欄に絞る）。写しの中に 印（（新設）（削除）（空））を値として持つ欄が在れば 1 違反ずつ（A-2）・欄名が文字列でないか「.」を含めば 1 違反ずつ（A-2）。憲法 meta.version の綴りが v + 数字列 + 「.」+ 数字列 でなければ 1 違反（A-2）。

床の定数の所在: anchor に関わる床の定数（file_keys・first_version・root_digest・version_pattern・digest_algo・projection_article_fields・statement_fields・scope_minimum と印 3 つ）は便 5 が `crates/folio/src/adr.rs` に置いた非公開の定数 FLOOR の中に在り（便 6 が crate の中へ可視性を広げるか読み口を足す）、`anchor.rs` はそれを読む。便 6 が開いた読み口は一覧型の定数（floor_strs）と記録の型（Keys・Adr・check_adr が記録を返す）だけで、値型（root_digest・first_version・version_pattern・digest_algo・印 3 つ）と数型の読み口は無い（admin の実測 2026-09-17 main 996cdd9）ので、便 7 は `adr.rs` に値型と数型の読み口を 1 つずつ足す（値と検査の式は変えない・adr.rs は write-set に在る）。便 6 の `crates/folio/src/link.rs` は unit の歯の名（adr_ids_follow_the_floor_pattern・floor_constants_are_read_through_the_floor）に adr を含み verify の filter adr で歯の file として解けるので、write-set に在るが触らない（本文も期待も変えない）。違反の記録型は便 4〜便 6 と同じで、`verdict.rs` の Report の関数 violation は種別を文字列で受け、種別（anchor・A-2・N-4・P-7・adr）は文字列 1 つで閉じる。

(f) anchors/ と索引。`<dir>/anchors/` が無ければ anchor 0 本として (i) へ。在って symlink か dir でなければ「まだ分からない（読めない側）」。`index.yaml` が在れば読み（symlink・design-intent の外の実体は 1 違反 adr で読まない）、kind が constitution-anchor-index で entries が一覧でなければ 1 違反（anchor）で索引なしと扱う。直下の `constitution-` で始まる `.yaml` を名前順に読む（symlink・外の実体は 1 違反で飛ばす）: 表でない・欄の集合が床の定数 anchor.file_keys（kind・digest_algo・version・previous・projection・meta_approval・approvals・content・digest）と一致しない・kind が constitution-anchor でない → 1 違反（anchor）で飛ばす／file 名が constitution-<version>.yaml と違う → 1 違反／同じ版が 2 本 → 2 本目は 1 違反で飛ばす／版の綴り → 1 違反／digest_algo が床の sha256-json-1 と違う → 測れない（pending・写しの照合は続ける）、同じなら (c) の正規化（digest 欄を除く全欄）の sha256 が digest 欄と違えば 1 違反／projection の article_fields・statement_fields が床の定数と違う → 測れない（pending）で飛ばす／projection.scope が床の scope_minimum（schema・precedence・articles）を含まない → 1 違反／content の欄の集合が scope と一致しない → 1 違反／meta_approval が憲法 meta.approval と（型付きの正規化の字面で）一致しない → 1 違反（N-4）／approvals が空でない一覧でない → 1 違反／各項の who・ruling・verbatim が空 → 1 違反、ruling に台帳 id の並び（便 5 の (d)）が無い → 1 違反、adr が null でなくその判断の記録が無い → 1 違反（N-4）、在って approval の 5 欄（who・date・ruling・verbatim・surface）の正規化の字面が一致しない → 1 違反（N-4）。

(g) 列。索引が在るとき、entries の各項に version・previous・digest が無ければ 1 違反。先頭の version が床の first_version（v1.0）でなければ 1 違反、先頭の digest が床の root_digest と違えば 1 違反（列の根は 1 度きり）。各項の previous が直前の項の version（先頭は null）でなければ 1 違反。version の anchor が読めていなければ 測れない（pending・列が切れている）。読めていれば anchor の digest が索引の digest と違えば 1 違反、anchor の previous が索引の列と違えば 1 違反。索引に無い anchor は 1 違反ずつ。索引が無く anchor が在れば 1 違反（索引が消された）。索引が無く anchor も無く、改訂の記録（憲法のいずれかの条の amended_by が非空・または発効した判断の amends が非空）が在れば 1 違反（N-4・anchor が消された）。entries が空なら 測れない（pending・比較元が立たない）。

(h) 版と記録。列の版の一覧（索引の順）と根の版（先頭・索引が無ければ v1.0）。名指せる版 = 列の根より後の版 ∪（現行 meta.version が根でなければそれ）。発効した各判断の amends の version がこの集合に無いとき: 根の版なら 1 違反（A-2・架空の記録）、それ以外は 1 違反（anchor・列に無い版）。最新の anchor（索引の末尾）が読めているとき、過去の anchor の列に在って最新 anchor に無い条 id・規範文 id が現行に再登場していれば 1 違反ずつ（P-7・番号の再利用）。

(i) 現行との一致。索引も anchor も無く記録も無ければ 測れない（pending・「凍結 anchor が 0 本」）。最新 anchor が読めているとき: 最新の版が現行 meta.version と違えば 1 違反（A-2・版を上げたのに凍結していない）とし、加えて 最新 anchor に在って現行に無い条 id は 1 違反ずつ（P-7）、最新 anchor に在って現行に無い規範文 id の text の集合と 現行に在って最新 anchor に無い規範文 id の text の集合が交われば交わる text ごとに 1 違反（P-7・改番）。同じ版なら、最新 anchor の content と (e) の現行の写しの正規化の字面が一致しなければ 1 違反（N-4・記録と承認を伴わない改憲）。

(j) 列に在った id を 4 file の参照 id の解決先へ足す（便 1 が「便 2 以降」とした分）: 読めた各 anchor の content.articles の条 id と規範文 id を、便 1 の `refs.rs` の解決先の集合に加える。読めない anchor は寄与しない。

(k) 判定は (a) の 3 値。正本（main）では違反 0・測れない 0（anchor 1 本・索引 1 項・版 v1.0 = 現行・content = 現行の写し・digest = root_digest。day-1 の床の実測 2026-09-17）。

便 0〜便 6 の fixture 14 組（`tests/fixtures/check/` の 3 組・`refs/` 3 組・`vocab/` 2 組・`adr/` 3 組・`link/` 3 組）は anchors/ を持たず、憲法の meta.version はすべて v1.0（実測 2026-09-17・11 組で確認・link の 3 組も同じ最小形）。このうち 13 組は改訂の記録も無いので (i) で 測れない（pending）が 1 つ立つだけで、(a) の並びにより 違反 1 件の組は 不合格（終了コード 1）のまま・要約行の「まだ分からない」が 0 から 1 になるだけ＝各歯（`crates/folio/tests/check.rs`・`refs.rs`・`vocab.rs`・`adr.rs`）の期待（終了コード・違反の行数・種別と文言）は変わらない。`check/missing-file/` は 4 file の読みで止まり 2 のまま。残る 1 組 `link/amended-by-orphan/` は憲法の P-1 に amended_by を 1 項持つ（便 6 の変異）ので改訂の記録が在り、(g) の「索引も anchor も無いのに記録が在る」で違反が 1 件増えて 2 件（N-4 発効していない + N-4 anchor が消された）になる。便 7 は便 6 の歯 `crates/folio/tests/link.rs` のこの 1 本だけ、期待を「違反 2 件・終了コード 1・2 件の種別と文言を名指す」に改める（fixture は触らない・他の歯と他の 2 組の期待は変えない）。`check.rs`・`refs.rs`・`vocab.rs`・`adr.rs` の 4 本は verify で回すために write-set に在るが触らない（本文も期待も変えない）。fixture 14 組も触らない。

`folio check` 自身の歯（`crates/folio/tests/anchor.rs`・binary 経由）の fixture は `tests/fixtures/anchor/` の 2 組。`no-anchor/` = 便 6 までの追記込みの最小 6 file（4 file + adr/schema.yaml + adr/ADR-1.yaml・違反 0・anchors/ なし）→ 終了コード 2・違反 0・標準エラーに「凍結 anchor が 0 本」を含む行（(a) の「測れない」が単独で立つ形）。`root-digest-drift/` = 同じ 6 file に憲法 meta.approval（who・date・ruling・verbatim・surface の 5 欄・ruling に台帳 id）と `anchors/index.yaml`（entries 1 項・version v1.0・previous null）と `anchors/constitution-v1.0.yaml`（file_keys の 9 欄・content = この fixture の憲法の (e) の写し・meta_approval = 憲法 meta.approval・approvals 1 項 {adr: null, who, date, ruling, verbatim, surface}・digest = (c) の正規化の sha256 で自分自身と一致・索引の digest も同じ値）を足したもの → 終了コード 1・違反 1（種別 anchor・文言に 列の根 と 床の定数）＝根の digest が folio2 の v1.0 の定数と違うことだけが違反（列の根は 1 度きり）。fixture の digest は worker が便 7 の (b)(c)(d) の実装で一度だけ計算して file に書く（歯の中では計算しない・凍結）。その実装が day-1 の床と同じ式である裏は、(d) の歯が main の anchor の digest 欄と床の定数 root_digest に一致することで取る（main の index.yaml の根の digest = acb52acd… は床の定数と一致・admin の実測）。新 fixture 2 組の 6 file は便 6 の追記済みの形（憲法の schema 節に enums と amendment_scope・P-2 の relations.rules と rules の thresholds に R-8 の行・語彙の identifiers は folio と opus だけ・ADR-1 の本文は日本語だけ）に揃え、anchors/ の 2 file の英字は便 4 の R-9 の母集団（4 file の欄だけ）に入らない。歯は加えて (d) の main の anchor の digest の一致を見る。parity の入力にはしない。

突き合わせの歯（`crates/folio/tests/parity.rs`・便 6 の 22 入力に足す・写し全部 + git 1 commit + 変異 1 つ・床と folio の終了コードの一致）: (23) `constitution.yaml` の P-1 の title を変える → 1／(24) `anchors/index.yaml` の digest の末尾 1 字を変える → 1／(25) `anchors/constitution-v1.0.yaml` の content の P-1.1 の text の 1 字を変える → 1／(26) `constitution.yaml` の meta.version を v1.1 にする → 1／(27) `anchors/index.yaml` の entries を空の一覧にする → 1。既存の 22 入力は変えない（写しは git 1 commit を含むので、便 8 までは床の版管理との照合が「測れない」を積まず、終了コードは (a) の並びで一致する）。

便 6 が置いた形との接続: `crates/folio/src/main.rs` に `mod anchor; mod sha256;` を足し、測れないの出力と要約を (a) に改める。実装は新規 `crates/folio/src/anchor.rs`（(e)〜(i)）と `crates/folio/src/sha256.rs`（(d)）。`yaml.rs` に (b) の型付きの読みと (c) の正規化を足す（既存の Node・parse・重複キーの検出は変えない）。`verdict.rs` に (a) の測れないの一覧と判定を足す（既存の unknowns・violations は変えない）。`check.rs` の `check_dir` から `link` の次に `anchor` を呼び、`refs.rs` の解決先に (j) を足す（`refs.rs` の他の式は変えない）。判断の記録（発効・amends・approval）は便 5・便 6 の `adr.rs` が読んだものを使う。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。便 2・便 3 の歯と fixture は触らない。

## 2. 範囲

- 入れる: 3 値の並びの是正（測れない）・型付きの読みと json の正規化・手書き sha256・anchors/ と索引と列・現行の写しとの一致・未凍結の版の条の消失と改番・番号の再利用・記録が名指す版・列に在った id の解決先への追加・歯 `anchor.rs` と fixture 2 組・parity の 5 入力。
- 入れない: 版管理（git）との照合（HEAD・履歴・ignore・浅い写し・環境変数の遮断）・列の区間（隣り合う anchor どうし）の amends の 1:1 の消し込みと amended_by の照合・`--freeze-anchor`・`--emit-amends`・`scripts/check_draft.py` の変更。すべて便 8。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| pending | 測れない | 読めないと分けて持ち、違反より後に立てる |
| typed | 型付きの読み | plain の scalar を部分集合の規則で型に解き、解けない形は止める |
| canon | 正規化 | 型付きの木を day-1 の床と同じ json の字面にする |
| sha256 | 要約 | 手書きの SHA-256（検証ベクトルで歯を打つ） |
| chain | 列と索引 | anchor の欄・digest・索引・previous・根の定数 |
| projection | 現行の写し | 範囲と 5 欄・4 欄に絞り、最新 anchor と比べる |

## 4. 検査（歯）

§1 のとおり（sha256 の unit の歯・anchor の歯 2 組 + main の anchor の digest・便 0〜便 6 の歯そのまま・parity 27 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま・sha256 は手書き）。parity の歯は CI が既に持つ Python 3 と pyyaml を使う。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "h"
title = "folio check に凍結 anchor の列（型付きの読み・正規化・sha256・索引・現行との一致）を足す"
req = ["FR5", "NFR3"]
section = "1"
write-set = ["+crates/folio/src/anchor.rs", "+crates/folio/src/sha256.rs", "crates/folio/src/yaml.rs", "crates/folio/src/verdict.rs", "crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/src/refs.rs", "crates/folio/src/adr.rs", "crates/folio/src/link.rs", "+crates/folio/tests/anchor.rs", "crates/folio/tests/parity.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/link.rs", "+tests/fixtures/anchor/no-anchor/constitution.yaml", "+tests/fixtures/anchor/no-anchor/rules.yaml", "+tests/fixtures/anchor/no-anchor/vocabulary.yaml", "+tests/fixtures/anchor/no-anchor/srs.yaml", "+tests/fixtures/anchor/no-anchor/adr/schema.yaml", "+tests/fixtures/anchor/no-anchor/adr/ADR-1.yaml", "+tests/fixtures/anchor/root-digest-drift/constitution.yaml", "+tests/fixtures/anchor/root-digest-drift/rules.yaml", "+tests/fixtures/anchor/root-digest-drift/vocabulary.yaml", "+tests/fixtures/anchor/root-digest-drift/srs.yaml", "+tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "+tests/fixtures/anchor/root-digest-drift/adr/ADR-1.yaml", "+tests/fixtures/anchor/root-digest-drift/anchors/index.yaml", "+tests/fixtures/anchor/root-digest-drift/anchors/constitution-v1.0.yaml"]
verify = ["cargo nextest run -p folio sha256", "cargo nextest run -p folio anchor", "cargo nextest run -p folio check", "cargo nextest run -p folio refs", "cargo nextest run -p folio vocab", "cargo nextest run -p folio adr", "cargo nextest run -p folio link", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "sha256 の歯が 4 つの検証ベクトルで緑、anchor の歯が fixture 2 組と main の anchor の digest で §1 のとおり緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、便 5 の歯 adr が期待不変で緑、便 6 の歯 link が期待不変で緑、parity の 27 入力が床と一致して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
