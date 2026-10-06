// flip-check: moved s2-07l.679
//! group の族の歯（接頭辞 `hook_group_` / `hook_permission_`・設計 docs/design/carry-prep.md §9 行 g）と、席の追加文脈の
//! 書き込みの検出線の 1 行の歯（接頭辞 `prompt_write_budget_`・設計 write-budget.md §7・群の歯と同じ fixture）。

use super::*;

/// `Bash` の周は **stdout ちょうど 1 行**で deny を返し、記録を 1 行残す。
#[test]
fn hook_permission_request_denies_bash_with_one_json_line() {
    let repo = git_repo();
    let state = linked(&repo);
    let before = inject_lines(&state).len();
    let out = run_hook_args(
        &["permission-request", "--state-dir", &state.display().to_string()],
        &tool_payload(&repo, "Bash", "unused"),
    );
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "承認の答えは rc 0");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(text.lines().count(), 1, "stdout はちょうど 1 行: {text}");
    assert!(
        text.contains("\"hookEventName\":\"PermissionRequest\""),
        "どの event への答えかを名乗る: {text}"
    );
    // 入れ子を剥がして **parse できること**まで測る（escape が壊れていれば落ちる）。
    let pairs = json_lite::parse_object(&decision_object(text.trim()))
        .unwrap_or_else(|err| panic!("決定 object を parse できる: {err} / {text}"));
    let behavior = pairs.iter().find(|(key, _)| key == "behavior").map(|(_, v)| v.clone());
    assert_eq!(
        behavior,
        Some(json_lite::Value::Str("deny".to_owned())),
        "一律 deny である: {text}"
    );
    let message = match pairs.iter().find(|(key, _)| key == "message").map(|(_, v)| v.clone()) {
        Some(json_lite::Value::Str(found)) => found,
        other => panic!("message が文字列である: {other:?}"),
    };
    assert!(message.starts_with(&format!("{NAME}: ")), "器が名乗る: {message}");
    assert!(
        message.contains("literal path") && message.contains("$(...)"),
        "次の一手（literal path で書き直す）まで返す: {message}"
    );
    let lines = inject_lines(&state);
    assert_eq!(lines.len(), before + 1, "記録は 1 行だけ増える: {lines:?}");
    // `who` だけを見ると、`what` / `when` を別の値へ書き換える変異が生き残る
    // （実測 2026-09-10・lens-43 M-2）。契約が名指した field は全部測る。
    let last = &lines[lines.len() - 1];
    for (key, want) in [
        ("who", "hook:permission-request"),
        ("what", "deny"),
        ("when", "PermissionRequest"),
    ] {
        assert_eq!(
            value_of(last, key),
            Some(json_lite::Value::Str(want.to_owned())),
            "記録の {key}: {lines:?}"
        );
    }
    assert_eq!(
        value_of(last, "tokens"),
        Some(json_lite::Value::Null),
        "数えていない token は 0 でなく null: {lines:?}"
    );
    clean(&[&repo, &state]);
}

/// `Bash` 以外の問いには答えない（**0 byte・rc 0**＝既定の問いへ戻す・FR24）。
///
/// 答えてしまうと、器が引き受ける筋合いの無い承認まで機械が deny する。
#[test]
fn hook_permission_request_is_silent_for_other_tools() {
    let repo = git_repo();
    let state = linked(&repo);
    let before = inject_lines(&state).len();
    for tool in ["Edit", "Write", "WebFetch", ""] {
        let out = run_hook_args(
            &["permission-request", "--state-dir", &state.display().to_string()],
            &tool_payload(&repo, tool, "unused"),
        );
        assert_silent(&out, &format!("{tool} の問いには答えない"));
    }
    // payload が読めない面も同じく黙る（FR24 の「payload 不能」・lens-43 L-3）。
    // `cwd` を持つが tool 名が無い / 壊れている形を撃つ——`cwd` を落とすと process の
    // cwd へ落ちて別 repo を判定するので、ここで測りたいのは tool 名側の欠落である。
    for (why, payload) in [
        ("tool_name が無い", format!("{{\"cwd\":\"{}\"}}", repo.display())),
        (
            "tool_input だけ在る",
            format!("{{\"cwd\":\"{}\",\"tool_input\":{{}}}}", repo.display()),
        ),
        (
            "JSON として壊れている",
            format!("{{\"cwd\":\"{}\",\"tool_name\"", repo.display()),
        ),
    ] {
        let out = run_hook_args(
            &["permission-request", "--state-dir", &state.display().to_string()],
            &payload,
        );
        assert_silent(&out, &format!("payload 不能（{why}）でも答えない"));
    }
    assert_eq!(
        inject_lines(&state).len(),
        before,
        "答えなかった周は記録も残さない"
    );
    clean(&[&repo, &state]);
}

