mod support;

use etas_hir::{HirDumpOptions, ResolveResult, SymbolKind, lower_source};
use etas_syntax::{DiagnosticCode, NameDiagnosticCode};
use support::{fixture, read_source};

#[test]
fn hir_lowers_fixture_into_symbols_scopes_and_resolved_paths() {
    let hir = lower_source(read_source(&fixture("hir/positive/phase0_minimal.es")));

    assert!(
        hir.diagnostics.is_empty(),
        "HIR diagnostics: {:#?}",
        hir.diagnostics
    );

    let kinds = hir
        .symbols
        .iter()
        .map(|symbol| (&symbol.name, symbol.kind))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&(&"Draft".to_string(), SymbolKind::TypeAlias)));
    assert!(kinds.contains(&(&"Writer".to_string(), SymbolKind::Agent)));
    assert!(kinds.contains(&(&"main".to_string(), SymbolKind::Flow)));
    assert!(kinds.contains(&(&"draft".to_string(), SymbolKind::Local)));
}

#[test]
fn hir_resolves_pipeline_input_and_stage_without_type_checking() {
    let hir = lower_source(read_source(&fixture("hir/positive/phase0_minimal.es")));
    assert!(hir.diagnostics.is_empty(), "{:#?}", hir.diagnostics);

    let resolved_path_count = hir
        .exprs
        .iter()
        .filter(|(_, expr)| match expr {
            etas_hir::HirExpr::Path(path) => matches!(path.resolution, ResolveResult::Resolved(_)),
            _ => false,
        })
        .count();

    assert!(resolved_path_count >= 2);
}

#[test]
fn hir_dump_for_phase0_fixture_matches_golden_output() {
    let hir = lower_source(read_source(&fixture("hir/positive/phase0_minimal.es")));
    assert!(hir.diagnostics.is_empty(), "{:#?}", hir.diagnostics);

    let actual = etas_hir::dump_hir(
        &hir,
        HirDumpOptions {
            include_spans: false,
            include_diagnostics: true,
            include_symbols: true,
            include_scopes: true,
            include_source_map: false,
        },
    );

    assert_eq!(
        actual,
        include_str!("../fixtures/hir/golden/phase0_minimal.hir.txt")
    );
}

#[test]
fn hir_reports_name_resolution_failures_without_syntax_errors() {
    let hir = lower_source(read_source(&fixture(
        "hir/negative/name_unresolved_pipeline_stage.es",
    )));

    assert!(hir.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::Name(NameDiagnosticCode::UnresolvedName)
            && diagnostic.message.contains("MissingWriter")
    }));
    assert!(
        !hir.diagnostics
            .iter()
            .any(|diagnostic| matches!(diagnostic.code, DiagnosticCode::Syntax(_)))
    );
}
