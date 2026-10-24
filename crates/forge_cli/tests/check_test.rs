use forge_cli::{parse_tsc_output, run_typecheck};
use forge_protocol::ScriptDiagnostic;

#[test]
fn test_forge_check_json_output_format() {
    let sample_tsc_output = r#"
scripts/player.ts(10,5): error TS2322: Type 'string' is not assignable to type 'number'.
scripts/coin.ts(20,12): warning TS6133: 'unusedVar' is declared but its value is never read.
"#;
    let diagnostics = parse_tsc_output(sample_tsc_output);
    assert_eq!(diagnostics.len(), 2);

    let json_str = serde_json::to_string_pretty(&diagnostics).expect("serialize diagnostics");
    let deserialized: Vec<ScriptDiagnostic> =
        serde_json::from_str(&json_str).expect("deserialize diagnostics");

    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized[0].file, "scripts/player.ts");
    assert_eq!(deserialized[0].line, 10);
    assert_eq!(deserialized[0].column, 5);
    assert_eq!(deserialized[0].code, "TS2322");
    assert_eq!(
        deserialized[0].message,
        "Type 'string' is not assignable to type 'number'."
    );
    assert_eq!(deserialized[0].severity, "error");

    assert_eq!(deserialized[1].file, "scripts/coin.ts");
    assert_eq!(deserialized[1].severity, "warning");
    assert_eq!(deserialized[1].code, "TS6133");
}

#[test]
fn test_forge_check_on_template_script_demo() {
    // When run on templates/script-demo which has valid scripts and forge.d.ts:
    if std::path::Path::new("templates/script-demo/tsconfig.json").exists() {
        let res = run_typecheck(Some("templates/script-demo"));
        if let Ok((success, diags)) = res {
            assert!(
                success,
                "templates/script-demo should pass typecheck without errors, but got: {:?}",
                diags
            );
            assert!(diags.is_empty());
        }
    }
}