// flip-check: retroactive s2-07l.47
/// 記録に失敗しても **deny は取り消さず**、失敗を stderr へ出す（FR21 は推奨で判定ではない）。
///
/// `inject.jsonl` を **dir** にすると append が open の時点で落ちる。この周でも stdout は
/// ちょうど 1 行の deny・rc 0 のままで、落ちたことは stderr の行になる——`record_lines` を
/// `let _ = append(…)` へ替える変異は、ここで stderr が 0 行になって落ちる。
#[test]
fn hook_permission_request_surfaces_record_failure_on_stderr() {
    let repo = git_repo();
    let state = linked(&repo);
    // `inject_path` は器と同じ path を組む（file 名の字面を歯へ 2 本目として置かない）。
    fs::create_dir(inject_path(&state)).expect("inject.jsonl を dir として作れる");
    let out = run_hook_args(
        &["permission-request", "--state-dir", &state.display().to_string()],
        &tool_payload(&repo, "Bash", "unused"),
    );
    assert_eq!(
        out.status.code(),
        Some(i32::from(RC_OK)),
        "記録が落ちても hook は落ちない"
    );
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(text.lines().count(), 1, "stdout はちょうど 1 行: {text}");
    assert!(
        text.contains("\"behavior\":\"deny\""),
        "記録に失敗しても deny は取り消さない: {text}"
    );
    assert!(
        stderr_lines(&out) >= 1,
        "記録の失敗を黙って消さない: {:?}",
        stderr_text(&out)
    );
    clean(&[&repo, &state]);
}

/// marker の無い repo では **`Bash` でも黙る**（他の器と衝突しない・FR24）。
#[test]
fn hook_permission_request_is_silent_outside_vessel() {
    let repo = git_repo();
    let out = run_hook_args(&["permission-request"], &tool_payload(&repo, "Bash", "unused"));
    assert_silent(&out, "marker の無い repo では答えない");
    clean(&[&repo]);
}

/// (1) SessionStart: 種 a1 の 5 時間窓 90（行の値 85 以上）の席は brief の後ろに 1 行を出し、記録 1 行
/// （`what` = `group-pressure`）を残す。
#[test]
fn hook_group_session_start_brief_carries_one_pressure_line() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpstart", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(90, 10, 10));
    let before = inject_lines(&place.state).len();
    assert_eq!(group_session_lines(&place, &path), vec![group_line("a1", "5h", 90, 85)], "brief の後ろに 1 行");
    let records = inject_lines(&place.state);
    let grouped = records.iter().skip(before).filter(|line| what_of(line) == "group-pressure").count();
    assert_eq!(grouped, 1, "群の行の記録 1 行: {records:?}");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (2) UserPromptSubmit: 同じ席の追加文脈（stdout）はちょうど 1 行（打刻だけの周の 0 byte に 1 行が足される）。
#[test]
fn hook_group_user_prompt_submit_adds_one_context_line() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpprompt", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(90, 10, 10));
    assert_eq!(group_prompt_lines(&place, &path), vec![group_line("a1", "5h", 90, 85)], "追加文脈は 1 行");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (発話 m) 撃つ位置（設計 fleet-event-log.md §13 約束 1 / 7）: prompt を持つ UserPromptSubmit は、stdout の 1 行目が発話の
/// `<NAME> utterance: ts=<ts>`・2 行目が群の行。発話 event は 1 件（ts は 1 行目と同じ）で、席の打刻は Busy。
#[test]
fn hook_utterance_record_line_comes_before_the_group_line() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "utterprompt", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(90, 10, 10));
    let cwd = json_lite::quote(&place.repo.display().to_string());
    let payload = format!("{{\"cwd\":{cwd},\"session_id\":\"sid-group\",\"prompt\":\"群の席への依頼\"}}");
    let out = run_stub_hook(&path, &["user-prompt-submit", "--pane", STUB_PANE], &payload);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "rc 0: {}", stderr_text(&out));
    assert_eq!(stderr_text(&out), "", "stderr 0 byte");
    let lines: Vec<String> = String::from_utf8_lossy(&out.stdout).lines().map(str::to_owned).collect();
    assert_eq!(lines.len(), 2, "発話の行と群の行の 2 行: {lines:?}");
    let ts = lines[0].strip_prefix(&format!("{NAME} utterance: ts=")).unwrap_or_else(|| panic!("1 行目は発話の行: {lines:?}"));
    assert_eq!(lines[1], group_line("a1", "5h", 90, 85), "2 行目は群の行");
    let events = vessel::fleet::store::read_all(&place.state).unwrap_or_else(|errors| panic!("log を読める: {errors:?}"));
    let said: Vec<_> = events.iter().filter(|event| event.kind == vessel::fleet::EventKind::UtteranceReceived).collect();
    assert_eq!(said.iter().map(|event| event.ts.as_str()).collect::<Vec<_>>(), [ts], "発話 event は 1 件で ts は 1 行目と同じ");
    let stamps = fs::read_to_string(state_file(&place.state, "utterprompt")).unwrap_or_default();
    let last = stamps.lines().last().unwrap_or_default();
    assert_eq!(value_of(last, "state"), Some(json_lite::Value::Str("busy".to_owned())), "打刻は Busy: {stamps}");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (3) 群に属さない anchor（群の置き場が別の path）は、逼迫の実測が在っても 2 event とも 0 行（UserPromptSubmit は
