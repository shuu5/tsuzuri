# 設計: 便 17 — 3 面と様式を配信先へ組み立てる `folio build` と、tailnet の内側だけで見せる `folio serve`

- 要件: FR7（build が実行されたとき全面を配信先へ生成する・serve は tailnet 上でのみ配信し公開ネットへ bind しない）/ CON3（見せる先は tailnet だけ）
- 条: N-6.1・N-6.2（bind 先が tailnet の外なら起動を拒む・配信の出力と支度表・台帳を認証なしの公開ネットに晒さない）/ P-2.1・P-2.2（人が読むページは 1 つの生成器から・手書きのページを配信先に置かない）/ P-4.1・P-4.2（実行できなかった生成を異常なしにしない・判定できないものは「まだ分からない」）/ P-6.2（生成物を手で直さない）/ P-10.1（独立した凍結 anchor）/ N-1.1（配信先の他の file を消さない）
- 判断の記録: ADR-5（決定 (1) 3 面の受入 = AC2 + walk 承認／帰結: 生成した 3 面が見本に置き換わる便で手書きの見本は退役する）。便 16（f2-648.27）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」（f2-648 notes・3 面 → walk 承認 → 組み立てと配信の順序）。3 面の walk 承認 = 持ち主 2026-09-18「確認した　大体これで良い」（f2-648 notes・AC5・入口の正本 v0.2 は発効済み）。見本 3 面の退役 = 持ち主 2026-09-18「１．見本は退役でよい」（f2-648 notes・A-1）。退役の移動は本便の後の planner の PR で行う（本便には含めない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 r が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

3 面（入口・憲法・要件書）は便 14〜16 で正本から生成できるようになり、持ち主の walk 承認も済んだ（AC5）。残る FR7 の 2 つの命令を置く。`folio build` は 3 面と様式（`folio.css`・`folio-ui.js`）を 1 つの配信先 dir へまとめて出す（面の生成は便 14〜16 の生成器をそのまま呼ぶ・本便は面の中身を 1 byte も変えない）。`folio serve` はその配信先を tailnet の内側だけで見せる小さな配信器で、bind 先が tailnet の外なら起動を拒む（N-6.1）。外部 crate は足さない（ADR-5 決定 (3) と同じ規律・配信器は標準 library の TcpListener だけで書く）。正規表現は使わない。

planner の実測（2026-09-18・main 38601e5）: 3 面の生成器は `crates/folio/src/face_index.rs`・`face_constitution.rs`・`face_srs.rs` の各 `pub fn derive(dir: &Path) -> R<String>`（3 本とも公開済み・`face.rs` の run はこの 3 本を面の名で選んでいる）。面の名の一覧は `parts.rs` の定数 FACES（index・constitution・srs の 3 つ）。様式は `design-intent/preview/folio.css`（59,895 byte）と `folio-ui.js`（5,723 byte）の 2 本で、各面の head は同じ dir からの相対（`folio.css`・`folio-ui.js`）で参照する（`face.rs` の Frame の head）。実の正本で生成した 3 面の大きさは 入口 13,071・憲法 153,488・要件書 114,501 byte。凍結 fixture `tests/fixtures/face/` は正本 5 file と `adr/ADR-1.yaml`・期待の面 3 本（`expected-index.html`・`expected.html`〔憲法〕・`expected-srs.html`）を持ち、様式の写しは持たない。verify の filter 語 site と serve を名に含む歯は base に 0 本（grep・src と tests の fn 名）。tailnet の住所の範囲は 100.64.0.0/10（tailscale が使う CGNAT の範囲）で、この host の tailnet の住所は tailscale0 の 1 つ（版管理へは書かない・D-8）。終了コードの 3 値は `verdict.rs`（合格 0・不合格 1・まだ分からない 2）。

