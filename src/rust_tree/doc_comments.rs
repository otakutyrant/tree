//! Extraction of leading documentation comments for supported languages.

use std::fs;
use std::path::Path;

type DocExtractor = fn(&str) -> Option<String>;
type DocFormatter = fn(&str) -> String;

struct LanguageDocSupport {
    extensions: &'static [&'static str],
    directory_doc_entry_files: Option<&'static [&'static str]>,
    extract_file_doc: DocExtractor,
    format: DocFormatter,
}

const LANGUAGE_DOCS: &[LanguageDocSupport] = &[
    LanguageDocSupport {
        extensions: &["ts", "tsx"],
        directory_doc_entry_files: Some(&["index.ts", "index.tsx"]),
        extract_file_doc: extract_leading_tsdoc,
        format: format_tsdoc,
    },
    LanguageDocSupport {
        extensions: &["py"],
        directory_doc_entry_files: Some(&["__init__.py"]),
        extract_file_doc: extract_module_docstring,
        format: format_python_docstring,
    },
    LanguageDocSupport {
        extensions: &["rs"],
        directory_doc_entry_files: Some(&["mod.rs"]),
        extract_file_doc: extract_rust_module_doc,
        format: format_rust_module_doc,
    },
];

pub fn format_doc_comment_for_path(path: &Path) -> Option<String> {
    if path.is_dir() {
        return format_doc_comment_for_directory(path);
    }

    format_doc_comment_for_file(path)
}

fn format_doc_comment_for_directory(path: &Path) -> Option<String> {
    LANGUAGE_DOCS
        .iter()
        .find_map(|language| {
            language
                .directory_doc_entry_files?
                .iter()
                .map(|name| path.join(name))
                .find_map(|entry_path| format_doc_comment_for_file(&entry_path))
        })
}

fn format_doc_comment_for_file(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    let contents = fs::read_to_string(path).ok()?;
    let language = find_language_doc_support(ext)?;
    let doc = (language.extract_file_doc)(&contents)?;

    Some((language.format)(&doc))
}

fn find_language_doc_support(ext: &str) -> Option<&'static LanguageDocSupport> {
    LANGUAGE_DOCS
        .iter()
        .find(|language| language.extensions.contains(&ext))
}

fn format_tsdoc(doc: &str) -> String {
    format!(" /** {doc} */")
}

fn format_python_docstring(doc: &str) -> String {
    format!(" \"\"\" {doc} \"\"\"")
}

fn format_rust_module_doc(doc: &str) -> String {
    format!(" //! {doc}")
}

fn extract_leading_tsdoc(contents: &str) -> Option<String> {
    let contents = contents.strip_prefix('\u{feff}').unwrap_or(contents);
    let contents = strip_shebang(contents);

    if !contents.starts_with("/**") {
        return None;
    }

    let end = contents.find("*/")?;
    let block = &contents[..end + 2];
    normalize_tsdoc_block(block)
}

fn strip_shebang(contents: &str) -> &str {
    if let Some(rest) = contents.strip_prefix("#!") {
        if let Some(newline_pos) = rest.find('\n') {
            return &rest[newline_pos + 1..];
        }
        return "";
    }

    contents
}

fn normalize_tsdoc_block(block: &str) -> Option<String> {
    let inner = block
        .strip_prefix("/**")?
        .strip_suffix("*/")?
        .trim_matches(|c: char| c == '\r' || c == '\n');

    let parts: Vec<&str> = inner
        .lines()
        .map(|line| line.trim())
        .map(|line| line.strip_prefix('*').unwrap_or(line).trim())
        .filter(|line| !line.is_empty())
        .collect();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn extract_module_docstring(contents: &str) -> Option<String> {
    let contents = contents.strip_prefix('\u{feff}').unwrap_or(contents);
    let contents = strip_shebang(contents);
    let contents = skip_python_leading_trivia(contents);
    let (literal, _) = parse_python_string_literal(contents)?;
    normalize_python_docstring(literal)
}

fn skip_python_leading_trivia(mut contents: &str) -> &str {
    loop {
        let trimmed = contents.trim_start_matches([' ', '\t', '\r', '\n']);
        if trimmed.len() != contents.len() {
            contents = trimmed;
            continue;
        }

        let line = contents.trim_start_matches([' ', '\t']);
        if let Some(rest) = line.strip_prefix('#') {
            if let Some(newline_pos) = rest.find('\n') {
                contents = &rest[newline_pos + 1..];
            } else {
                return "";
            }
            continue;
        }

        return contents;
    }
}

fn parse_python_string_literal(contents: &str) -> Option<(&str, &str)> {
    let bytes = contents.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() && bytes[idx].is_ascii_alphabetic() {
        idx += 1;
    }

    let prefix = &contents[..idx];
    if prefix.chars().any(|c| matches!(c, 'b' | 'B' | 'f' | 'F')) {
        return None;
    }

    let rest = &contents[idx..];
    let quote = rest.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }

    if rest.starts_with("\"\"\"") || rest.starts_with("'''") {
        let delimiter = &rest[..3];
        let body = &rest[3..];
        let end = body.find(delimiter)?;
        let literal = &body[..end];
        let remaining = &body[end + 3..];
        Some((literal, remaining))
    } else {
        let mut escaped = false;
        for (pos, ch) in rest[1..].char_indices() {
            if ch == '\n' {
                return None;
            }
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                let end = pos + 2;
                let literal = &rest[1..end - 1];
                let remaining = &rest[end..];
                return Some((literal, remaining));
            }
        }
        None
    }
}