/// stdout 0 byte のまま・SessionStart は群の行を足さない）。
#[test]
fn hook_group_anchor_outside_every_group_prints_nothing() {
    let place = group_role_place();
    put_group(&place, "/repo/not-this-one");
    let path = group_seat(&place, "grpoutside", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(99, 99, 99));
    assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "UserPromptSubmit は 0 byte");
    assert_eq!(group_session_lines(&place, &path), Vec::<String>::new(), "SessionStart も群の行なし");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (4) 閾値未満（5 時間窓 84・7 日窓 94・モデル別窓 94）は 0 行（鮮度の内側なので子も起こさない＝measuring も無い）。
#[test]
fn hook_group_under_threshold_prints_nothing() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpunder", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(84, 94, 94));
    assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "閾値未満は 0 行");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (5) 鮮度の外（実測なし・古い実測）は hook の中で測らず、器を子として 1 本起こして `usage: measuring` の 1 行だけを出す
/// （古い実測が閾値以上でも逼迫の行は出さない）。子は自席の口座 a1 だけを測る（credential の無い a1 の Unmeasured が
/// 置き場に届き、a2 の行は 1 件も来ない）。
#[test]
fn hook_group_stale_measurement_spawns_one_child_and_says_measuring() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpstale", "a1");
    let measuring = vec!["usage: measuring account=a1".to_owned()];
    assert_eq!(group_prompt_lines(&place, &path), measuring, "実測なし: measuring の 1 行");
    let rows = |label: &str| -> usize {
        vessel::fleet::store::read_all(&place.state)
            .unwrap_or_default()
            .iter()
            .filter_map(|event| event.allowance.as_ref())
            .filter(|row| row.key().account == label)
            .count()
    };
    let arrived = |label: &str, over: usize| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while rows(label) <= over && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        rows(label) > over
    };
    assert!(arrived("a1", 0), "子が a1 を測った行が置き場に届く");
    assert_eq!(rows("a2"), 0, "子は自席の口座だけを測る");
    put_group_round(&place.state, GROUP_STALE_TS, "a2", &group_windows(99, 99, 99));
    let other = group_seat(&place, "grpstale2", "a2");
    let measuring = vec!["usage: measuring account=a2".to_owned()];
    assert_eq!(group_prompt_lines(&place, &other), measuring, "古い実測（閾値以上）も measuring の 1 行だけ");
    assert!(arrived("a2", 3), "古い実測の口座も子が測り直す（置いた 3 行の後ろに届く）");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (6) 席の登録 row の口座が種と違う周は**自席の口座**を読む: 種 a1 は閾値未満・席の口座 a2 の 5 時間窓 90 → a2 の 1 行
/// （種を読む変異は 0 行になる）。
#[test]
fn hook_group_seat_account_differs_from_the_seed_and_is_read() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpseat", "a2");
    let now = group_now();
    put_group_round(&place.state, &now, "a1", &group_windows(10, 10, 10));
    put_group_round(&place.state, &now, "a2", &group_windows(90, 10, 10));
    assert_eq!(group_prompt_lines(&place, &path), vec![group_line("a2", "5h", 90, 85)], "自席の口座 a2 の 1 行");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (7) モデル別窓だけが閾値以上（5 時間窓 10・7 日窓 10・モデル別窓 96）の口座も 1 行で、window は model・cap は
/// モデル別窓の行の値（5 時間窓と 7 日窓の歯は model の窓を判定から落とす変異を捕まえない）。
#[test]
fn hook_group_model_window_alone_prints_the_model_window() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpmodel", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(10, 10, 96));
    assert_eq!(group_prompt_lines(&place, &path), vec![group_line("a1", "model", 96, 95)], "model の 1 行");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (f・account-lifecycle.md §33・契約表の行 w・接頭辞 `hook_group_model_gate_`) 埋め込みの役割の既定の表示名と**違う** model の
