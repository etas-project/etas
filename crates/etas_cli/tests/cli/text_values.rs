use super::*;

#[test]
fn segmented_prompt_json_matches_source_after_cli_checkpoint_resume() {
    let piece = "中😀\n\"\\\u{0}payload";
    let original = piece.repeat(32);
    let input = format!("{original}tail");
    let source = fixture(
        "segmented-prompt-json",
        &format!(
            r#"
module tests.cli.segmented_prompt_json;
import std.runtime.checkpoint;
flow main(args: Array<string>) -> i32 {{
    var source = "";
    var index = 0;
    while index < 32 limit Iterations(64) {{
        source = source + {piece};
        index = index + 1;
    }}
    let original = source;
    let prompt = Prompt.new().data({{payload = source + "tail"}});
    source = source + "!";
    checkpoint("segmented-prompt-json");
    let completed = prompt.data({{original, appended = source}});
    let expected = Prompt.new().data({{payload = {input}}})
        .data({{original = {original}, appended = {appended}}});
    if completed == expected && original == {original} && source == {appended} {{ return 0; }}
    return 1;
}}
"#,
            piece = serde_json::to_string(piece).unwrap(),
            input = serde_json::to_string(&input).unwrap(),
            original = serde_json::to_string(&original).unwrap(),
            appended = serde_json::to_string(&format!("{original}!")).unwrap(),
        ),
    );
    let checkpoints = temp_path("segmented-prompt-checkpoints", "dir");
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["run", path(&source), "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    let saved = checkpoints.join("checkpoint-0.json");
    let before = fs::read(&saved).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["resume", "0", "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    assert_eq!(
        fs::read(saved).unwrap(),
        before,
        "resume changed its input artifact"
    );
}

#[test]
fn segmented_unicode_case_mapping_preserves_aliases_across_cli_checkpoint_resume() {
    let piece = "AΣ\u{301}Aßİ中😀 ";
    let original = piece.repeat(32);
    let input = format!("{original}AΣ\u{301}");
    let quoted = |text: &str| serde_json::to_string(text).unwrap();
    let source = fixture(
        "segmented-unicode-case",
        &format!(
            r#"
module tests.cli.segmented_unicode_case;
import std.text.{{lowercase, uppercase}};
import std.runtime.checkpoint;
flow main(args: Array<string>) -> i32 {{
    var growing = "";
    var index = 0;
    while index < 32 limit Iterations(64) {{
        growing = growing + {piece};
        index = index + 1;
    }}
    let original = growing;
    let source = growing + "AΣ́";
    let lower = lowercase(source);
    let upper = uppercase(source);
    growing = growing + "!";
    checkpoint("unicode-case");
    if lower == {lower} && upper == {upper} && original == {original}
        && source == {input} && growing == {appended} {{ return 0; }}
    return 1;
}}
"#,
            piece = quoted(piece),
            lower = quoted(&input.to_lowercase()),
            upper = quoted(&input.to_uppercase()),
            original = quoted(&original),
            input = quoted(&input),
            appended = quoted(&format!("{original}!"))
        ),
    );
    let checkpoints = temp_path("unicode-case-checkpoints", "dir");
    let trace = temp_path("unicode-case-trace", "json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&source),
            "--checkpoint-dir",
            path(&checkpoints),
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    let saved = checkpoints.join("checkpoint-0.json");
    let before = fs::read(&saved).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["resume", "0", "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    assert_eq!(
        fs::read(saved).unwrap(),
        before,
        "resume changed its input artifact"
    );
    let (code, stdout, stderr) = run_process(["replay", path(&trace)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
}

#[test]
fn checked_text_queries_and_parse_errors_survive_cli_checkpoint_resume() {
    let source = fixture(
        "checked-text-queries",
        r#"
module tests.cli.checked_text_queries;
import std.text.{parse_i32, contains, starts_with, ends_with, len, trim, lines, split, join};
import std.runtime.checkpoint;
flow main(args: Array<string>) -> i32 {
    var text = "start";
    var index = 0;
    while index < 256 limit Iterations(512) {
        text = text + "中😀";
        index = index + 1;
    }
    let alias = text;
    text = text + "end";
    let error = parse_i32("not-an-integer");
    let number = parse_i32("000" + "42");
    let line_prefix = "  A\r";
    let line_parts = lines(trim(line_prefix + "\n中😀\n\ré  "));
    let split_prefix = "中😀:";
    let separator_prefix = ":";
    let separated = split(split_prefix + ":tail::", separator_prefix + ":");
    let combined = join(separated, "-" + ":");
    checkpoint("checked-text-queries");
    let invalid = match error { Ok(_) => false, Err(_) => true };
    let valid = match number { Ok(value) => value == 42, Err(_) => false };
    if invalid && valid && len(alias) == 517 && len(text) == 520
        && starts_with(text, "start中") && ends_with(text, "😀end")
        && contains(text, "😀end") && !contains(text, "missing")
        && text[5] == '中' && text[6] == '😀'
        && line_parts == ["A", "中😀", "\ré"]
        && separated == ["中😀", "tail", ""] && combined == "中😀-:tail-:" { return 0; }
    return 1;
}
"#,
    );
    let checkpoints = temp_path("checked-text-query-checkpoints", "dir");
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["run", path(&source), "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    let saved = checkpoints.join("checkpoint-0.json");
    let before = fs::read(&saved).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["resume", "0", "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    assert_eq!(fs::read(&saved).unwrap(), before);
}

#[test]
fn shared_text_reassignment_survives_real_cli_checkpoint_resume_and_report() {
    let source = fixture(
        "shared-text-reassignment",
        r#"
module tests.cli.shared_text;
import std.text.len;
import std.runtime.checkpoint;
effect Append { action suffix() -> string; }
flow main(args: Array<string>) -> i32 {
    var text = "";
    var index = 0;
    while index < 256 limit Iterations(512) {
        text = text + "中😀";
        index = index + 1;
    }
    let alias = text;
    handle { text = text + perform Append.suffix(); } with {
        Append.suffix() => { checkpoint("pending-text-append"); resume "!"; }
    };
    if len(text) == 513 && len(alias) == 512 { return 0; }
    return 1;
}
"#,
    );
    let checkpoints = temp_path("shared-text-checkpoints", "dir");
    let trace = temp_path("shared-text-trace", "json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&source),
            "--checkpoint-dir",
            path(&checkpoints),
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    let saved = checkpoints.join("checkpoint-0.json");
    let before = fs::read(&saved).unwrap();
    let artifact = read_checkpoint_file(&saved);
    assert!(artifact.to_string().contains("中😀"));
    assert!(
        !artifact.to_string().contains("Concat"),
        "internal text representation is not a wire format"
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["resume", "0", "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    assert_eq!(fs::read(&saved).unwrap(), before);
    let (code, stdout, stderr) = run_process(["replay", path(&trace)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("replayed interpreter report"), "{stdout}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
}

#[test]
fn segmented_text_membership_and_order_survive_real_cli_checkpoint_resume() {
    let source = fixture(
        "segmented-text-membership",
        r#"
module tests.cli.segmented_text_membership;
import std.runtime.checkpoint;
flow main(args: Array<string>) -> i32 {
    let prefix = "中😀";
    let key = prefix + "é";
    let map = {key => 7};
    let set = #{key, "中😀é"};
    let ordered = OrderedMap.new<string, i32>().insert(key, 9).insert("a", 1);
    let kept = Some(key);
    checkpoint("segmented-text-membership");
    let other = "中" + "😀é";
    var count = 0;
    for item in set limit Iterations(4) {
        if item != other { return 1; }
        count = count + 1;
    }
    if key == other && key != "中😀é" && "a" < key && kept == Some(other)
        && map.get(other) == Some(7) && map.get("missing") == None
        && count == 1
        && ordered.get(other) == Some(9) && ordered.get("a") == Some(1) {
        return 0;
    }
    return 1;
}
"#,
    );
    let checkpoints = temp_path("segmented-text-membership-checkpoints", "dir");
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["run", path(&source), "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    let saved = checkpoints.join("checkpoint-0.json");
    let before = fs::read(&saved).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["resume", "0", "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    assert_eq!(fs::read(&saved).unwrap(), before);
}
