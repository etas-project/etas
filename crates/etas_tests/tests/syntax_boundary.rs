mod support;

use etas_syntax::{
    DiagnosticCode, DumpOptions, Keyword, SyntaxDiagnosticCode, TokenKind, dump_ast, lex,
    parse_program,
};
use support::{fixture, fixtures_under, read_source};

#[test]
fn syntax_public_api_parses_fixture_without_semantic_resolution() {
    let source = read_source(&fixture("syntax/valid/declarations/phase0_minimal.es"));
    let parsed = parse_program(source.clone());

    assert!(
        parsed.diagnostics.is_empty(),
        "parse diagnostics: {:#?}",
        parsed.diagnostics
    );
    assert_eq!(parsed.tokens.source, source.id);
    assert_eq!(parsed.value.items.len(), 3);

    let dump = dump_ast(
        &parsed.value,
        DumpOptions {
            include_spans: false,
            include_tokens: true,
            include_diagnostics: false,
        },
    );

    assert!(dump.contains("Program"));
    assert!(dump.contains("TypeDecl"));
    assert!(dump.contains("AgentDecl name=Writer"));
    assert!(dump.contains("PipelineExpr"));
    assert!(!dump.contains("Symbols"));
    assert!(!dump.contains("Resolved"));
}

#[test]
fn syntax_lexer_keeps_surface_tokens_and_trivia_only() {
    let source = read_source(&fixture("syntax/valid/declarations/phase0_minimal.es"));
    let tokens = lex(&source).tokens;

    assert!(
        tokens
            .iter()
            .any(|token| token.kind == TokenKind::Keyword(Keyword::Module))
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.kind == TokenKind::Keyword(Keyword::Flow))
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.kind == TokenKind::Whitespace)
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Ident))
    );
}

#[test]
fn syntax_recovery_returns_ast_value_and_syntax_diagnostic() {
    let source = read_source(&fixture(
        "syntax/recovery/expressions/invalid_missing_expression_001.es",
    ));
    let parsed = parse_program(source);

    assert!(parsed.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::Syntax(SyntaxDiagnosticCode::InvalidExpression)
    }));
    assert_eq!(parsed.value.items.len(), 1);

    let dump = dump_ast(
        &parsed.value,
        DumpOptions {
            include_spans: false,
            include_tokens: true,
            include_diagnostics: false,
        },
    );

    assert!(dump.contains("FlowDecl name=broken"));
    assert!(dump.contains("ErrorExpr"));
}

#[test]
fn syntax_fixture_corpus_has_at_least_one_hundred_files() {
    let fixtures = fixtures_under("syntax");

    assert!(
        fixtures.len() >= 100,
        "expected at least 100 syntax fixtures, got {}",
        fixtures.len()
    );
}

#[test]
fn syntax_module_import_fixtures_cover_current_spec_forms() {
    let fixtures = fixtures_under("syntax/valid/modules");
    assert_eq!(
        fixtures.len(),
        10,
        "expected 10 dedicated module/import syntax fixtures"
    );

    let corpus = fixtures
        .iter()
        .map(|fixture| std::fs::read_to_string(fixture).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    for expected in [
        "import std.io;",
        "import std.io as io;",
        "import std.io.println;",
        "import std.io.println as log;",
        "import std.io.{print, println, eprintln};",
        "import std.io.{println as log, read_line,};",
        "import std.io.*;",
        "public import std.io.{println, eprintln};",
        "public import std.prelude.*;",
    ] {
        assert!(corpus.contains(expected), "missing import form: {expected}");
    }

    for fixture in fixtures {
        let parsed = parse_program(read_source(&fixture));
        assert!(
            parsed.diagnostics.is_empty(),
            "{} produced diagnostics: {:#?}",
            fixture.display(),
            parsed.diagnostics
        );
    }
}

#[test]
fn syntax_valid_fixture_corpus_parses_without_diagnostics() {
    let fixtures = fixtures_under("syntax/valid");
    assert!(!fixtures.is_empty());

    for fixture in fixtures {
        let parsed = parse_program(read_source(&fixture));
        assert!(
            parsed.diagnostics.is_empty(),
            "{} produced diagnostics: {:#?}",
            fixture.display(),
            parsed.diagnostics
        );
        assert!(
            !parsed.value.items.is_empty(),
            "{} produced an empty program",
            fixture.display()
        );
    }
}

#[test]
fn syntax_recovery_fixture_corpus_reports_syntax_diagnostics() {
    let fixtures = fixtures_under("syntax/recovery");
    assert!(!fixtures.is_empty());

    for fixture in fixtures {
        let parsed = parse_program(read_source(&fixture));
        assert!(
            parsed
                .diagnostics
                .iter()
                .any(|diagnostic| matches!(diagnostic.code, DiagnosticCode::Syntax(_))),
            "{} did not report a syntax diagnostic",
            fixture.display()
        );
    }
}

#[test]
fn syntax_recovery_fixtures_reject_obsolete_policy_surface() {
    let cases = [
        (
            "syntax/recovery/items/obsolete_policy_declaration_001.es",
            "`policy` declarations are obsolete",
        ),
        (
            "syntax/recovery/items/obsolete_flow_follows_clause_001.es",
            "`follows` is obsolete",
        ),
    ];

    for (relative, message) in cases {
        let path = fixture(relative);
        let parsed = parse_program(read_source(&path));
        assert!(
            parsed.diagnostics.iter().any(|diagnostic| {
                matches!(diagnostic.code, DiagnosticCode::Syntax(_))
                    && diagnostic.message.contains(message)
            }),
            "{} did not report the expected obsolete syntax diagnostic `{message}`: {:#?}",
            path.display(),
            parsed.diagnostics
        );
    }
}