/// モデル別窓だけが 96（5 時間窓 10・7 日窓 10）の席は、逼迫の 1 行を出さず移動を頼む記録も置かない（base は出す ＝ RED）。
#[test]
fn hook_group_model_gate_other_model_window_alone_prints_nothing() {
    let other = ["Opus", "Fable", "Sonnet", "Haiku"].into_iter().find(|name| *name != group_role_model()).unwrap_or("Haiku");
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpother", "a1");
    put_group_round_as(&place.state, &group_now(), "a1", &group_windows(10, 10, 96), other);
    assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "役割の model でない {other} の窓は数えない");
    assert_eq!(group_requests(&place), Vec::<String>::new(), "頼みも置かない");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (8) 窓ごとに別の行の値を当てる: 7 日窓 90（5 時間窓の行 85 なら越える値）は 0 行・7 日窓 96 は 1 行で cap は 95
/// （5 時間窓の行を他の窓に当てる変異を捕まえる）。
#[test]
fn hook_group_caps_differ_per_window() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpcaps", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(10, 90, 10));
    assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "7 日窓 90 は 0 行");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(10, 96, 10));
    assert_eq!(group_prompt_lines(&place, &path), vec![group_line("a1", "7d", 96, 95)], "7 日窓 96 は 1 行");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (頼みを置く) 逼迫を読んだ UserPromptSubmit は群用 dir に移動を頼む記録を 1 file（`ts=` / `account=` / `window=` の 3 行）置く。
/// 別の窓（7 日窓 96）が逼迫する 2 度目の hook（SessionStart）は上書きしない（1 file のまま・本文は 1 度目と同じ＝ts も窓も不変）。
#[test]
fn hook_group_move_pressure_puts_one_request_and_never_overwrites_it() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpask", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(90, 10, 10));
    assert_eq!(group_prompt_lines(&place, &path), vec![group_line("a1", "5h", 90, 85)], "逼迫の 1 行");
    assert_eq!(group_requests(&place), vec![format!("{GROUP_NAME}.request")], "頼みは 1 file");
    let first = group_request_body(&place);
    let keys: Vec<&str> = first.lines().filter_map(|line| line.split_once('=').map(|(key, _)| key)).collect();
    assert_eq!(keys, ["ts", "account", "window"], "3 行: {first}");
    assert!(first.contains("\naccount=a1\nwindow=5h\n"), "口座と窓: {first}");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(10, 96, 10));
    assert_eq!(group_session_lines(&place, &path), vec![group_line("a1", "7d", 96, 95)], "2 度目も逼迫の 1 行");
    assert_eq!(group_requests(&place), vec![format!("{GROUP_NAME}.request")], "2 度目も 1 file");
    assert_eq!(group_request_body(&place), first, "上書きしない（ts も窓も 1 度目のまま）");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (頼みを置かない) 閾値未満の周と、群に属さない anchor の逼迫の周は、どちらも移動を頼む記録を 1 file も置かない。
#[test]
fn hook_group_move_under_threshold_or_outside_anchor_puts_no_request() {
    let under = group_role_place();
    put_group(&under, &under.repo.display().to_string());
    let path = group_seat(&under, "grpaskunder", "a1");
    put_group_round(&under.state, &group_now(), "a1", &group_windows(84, 94, 94));
    assert_eq!(group_prompt_lines(&under, &path), Vec::<String>::new(), "閾値未満は 0 行");
    assert_eq!(group_requests(&under), Vec::<String>::new(), "閾値未満は 0 file");
    clean(&[&under.repo, &under.state, &under.sock_dir]);
    let outside = group_role_place();
    put_group(&outside, "/repo/not-this-one");
    let path = group_seat(&outside, "grpaskoutside", "a1");
    put_group_round(&outside.state, &group_now(), "a1", &group_windows(99, 99, 99));
    assert_eq!(group_prompt_lines(&outside, &path), Vec::<String>::new(), "群の外は 0 行");
    assert_eq!(group_session_lines(&outside, &path), Vec::<String>::new(), "SessionStart も 0 行");
    assert_eq!(group_requests(&outside), Vec::<String>::new(), "群の外は 0 file");
    clean(&[&outside.repo, &outside.state, &outside.sock_dir]);
}

/// (食い違い) 記録の口座 a2 ≠ 登録 row の口座 a1 の席は、a1 が逼迫（5 時間窓 90）でも逼迫を測らず、UserPromptSubmit と
/// SessionStart の両方に `row=a1 current=a2` の 1 行だけを出し（`window=` を持たない）、移動を頼む記録を置かない。
#[test]
fn hook_group_current_record_differs_from_the_row_says_moving() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpmoving", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(90, 10, 10));
    put_group_record(&place, &group_record("a2", "a1"));
    let prompt = group_prompt_lines(&place, &path);
    assert_eq!(prompt, vec![moving_line("a1", "a2")], "UserPromptSubmit は移動中の 1 行");
    assert!(prompt.iter().all(|line| !line.contains("window=")), "逼迫を測らない: {prompt:?}");
    assert_eq!(group_session_lines(&place, &path), vec![moving_line("a1", "a2")], "SessionStart も移動中の 1 行");
    assert_eq!(group_requests(&place), Vec::<String>::new(), "移動を頼む記録は置かない");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (一致して逼迫) 記録の口座 a2 = 登録 row の口座 a2 で a2 が逼迫の席は「次の 1 周が移り先を決める」の 1 行で、「第 3 段」の
