use glob::Pattern;
use rust_tree::rust_tree::options::TreeOptions;
use rust_tree::rust_tree::traversal::list_directory_as_string;
use std::fs;
use tempfile::tempdir;

fn create_default_options() -> TreeOptions {
    TreeOptions {
        all_files: false,
        level: None,
        full_path: false,
        dir_only: false,
        no_indent: false,
        print_size: false,
        human_readable: false,
        pattern_glob: vec![],
        exclude_patterns: vec![],
        color: false,
        no_color: false,
        ascii: false,
        sort_by_time: false,
        reverse: false,
        print_mod_date: false,
        output_file: None,
        file_limit: None,
        dirs_first: false,
        classify: false,
        no_report: false,
        print_permissions: false,
        from_file: false,
        icons: false,
        doc: false,
        prune: false,
        match_dirs: false,
        gitignore: false,
    }
}

#[test]
fn test_doc_shows_tsdoc_for_ts_file() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("api.ts"),
        "/** API surface */\nexport const api = true;\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(output.contains("api.ts /** API surface */"), "{output}");
}

#[test]
fn test_doc_shows_index_tsdoc_for_directory() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("feature")).unwrap();
    fs::write(
        dir.path().join("feature").join("index.ts"),
        "/** Feature module */\nexport * from './impl';\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("feature").join("impl.ts"),
        "/** Implementation */\nexport const impl_value = 1;\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(output.contains("feature\n"), "{output}");
    assert!(!output.contains("feature /** Feature module */"), "{output}");
    assert!(output.contains("index.ts /** Feature module */"), "{output}");
}

#[test]
fn test_doc_shows_python_module_docstring_for_file() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("api.py"),
        "# Leading comment is allowed\n\n\"\"\"API surface\"\"\"\napi = True\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(
        output.contains("api.py \"\"\" API surface \"\"\""),
        "{output}"
    );
}

#[test]
fn test_doc_shows_package_docstring_for_directory() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("feature")).unwrap();
    fs::write(
        dir.path().join("feature").join("__init__.py"),
        "\"\"\"Feature package\"\"\"\nfrom .impl import impl_value\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("feature").join("impl.py"),
        "\"\"\"Implementation\"\"\"\nimpl_value = 1\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(output.contains("feature\n"), "{output}");
    assert!(!output.contains("feature \"\"\" Feature package \"\"\""), "{output}");
    assert!(output.contains("__init__.py \"\"\" Feature package \"\"\""), "{output}");
}

#[test]
fn test_doc_shows_rust_module_docs_for_file() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("api.rs"),
        "//! API surface\n//! for tree output.\n\npub fn run() {}\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(
        output.contains("api.rs\n    //! API surface\n    //! for tree output."),
        "{output}"
    );
}

#[test]
fn test_doc_shows_mod_rs_docs_for_directory() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("feature")).unwrap();
    fs::write(
        dir.path().join("feature").join("mod.rs"),
        "//! Feature module\n//! for tree output.\n\npub fn run() {}\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(output.contains("feature\n"), "{output}");
    assert!(
        !output.contains("feature\n    //! Feature module\n    //! for tree output."),
        "{output}"
    );
    assert!(
        output.contains("mod.rs\n        //! Feature module\n        //! for tree output."),
        "{output}"
    );
}

#[test]
fn test_doc_keeps_directory_doc_when_entry_file_is_not_visible() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("feature")).unwrap();
    fs::write(
        dir.path().join("feature").join("mod.rs"),
        "//! Feature module\n//! for tree output.\n\npub fn run() {}\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.level = Some(1);
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(
        output.contains("feature\n    //! Feature module\n    //! for tree output."),
        "{output}"
    );
    assert!(!output.contains("mod.rs"), "{output}");
}

#[test]
fn test_doc_shows_combined_directory_docs_when_only_some_entry_files_are_visible() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("feature")).unwrap();
    fs::write(
        dir.path().join("feature").join("index.ts"),
        "/** TypeScript module */\nexport const impl_value = 1;\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("feature").join("__init__.py"),
        "\"\"\"Python package\"\"\"\nimpl_value = 1\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.pattern_glob = vec![Pattern::new("*.ts").unwrap()];
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(
        output.contains("feature\n    /** TypeScript module */\n    \n    \"\"\" Python package \"\"\""),
        "{output}"
    );
    assert!(output.contains("index.ts /** TypeScript module */"), "{output}");
    assert!(!output.contains("__init__.py"), "{output}");
}

#[test]
fn test_doc_keeps_multiple_entry_file_docs_on_files_when_all_are_visible() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("feature")).unwrap();
    fs::write(
        dir.path().join("feature").join("index.ts"),
        "/** TypeScript module */\nexport const impl_value = 1;\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("feature").join("__init__.py"),
        "\"\"\"Python package\"\"\"\nimpl_value = 1\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(output.contains("feature\n"), "{output}");
    assert!(!output.contains("feature /** TypeScript module */"), "{output}");
    assert!(!output.contains("feature \"\"\" Python package \"\"\""), "{output}");
    assert!(output.contains("index.ts /** TypeScript module */"), "{output}");
    assert!(output.contains("__init__.py \"\"\" Python package \"\"\""), "{output}");
}

#[test]
fn test_doc_keeps_single_line_docs_inline() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("main.rs"),
        "//! Command-line binary entry point for `tree`.\nfn main() {}\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(
        output.contains("main.rs //! Command-line binary entry point for `tree`."),
        "{output}"
    );
}

#[test]
fn test_doc_ignores_non_leading_tsdoc() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("api.ts"),
        "// Not a tsdoc header\n/** API surface */\nexport const api = true;\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(!output.contains("/** API surface */"), "{output}");
}

#[test]
fn test_doc_ignores_non_leading_python_docstring() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("api.py"),
        "import os\n\"\"\"API surface\"\"\"\napi = True\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(!output.contains("\"\"\" API surface \"\"\""), "{output}");
}

#[test]
fn test_doc_ignores_non_leading_rust_module_docs() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("api.rs"),
        "// Regular comment\n//! API surface\npub fn run() {}\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(!output.contains("//! API surface"), "{output}");
}

#[test]
fn test_doc_allows_bom_and_shebang_before_tsdoc() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("cli.ts"),
        "\u{feff}#!/usr/bin/env node\n/** CLI entrypoint */\nconsole.log('hi');\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(output.contains("cli.ts /** CLI entrypoint */"), "{output}");
}

#[test]
fn test_doc_is_green_when_color_enabled() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("api.ts"),
        "/** API surface */\nexport const api = true;\n",
    )
    .unwrap();

    let mut options = create_default_options();
    options.doc = true;
    options.color = true;
    options.no_report = true;

    let output = list_directory_as_string(dir.path(), &options).unwrap();
    assert!(output.contains("api.ts"));
    assert!(
        output.contains(" \u{1b}[32m/** API surface */\u{1b}[0m"),
        "{output}"
    );
}