fn normalize_python_docstring(literal: &str) -> Option<String> {
    let parts: Vec<&str> = literal
        .trim_matches(|c: char| c == '\r' || c == '\n')
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn extract_rust_module_doc(contents: &str) -> Option<String> {
    let contents = contents.strip_prefix('\u{feff}').unwrap_or(contents);
    let contents = strip_shebang(contents);
    let contents = contents.trim_start_matches([' ', '\t', '\r', '\n']);

    if contents.starts_with("//!") {
        return normalize_rust_line_docs(contents);
    }

    if contents.starts_with("/*!") {
        return normalize_rust_block_doc(contents);
    }

    None
}

fn normalize_rust_line_docs(contents: &str) -> Option<String> {
    let mut parts = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim_start();
        if let Some(doc_line) = trimmed.strip_prefix("//!") {
            let doc_line = doc_line.trim();
            if !doc_line.is_empty() {
                parts.push(doc_line);
            }
            continue;
        }

        if trimmed.is_empty() {
            if parts.is_empty() {
                continue;
            }
        }
        break;
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn normalize_rust_block_doc(contents: &str) -> Option<String> {
    let end = contents.find("*/")?;
    let block = &contents[..end + 2];
    let inner = block
        .strip_prefix("/*!")?
        .strip_suffix("*/")?
        .trim_matches(|c: char| c == '\r' || c == '\n');

    let parts: Vec<&str> = inner
        .lines()
        .map(|line| line.trim())
        .map(|line| line.strip_prefix('*').unwrap_or(line).trim())
        .filter(|line| !line.is_empty())
        .collect();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn extracts_single_line_tsdoc() {
        assert_eq!(
            extract_leading_tsdoc("/** Hello world */\nexport const x = 1;"),
            Some("Hello world".to_string())
        );
    }

    #[test]
    fn extracts_multiline_tsdoc() {
        assert_eq!(
            extract_leading_tsdoc("/**\n * Hello\n * world\n */\nexport const x = 1;"),
            Some("Hello world".to_string())
        );
    }

    #[test]
    fn allows_bom_and_shebang_before_tsdoc() {
        assert_eq!(
            extract_leading_tsdoc("\u{feff}#!/usr/bin/env node\n/** Hi */\nconsole.log('x');"),
            Some("Hi".to_string())
        );
    }

    #[test]
    fn rejects_other_content_before_tsdoc() {
        assert_eq!(
            extract_leading_tsdoc("// comment\n/** Hi */\nconsole.log('x');"),
            None
        );
    }

    #[test]
    fn ignores_non_ts_files() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "/** Not used */").unwrap();
        let md_path = file.path().with_extension("md");
        fs::copy(file.path(), &md_path).unwrap();
        assert_eq!(format_doc_comment_for_file(&md_path), None);
    }

    #[test]
    fn extracts_python_module_docstring() {
        assert_eq!(
            extract_module_docstring("\"\"\"Hello world\"\"\"\nvalue = 1\n"),
            Some("Hello world".to_string())
        );
    }

    #[test]
    fn extracts_python_module_docstring_after_comments() {
        assert_eq!(
            extract_module_docstring(
                "#!/usr/bin/env python3\n# coding: utf-8\n\n\"\"\"Hello\nworld\"\"\"\nvalue = 1\n"
            ),
            Some("Hello world".to_string())
        );
    }

    #[test]
    fn rejects_non_leading_python_docstring() {
        assert_eq!(
            extract_module_docstring("import os\n\"\"\"Not a module docstring\"\"\"\n"),
            None
        );
    }

    #[test]
    fn extracts_rust_line_module_docs() {
        assert_eq!(
            extract_rust_module_doc(
                "//! Core tree traversal.\n//! Renders output.\n\npub fn run() {}\n"
            ),
            Some("Core tree traversal. Renders output.".to_string())
        );
    }

    #[test]
    fn extracts_rust_block_module_docs() {
        assert_eq!(
            extract_rust_module_doc(
                "/*!\n * Core tree traversal.\n * Renders output.\n */\npub fn run() {}\n"
            ),
            Some("Core tree traversal. Renders output.".to_string())
        );
    }

    #[test]
    fn rejects_non_leading_rust_docs() {
        assert_eq!(
            extract_rust_module_doc(
                "// regular comment\n//! Not module docs here\npub fn run() {}\n"
            ),
            None
        );
    }
}