/// 語を持たない（移動を頼む記録は §20 形 4 のとおり置く）。
#[test]
fn hook_group_current_record_matches_the_row_says_the_next_round_decides() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpmatch", "a2");
    put_group_round(&place.state, &group_now(), "a2", &group_windows(90, 10, 10));
    put_group_record(&place, &group_record("a2", "a1"));
    let lines = group_prompt_lines(&place, &path);
    assert_eq!(lines, vec![group_line("a2", "5h", 90, 85)], "逼迫の 1 行");
    assert!(lines.iter().all(|line| line.contains("次の 1 周が移り先を決める") && !line.contains("第 3 段")), "{lines:?}");
    assert_eq!(group_requests(&place), vec![format!("{GROUP_NAME}.request")], "頼みは置く");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (読めない記録) 記録の形が壊れた席は、登録 row の口座が逼迫でも 2 event とも 0 行（席は止めない・rc 0・頼みも置かない）。
#[test]
fn hook_group_current_unreadable_record_prints_nothing() {
    let place = group_role_place();
    put_group(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "grpbroken", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(90, 10, 10));
    put_group_record(&place, "account=a2\n");
    assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "UserPromptSubmit は 0 行");
    assert_eq!(group_session_lines(&place, &path), Vec::<String>::new(), "SessionStart も 0 行");
    assert_eq!(group_requests(&place), Vec::<String>::new(), "頼みも置かない");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

// ─────── park の区画の席の hook（seat-heartbeat.md §21 形 7・契約表の行 z・FR95・接頭辞 `hook_park_`・§19 の fixture） ───────
//
// 区画（Tier9）の席は群と同じ門で逼迫の 1 行（`group=Tier9 …`）を出すが、移動を頼む記録を置かず、鮮度の外でも計測の子を起こさない
// （区画の手は計測を起こさない）。字面は契約から組む。

/// 区画の名。
const PARK_NAME: &str = "Tier9";

/// host の面に口座 a1 / a2 と区画 1 つ（置き場 = `anchor`・候補 = a1 → a2）を書く。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn put_park(place: &RolePlace, anchor: &str) {
    let body = format!(
        "schema = 1\n\n[[account]]\nlabel = \"a1\"\n\n[[account]]\nlabel = \"a2\"\n\n[[account-group]]\nname = \"{PARK_NAME}\"\n\
         anchors = [\"{anchor}\"]\naccounts = [\"a1\", \"a2\"]\n"
    );
    fs::write(place.state.join(vessel::rules::HOST_MANIFEST), body).expect("host の面を書ける");
}

/// 区画の逼迫の 1 行（器の字面を借りない）。
fn park_line(account: &str, window: &str, used: u64, cap: u64) -> String {
    format!("group={PARK_NAME} account={account} window={window} used={used} cap={cap} — 次の 1 周が移り先を決める")
}

/// host の根の群用 dir の直下の file 名（無ければ空）。
fn park_groups_files(place: &RolePlace) -> Vec<String> {
    let dir = place.state.parent().unwrap_or(&place.state).join(format!("{}-host", vessel::name::NAME)).join("groups");
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|entries| entries.filter_map(Result::ok).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    names.sort();
    names
}