(a) 命令 `folio build --dir <正本の置き場・既定 design-intent> --out <配信先> --write|--check`。
- `--out` は必ず指定する（既定を持たない・相対なら `--dir` からの相対・絶対ならそのまま）。既定を持たないのは、手書きの見本が在る `design-intent/preview/` を黙って上書きしないため（見本の退役の後に既定を足すかは別の便）。`--write` と `--check` はどちらか 1 つを必ず指定する（便 14 と同じ ArgGroup の形）。
- 配信先へ出す file は 5 本だけ（この表が閉じた一覧・順もこのとおり）: `index.html`（面 index の derive）・`constitution.html`（面 constitution の derive）・`srs.html`（面 srs の derive）・`folio.css`（`<dir>/preview/folio.css` の byte の写し）・`folio-ui.js`（`<dir>/preview/folio-ui.js` の byte の写し）。
- 全部か無しか: 3 面の導出と 2 本の様式の読みを先に全部 memory の上で済ませ、1 つでも導出できなければ 2 で終わり、配信先に 1 byte も書かない（配信先の dir も作らない）。
- write: `--out` の dir が無ければ作る（親 dir が無ければ 2「配信先の親 dir が無い」）。5 本を書く。配信先に在る他の file は消さない（N-1.1）。終了 0・標準出力「folio build: 書いた（5 file・N byte）」（N = 5 本の byte の合計）。
- check: 5 本のうち 1 本でも無ければ 2・標準エラー「folio build: 配信先に無い: <無い file の名を・で繋ぐ>」。全部在って 1 本でも byte 列が違えば 1・標準エラー「folio build: DRIFT — <違う file の名を・で繋ぐ>（手で直したか正本が変わった）」。全部一致なら 0・標準出力「folio build: OK — 配信先は正本と一致（5 file・N byte）」。比較は byte 列の一致（改行の読み替えをしない）。
- 導出できない = 2・標準エラー「folio build: まだ分からない: <理由>」（理由は面の生成器が返す文か、様式の file が読めない旨）。

