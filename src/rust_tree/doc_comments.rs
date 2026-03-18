use std::fs;
use std::path::Path;

pub fn format_doc_comment_for_path(path: &Path) -> Option<String> {
    if path.is_dir() {
        return ["index.ts", "index.tsx"]
            .iter()
            .map(|name| path.join(name))
            .find_map(|index_path| format_doc_comment_for_file(&index_path));
    }

    format_doc_comment_for_file(path)
}

fn format_doc_comment_for_file(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    if !matches!(ext, "ts" | "tsx") {
        return None;
    }

    let contents = fs::read_to_string(path).ok()?;
    let doc = extract_leading_tsdoc(&contents)?;
    Some(format!(" /** {doc} */"))
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
}