/// (i) 区画の席の hook が a1 の逼迫（5 時間窓 90・行の値 85）の fixture で `group=Tier9 account=a1` で始まる 1 行を 2 event とも出し、
/// 群用 dir に移動を頼む記録の file を 1 つも置かない（base は 0 行 ＝ RED）。
#[test]
fn hook_park_pressed_seat_says_one_tier9_line_and_asks_for_nothing() {
    let place = group_role_place();
    put_park(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "parkhot", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(90, 10, 10));
    assert_eq!(group_prompt_lines(&place, &path), vec![park_line("a1", "5h", 90, 85)], "UserPromptSubmit は 1 行");
    assert_eq!(group_session_lines(&place, &path), vec![park_line("a1", "5h", 90, 85)], "SessionStart も 1 行");
    assert_eq!(park_groups_files(&place), Vec::<String>::new(), "移動を頼む記録は置かない（群用 dir の file 0）");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (j) 閾値未満（5 時間窓 84・7 日窓 94・モデル別窓 94）は 0 行・file 0（回帰の歯・base でも緑）。
#[test]
fn hook_park_under_threshold_prints_nothing() {
    let place = group_role_place();
    put_park(&place, &place.repo.display().to_string());
    let path = group_seat(&place, "parkunder", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(84, 94, 94));
    assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "閾値未満は 0 行");
    assert_eq!(group_session_lines(&place, &path), Vec::<String>::new(), "SessionStart も 0 行");
    assert_eq!(park_groups_files(&place), Vec::<String>::new(), "file 0");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (k) 記録なし（a1 の行が置き場に 0 件）と古い記録（鮮度の外の a1 の 3 行）の 2 fixture は 0 行（`usage: measuring` も無い）で、hook の
/// 後に 3 秒待っても置き場の a1 の計測の行が増えない（記録なしは 0 件のまま・古い記録は 3 件のまま＝計測の子の起動 0）。
#[test]
fn hook_park_missing_or_stale_measurement_prints_nothing_and_measures_nothing() {
    for (why, stale, held) in [("記録なし", false, 0), ("古い記録", true, 3)] {
        let place = group_role_place();
        put_park(&place, &place.repo.display().to_string());
        let path = group_seat(&place, "parkstale", "a1");
        if stale {
            put_group_round(&place.state, GROUP_STALE_TS, "a1", &group_windows(99, 99, 99));
        }
        assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "{why}: UserPromptSubmit は 0 行");
        assert_eq!(group_session_lines(&place, &path), Vec::<String>::new(), "{why}: SessionStart も 0 行");
        std::thread::sleep(std::time::Duration::from_secs(3));
        let rows = vessel::fleet::store::read_all(&place.state)
            .unwrap_or_default()
            .iter()
            .filter_map(|event| event.allowance.as_ref())
            .filter(|row| row.key().account == "a1")
            .count();
        assert_eq!(rows, held, "{why}: 置き場の a1 の行は増えない（計測の子の起動 0）");
        assert_eq!(park_groups_files(&place), Vec::<String>::new(), "{why}: file 0");
        clean(&[&place.repo, &place.state, &place.sock_dir]);
    }
}

// ─────── 書き込みの検出線の席の 1 行（設計 write-budget.md §7・接頭辞 `prompt_write_budget_`・群の歯と同じ fixture） ───────
//
// 表 `[[write-budget]]` は host の面（`<state>/host.toml`）に群の面の後ろへ足し、記録は host の根（`<state の親>/<NAME>-host/
// write-budget/<name>/`）に設計 §4 の形の `open` と `days.log` を歯が組んで置く（日付は撃つ前の今から組む・器の判定の口を使わない）。

/// 1 TB（10^12 byte）。
const WB_TB: u64 = 1_000_000_000_000;

/// 1 日の秒数。
const WB_DAY: u64 = 86_400;

/// 席の 1 行の定型の文（行の末）。
const WB_TAIL: &str = "— 書き込みの検出線の知らせ（器は作業を止めない・持ち主への札は owner=yes の周に消費側の板が出す）";

/// 撃つ前の今日の 0 時（UNIX 秒）。
fn wb_today0() -> u64 {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_secs());
    now - now % WB_DAY
}

/// 記録の dir（契約の字面から組む）。
fn wb_dir(place: &RolePlace, name: &str) -> PathBuf {
    place.state.parent().unwrap_or(&place.state).join(format!("{NAME}-host")).join("write-budget").join(name)
}

/// 群の面（`anchor` の群）を書き、`table` なら表の行 nvme-a を足す。
fn wb_face(place: &RolePlace, anchor: &str, table: bool) {
    put_group(place, anchor);
    if table {
        let path = place.state.join(vessel::rules::HOST_MANIFEST);
        let mut host = fs::read_to_string(&path).unwrap_or_default();
        host.push_str(&format!("\n[[write-budget]]\nname = \"nvme-a\"\nstat = \"{}\"\n", place.sock_dir.join("stat-nvme-a").display()));
        fs::write(path, host).ok();
    }
}

/// nvme-a の `days.log` を書く（`high` 日前までの閉じた日が 3.5 TB・その後ろ 7 日前までが 1 TB の measured）。
fn wb_days(place: &RolePlace, high: u64) {
    let today0 = wb_today0();
    let text: String = (1..=7)
        .map(|back| {
            let written = if back <= high { 35 * WB_TB / 10 } else { WB_TB };
            let date: String = vessel::fleet::cli::format_utc(today0 - back * WB_DAY).chars().take(10).collect();
            format!("schema=1 date={date} written={written} state=measured reboots=0 tail=-\n")
        })
        .collect();
    fs::create_dir_all(wb_dir(place, "nvme-a")).ok();
    fs::write(wb_dir(place, "nvme-a").join("days.log"), text).ok();
}

/// 注入の記録のうち what が `write-budget` の行。
fn wb_records(place: &RolePlace) -> Vec<String> {
    let what = Some(json_lite::Value::Str("write-budget".to_owned()));
    inject_lines(&place.state).into_iter().filter(|line| value_of(line, "what") == what).collect()
}