(b) 命令 `folio serve --dir <配信先> [--host <IPv4>] [--port <数・既定 0 = 空きを OS が選ぶ>]`。起動の順に判定し、最初に当たったもので終わる:
1. `--dir` が無い・dir でない = 2「folio serve: まだ分からない: 配信先が無い」。
2. `--dir` の直下に `index.html` が無い = 2「folio serve: まだ分からない: 入口 index.html が無い」（入口の無い dir は見せない・手書きのページを入口に置かない P-2.2）。
3. `--dir` の直下に `.git` か `.beads` という名の項目が在る = 1「folio serve: 拒否 — 配信先に版管理か台帳が在る」（台帳の中身を晒さない N-6.2・repo の root や作業ツリーを配信先にしない）。
4. bind 先の決定。`--host` が在れば IPv4 として読み（読めなければ 2「bind 先が IPv4 でない」）、loopback（127.0.0.0/8）か tailnet の範囲（100.64.0.0/10）のどちらかでなければ 1「folio serve: 拒否 — bind 先 <住所> は tailnet の外」（N-6.1・loopback は同じ端末の中だけなので内側と扱う・歯と手元の確かめ用）。`--host` が無ければ tailnet の住所を自分で解く: UDP の socket を 0.0.0.0:0 に bind し 100.100.100.100:53（tailscale の名前引きの決まった住所・packet は送らない・connect で経路の出口の住所を OS に問うだけ）へ connect して local_addr を取り、それが 100.64.0.0/10 の中ならその住所、取れないか範囲の外なら 1「folio serve: 拒否 — tailnet の住所が無い」（FR7 の確かめ方・fail-closed）。
5. TcpListener を <住所>:<port> に bind する。できなければ 2「folio serve: まだ分からない: bind できない: <理由>」。
6. 標準出力に 1 行「folio serve: http://<住所>:<実際の port>/index.html（配信先 <dir の絶対 path>・止めるには Ctrl-C）」を書いて flush し、以後は接続を 1 つずつ順に処理する（並列にしない・止めるのは持ち主の Ctrl-C だけ・自分では終わらない）。
- 1 接続 = 1 要求（Connection: close）。読みの timeout 5 秒。要求の頭（空行まで）は 8 KiB まで（超えたら 400）。
- method は GET と HEAD だけ（他は 405）。要求の path は `?` と `#` から後ろを捨て、`/` で始まらなければ 400。`/` だけなら `/index.html`。`/` で区切った各節が 空・`.`・`..`・先頭 `.`（隠し file）・`%` か `\` を含む・ASCII 以外を含む のどれかなら 404。docroot と繋いで canonicalize し、canonicalize した docroot の下に無ければ 404（docroot の外を指す symlink を含む）。通常の file でなければ 404（dir は 404・一覧を出さない）。
- 200 の応答: 状態行・`Content-Type`（拡張子の表: html = text/html; charset=utf-8／css = text/css; charset=utf-8／js = text/javascript; charset=utf-8／json = application/json／svg = image/svg+xml／png = image/png／jpg と jpeg = image/jpeg／txt = text/plain; charset=utf-8／他 = application/octet-stream）・`Content-Length`・`Connection: close`・空行・本文（HEAD は本文なし・Content-Length は file の大きさ）。400・404・405 は text/plain; charset=utf-8 の短い本文（「400 読めない要求」「404 無い」「405 GET と HEAD だけ」）。
- 標準エラーに要求ごとに 1 行「<状態の数> <method> <path>」を書く（手元で確かめるため・stdout には 6. の 1 行以外を書かない）。書けない応答（相手が切った）は無視して次へ。

(c) 本便の「まだ分からない」（2）と「拒む」（1）はここに挙げたものだけ: build = (a) の 導出できない・様式が読めない・親 dir が無い・check の無い（2）と DRIFT（1）／serve = (b) の 1.〜5. のとおり（1 = 版管理か台帳が在る・tailnet の外・tailnet の住所が無い／2 = 配信先が無い・入口が無い・IPv4 でない・bind できない）。

(d) 歯。新規 2 本（`crates/folio/tests/site.rs` = build の歯・関数名はすべて site を含める／`crates/folio/tests/serve.rs` = serve の歯・関数名はすべて serve を含める＝verify の filter 語）。binary 経由（`CARGO_BIN_EXE_folio`）。版管理の `design-intent/preview/` は書き換えない（出力は必ず一時 dir）。
- 凍結 fixture との byte 一致（P-10.1）: `tests/fixtures/face/` の正本 5 file と `adr/` を一時 dir の `src/` へ写し、`src/preview/` を作って本便で足す最小の様式 `tests/fixtures/face/folio.css`・`tests/fixtures/face/folio-ui.js` を写す。`build --dir <src> --out <site> --write` = 0 ∧ 5 本が在る ∧ `index.html` = `expected-index.html`・`constitution.html` = `expected.html`・`srs.html` = `expected-srs.html`・様式 2 本 = fixture の 2 本（byte 一致）∧ 標準出力の N が 5 本の合計。
- check の 3 値: write の直後の `--check` = 0 ／ `srs.html` を 1 byte 変えて 1 かつ標準エラーに DRIFT と srs.html ／ `folio.css` を消して 2 かつ標準エラーに folio.css ／ 空の dir を `--out` にして 2。
- 実の正本（AC2 の機構）: `build --dir design-intent --out <一時 dir> --write` = 0 ∧ `folio parts --check --dir design-intent --page index=<site>/index.html --page constitution=<site>/constitution.html --page srs=<site>/srs.html` = 0 ∧ 各面の `folio face --face <面> --dir design-intent --out <site>/<面>.html --check` = 0（build と face の出力が同じ）。
- 全部か無しか: fixture の写しの `constitution.yaml` の counts を 1 ずらす（便 16 の歯と同じ変異）→ 2 ∧ `--out` の dir が出来ていない。親 dir が無い `--out` → 2。
- 消さない: `--out` に `extra.txt` を置いてから write → 0 ∧ `extra.txt` が残る。
- serve の 拒む: `--host 0.0.0.0`・`--host 192.168.0.1`・`--host 8.8.8.8` はどれも 1 ∧ 標準エラーに「tailnet の外」。`--dir` が無い → 2。`index.html` の無い dir → 2 ∧「入口」。`.git` という空の dir を持つ dir → 1 ∧「版管理か台帳」。`--host` 無し（FR7 の確かめ方）: 子 process を起動し、5 秒以内に 終了したら 1 ∧ 標準エラーに「tailnet の住所が無い」／標準出力に 1 行が出たら その住所が 100.64.0.0/10 の中であることを見て kill する（どちらの枝も断定・環境に tailnet が在るか無いかで分岐する）。
- serve の見せる（loopback）: fixture の写しで build した dir に `.hidden.txt`・下位 dir `sub/`・docroot の外の file への symlink `out.txt` を足し、`--host 127.0.0.1 --port 0` で起動 → 標準出力の 1 行から port を読む → TcpStream で `GET /index.html` = 200 ∧ Content-Type が text/html ∧ 本文が file と byte 一致／`GET /` = 同じ本文／`GET /folio.css` = 200 ∧ text/css／`HEAD /srs.html` = 200 ∧ Content-Length が file の大きさ ∧ 本文なし／`GET /nothing.html` = 404／`GET /.hidden.txt` = 404／`GET /sub` = 404／`GET /out.txt` = 404／`GET /../Cargo.toml` = 404／`POST /index.html` = 405／`GET index.html`（`/` で始まらない）= 400 → kill。
- unit（`src/serve.rs` の中・名に serve を含む）: 範囲の判定（127.0.0.1・100.64.0.0・100.127.255.255 は内側／100.128.0.0・0.0.0.0・10.0.0.1・192.168.0.1 は外）・path の各節の判定・拡張子の表・要求行の読み。unit（`src/site.rs` の中・名に site を含む）: 出す file の一覧が 5 本でこの順。
- 凍結 fixture に足すもの（最小の手書き・正本の写しは置かない）: `tests/fixtures/face/folio.css`（1 行の注釈と 1 つの規則だけ）と `tests/fixtures/face/folio-ui.js`（1 行の注釈だけ）。期待の面 3 本は変えない。
- 版管理へ書かないもの（D-8）: この host の tailnet の住所・機器名・口座名。歯に書く住所は loopback・範囲の境界の値・公開の例の住所（8.8.8.8・192.168.0.1）だけ。

(e) 便 16 までの形との接続: 新規は `crates/folio/src/site.rs`（build）・`crates/folio/src/serve.rs`・歯 2 本・fixture 2 本。`crates/folio/src/main.rs` は `mod site;`・`mod serve;` の 2 行と Command の 2 つの variant（Build・Serve）とその分岐だけ（既存の variant と分岐は触らない）。`face.rs`・`face_*.rs`・`parts.rs`・`render.rs`・他の src・`build.rs`・`Cargo.toml`・`Cargo.lock`・`scripts/`・`.github/workflows/`・`design-intent/` は触らない。外部 crate は増えない。正規表現は使わない。size は S = 中身を変える既存の file 1 本あたりの増分の見積（`main.rs` の増分は variant 2 つと分岐で 100 行未満・新規の 2 module と歯と fixture は増分に数えない）。

## 2. 範囲

- 入れる: `folio build`（5 本を配信先へ・全部か無しか・3 値の check）・`folio serve`（tailnet の内側だけ・fail-closed・GET と HEAD の静的配信）・歯・fixture の様式 2 本。
- 入れない: 手書きの見本 3 面の退役（承認済み・planner の別 PR・`tests/parts.rs` の当て先の更新を伴う）・`build` の `--out` の既定・読み物 `readable.html` の同梱（`folio render` のまま）・契約表の導出物と `build --check` の差分 0（FR11・M1）・id の索引（FR14・M1）・認証と https・接続の並列処理・dir の一覧・正本の変化を見張る再生成・配信先の掃除（他の file を消すこと）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| site | 組み立て | 3 面の derive と様式 2 本の写しを 1 つの配信先へ（全部か無しか・3 値の check） |
| bind | bind 先の判定 | 明示の住所の範囲の検査・tailnet の住所の自動の解決・fail-closed |
| http | 配信器 | GET と HEAD の静的配信・path の閉じ込め・拡張子の表 |
| fixture | 凍結 fixture | 最小の様式 2 本（期待の面 3 本は変えない） |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 16 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま・配信器は標準 library の net と fs だけ）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "r"
title = "3 面と様式を配信先へ組み立てる folio build と、tailnet の内側だけで見せる folio serve"
req = ["FR7"]
section = "1"
write-set = ["+crates/folio/src/site.rs", "+crates/folio/src/serve.rs", "crates/folio/src/main.rs", "+crates/folio/tests/site.rs", "+crates/folio/tests/serve.rs", "+tests/fixtures/face/folio.css", "+tests/fixtures/face/folio-ui.js"]
verify = ["cargo nextest run -p folio site", "cargo nextest run -p folio serve", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "site の歯（凍結 fixture との byte 一致・check の 3 値・実の正本で parts --check と face --check に合格・全部か無しか・消さない・unit）が緑、serve の歯（拒む 3 種と tailnet の住所の無い起動の拒否・loopback での配信 11 要求・unit）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