/// 呼び出しを 1 行ずつ `log` に残す偽 tmux の席（target `<name>:<name>`・口座 a1・anchor = repo）。PATH の値を返す。
#[expect(
    clippy::expect_used,
    reason = "統合 test の helper。clippy の allow-expect-in-tests は #[test] 関数の中だけに効く"
)]
fn wb_seat(place: &RolePlace, name: &str) -> (String, PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let path = group_seat(place, name, "a1");
    let log = place.sock_dir.join(format!("tmux-{name}.log"));
    let stub = place.sock_dir.join(format!("tmux-{name}")).join("tmux");
    fs::write(&stub, format!("#!/bin/sh\necho \"$*\" >> '{}'\necho '{name}:{name}'\n", log.display())).expect("偽 tmux を書ける");
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).expect("偽 tmux に実行権を付ける");
    (path, log)
}

/// UserPromptSubmit を撃つ（`args` は event の後ろの引数）。
fn wb_prompt(place: &RolePlace, path: &str, args: &[&str]) -> Output {
    let mut all = vec!["user-prompt-submit"];
    all.extend_from_slice(args);
    run_stub_hook(path, &all, &stamp_payload(&place.repo, "sid-group"))
}

/// 席の 1 行の字（契約の字面から組む）。
fn wb_line(over: &str, (yesterday, avg): (&str, &str), (streak, owner): (&str, &str)) -> String {
    format!("write-budget: name=nvme-a over={over} today=unmeasured yesterday={yesterday} avg={avg} streak={streak} owner={owner} {WB_TAIL}")
}

/// (a) 群に属さない anchor の席で昨日 3.5 TB（ほか 6 日 1 TB）の記録 → stdout がちょうど 1 行で、what が `write-budget` の記録が
/// 1 本増え（who と when も契約の字）、偽 tmux に send-keys が 0 回。逼迫の群の席で同じ記録 → 1 行目が群の行・2 行目が書き込みの行。
/// 7 日とも 1 TB の記録を `host.write_owner_days` の行を欠く `--rules` の写しで撃つ → over=no-rule の 1 行。
#[test]
fn prompt_write_budget_over_round_adds_one_line_after_the_group_line() {
    let over = wb_line("day", ("3500000000000", "1357142857142"), ("1", "no"));
    let place = group_role_place();
    wb_face(&place, "/repo/not-this-one", true);
    wb_days(&place, 1);
    let (path, log) = wb_seat(&place, "wbover");
    let before = wb_records(&place).len();
    assert_eq!(group_prompt_lines(&place, &path), vec![over.clone()], "越えた周は 1 行");
    let records = wb_records(&place);
    assert_eq!(records.len(), before + 1, "記録は 1 本増える: {records:?}");
    for (key, want) in [("who", "hook:user-prompt-submit"), ("when", "UserPromptSubmit")] {
        assert_eq!(value_of(&records[before], key), Some(json_lite::Value::Str(want.to_owned())), "記録の {key}");
    }
    let calls = fs::read_to_string(&log).unwrap_or_default();
    assert!(!calls.contains("send-keys"), "入力欄へ送らない: {calls}");
    clean(&[&place.repo, &place.state, &place.sock_dir]);

    let place = group_role_place();
    wb_face(&place, &place.repo.display().to_string(), true);
    wb_days(&place, 1);
    let path = group_seat(&place, "wbgroup", "a1");
    put_group_round(&place.state, &group_now(), "a1", &group_windows(90, 10, 10));
    assert_eq!(group_prompt_lines(&place, &path), vec![group_line("a1", "5h", 90, 85), over], "群の行の後ろ");
    clean(&[&place.repo, &place.state, &place.sock_dir]);

    let place = group_role_place();
    wb_face(&place, "/repo/not-this-one", true);
    wb_days(&place, 0);
    let embedded = include_str!("../../../../../rules/manifest.toml");
    let copy = embedded.replace("id = \"host.write_owner_days\"", "id = \"host.write_owner_dayz\"");
    assert_ne!(copy, embedded, "写しの字が替わる");
    let rules = place.sock_dir.join("no-owner.toml");
    fs::write(&rules, copy).ok();
    let path = group_seat(&place, "wbnorule", "a1");
    let out = wb_prompt(&place, &path, &["--pane", STUB_PANE, "--rules", &rules.display().to_string()]);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "rc 0: {}", stderr_text(&out));
    let lines: Vec<String> = String::from_utf8_lossy(&out.stdout).lines().map(str::to_owned).collect();
    assert_eq!(lines, vec![wb_line("no-rule", ("1000000000000", "no-rule"), ("no-rule", "no-rule"))], "線を読めない周も 1 行");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// 置き場を 1 つ作り、`days` の記録（`None` は記録なし）と `table` の面で UserPromptSubmit を `args` で撃ち、stdout と注入の記録の行数と
/// what が `write-budget` の記録の数を返す。
fn wb_quiet_round(days: Option<u64>, table: bool, args: &[&str]) -> (String, usize, usize) {
    let place = group_role_place();
    wb_face(&place, "/repo/not-this-one", table);
    if let Some(high) = days {
        wb_days(&place, high);
    }
    let path = group_seat(&place, "wbquiet", "a1");
    let out = wb_prompt(&place, &path, args);
    assert_eq!(out.status.code(), Some(i32::from(RC_OK)), "rc 0: {}", stderr_text(&out));
    let found = (String::from_utf8_lossy(&out.stdout).into_owned(), inject_lines(&place.state).len(), wb_records(&place).len());
    clean(&[&place.repo, &place.state, &place.sock_dir]);
    found
}

/// (b) 7 日とも 1 TB で `--pane` あり・表が無く昨日 3.5 TB の記録だけが在る置き場で `--pane` あり・表を持ち昨日 3.5 TB で `--pane` の
/// 無い呼び出しの 3 形は stdout 0 byte で、what が `write-budget` の記録が 0 本で、記録の行数が表と記録の無い同じ形の置き場の周と
/// 等しい。表を持ち昨日 3.5 TB の置き場の SessionStart の出力に `write-budget` の字が無い。同じ置き場で `--pane` ありは 1 行（対照）。
#[test]
fn prompt_write_budget_quiet_tableless_or_paneless_rounds_print_nothing() {
    let pane = ["--pane", STUB_PANE];
    for (why, days, table, args) in [("越えない", Some(0), true, &pane[..]), ("表が無い", Some(1), false, &pane[..]), ("pane が無い", Some(1), true, &[][..])] {
        let (out, records, budget) = wb_quiet_round(days, table, args);
        let (_, plain, _) = wb_quiet_round(None, false, args);
        assert_eq!((out.as_str(), budget, records), ("", 0, plain), "{why}: stdout 0 byte・記録を足さない");
    }
    let place = group_role_place();
    wb_face(&place, "/repo/not-this-one", true);
    wb_days(&place, 1);
    let path = group_seat(&place, "wbsession", "a1");
    let args = ["session-start", "--pane", STUB_PANE, "--bd", &place.bd];
    let session = run_stub_hook(&path, &args, &stamp_payload(&place.repo, "sid-group"));
    let text = String::from_utf8_lossy(&session.stdout).into_owned();
    assert!(!text.contains("write-budget"), "SessionStart には足さない: {text}");
    let out = String::from_utf8_lossy(&wb_prompt(&place, &path, &pane).stdout).into_owned();
    assert_eq!((out.lines().count(), wb_records(&place).len()), (1, 1), "対照: 越えた表の席は 1 行: {out}");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (c) 昨日から 3 日前まで 3.5 TB（ほか 1 TB）の記録 → 行が streak=3 owner=yes を持つ。
#[test]
fn prompt_write_budget_owner_round_carries_owner_yes() {
    let place = group_role_place();
    wb_face(&place, "/repo/not-this-one", true);
    wb_days(&place, 3);
    let path = group_seat(&place, "wbowner", "a1");
    assert_eq!(group_prompt_lines(&place, &path), vec![wb_line("day,avg", ("3500000000000", "2071428571428"), ("3", "yes"))]);
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}

/// (d) open を dir にし days.log の無い置き場と、壊れた host の面（表の行の stat の欠け）は rc 0・stdout 0 byte・stderr 0 byte。同じ
/// 置き場で days.log に越えの記録を置くと 1 行（対照）。
#[test]
fn prompt_write_budget_unreadable_round_prints_nothing_and_keeps_rc() {
    let place = group_role_place();
    wb_face(&place, "/repo/not-this-one", true);
    fs::create_dir_all(wb_dir(&place, "nvme-a").join("open")).ok();
    let path = group_seat(&place, "wbunread", "a1");
    assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "open が dir で days.log が無い周は 0 行");
    let host = place.state.join(vessel::rules::HOST_MANIFEST);
    let good = fs::read_to_string(&host).unwrap_or_default();
    fs::write(&host, "schema = 1\n\n[[write-budget]]\nname = \"nvme-a\"\n").ok();
    wb_days(&place, 1);
    assert_eq!(group_prompt_lines(&place, &path), Vec::<String>::new(), "壊れた host の面は 0 行");
    fs::write(&host, good).ok();
    let want = format!(
        "write-budget: name=nvme-a over=day today=unreadable yesterday=3500000000000 avg=1357142857142 streak=1 owner=no {WB_TAIL}"
    );
    assert_eq!(group_prompt_lines(&place, &path), vec![want], "対照: 越えの記録を置くと 1 行");
    clean(&[&place.repo, &place.state, &place.sock_dir]);
}
