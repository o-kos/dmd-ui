//! Generic hygiene checks for outgoing commit messages, filenames and added text.

use std::{
    io::{self, Read},
    process::Command,
};

const HOSTS: &[&str] = &[
    "github.com",
    "crates.io",
    "index.crates.io",
    "doc.rust-lang.org",
    "keepachangelog.com",
];
// Bare names use common DNS suffixes to distinguish hosts from source expressions.
// URLs are checked for every scheme and suffix, independently of this heuristic.
const DNS_SUFFIXES: &[&str] = &[
    "com", "org", "net", "io", "dev", "app", "edu", "gov", "co", "uk", "de", "fr", "ru", "info",
    "biz", "xyz", "invalid",
];
const EXTENSIONS: &[&str] = &[
    "rs", "md", "toml", "lock", "yml", "yaml", "json", "h", "c", "cpp", "txt", "sh", "exe", "dll",
    "so", "dylib", "a", "lib",
];

fn escapes_tree(value: &str) -> bool {
    let mut depth = 0;
    for part in value.split(['/', '\\']) {
        match part {
            ".." if depth == 0 => return true,
            ".." => depth -= 1,
            "" | "." => (),
            _ => depth += 1,
        }
    }
    false
}

fn host_like(value: &str) -> bool {
    let labels: Vec<_> = value.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|label| {
            !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
        && labels.last().is_some_and(|suffix| {
            suffix.len() >= 2 && suffix.chars().all(|c| c.is_ascii_alphabetic())
        })
}

fn token_violation(token: &str) -> Option<&'static str> {
    let token = token.trim_end_matches(['.', ',', ':', '!', '?']);
    if token.is_empty() {
        return None;
    }
    if let Some((scheme, address)) = token.split_once("://")
        && !scheme.is_empty()
        && !address.is_empty()
    {
        let host = address.split(['/', ':', '?', '#']).next().unwrap_or("");
        return (!HOSTS.contains(&host)).then_some("unapproved URL host");
    }
    if let Some((user, host)) = token.split_once('@') {
        if !user.is_empty() && host_like(host) {
            return Some("e-mail address");
        }
    }
    let drive = token.as_bytes()[0].is_ascii_alphabetic()
        && token.as_bytes().get(1) == Some(&b':')
        && token
            .as_bytes()
            .get(2)
            .is_some_and(|c| matches!(c, b'/' | b'\\'));
    let unc = token
        .strip_prefix("\\\\")
        .and_then(|rest| rest.split_once(['\\', '/']))
        .is_some_and(|(host, _)| !host.is_empty());
    if drive
        || token.starts_with('/')
        || unc
        || token.starts_with('~') && token.as_bytes().get(1) == Some(&b'/')
    {
        // Standalone comment delimiters are syntax, and the null device is the same on
        // every Unix host, so neither discloses anything about an environment.
        if !matches!(token, "/" | "//" | "///" | "/*" | "/**" | "/dev/null") {
            return Some("absolute filesystem path");
        }
    }
    if escapes_tree(token) {
        return Some("path escapes the working tree");
    }
    let host = token.split(['/', ':']).next().unwrap_or("");
    if host_like(host)
        && host
            .rsplit('.')
            .next()
            .is_some_and(|suffix| DNS_SUFFIXES.contains(&suffix))
        && !HOSTS.contains(&host)
        && !host
            .rsplit('.')
            .next()
            .is_some_and(|ext| EXTENSIONS.contains(&ext))
        && !host.contains('_')
    {
        return Some("unapproved host");
    }
    None
}

fn violation(text: &str) -> Option<&'static str> {
    text.split(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '"' | '\''
                    | '`'
                    | '<'
                    | '>'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '='
                    | ';'
                    | ','
                    | '|'
            )
    })
    .find_map(token_violation)
}

fn git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("git {} failed", args[0]));
    }
    String::from_utf8(output.stdout).map_err(|_| "non-UTF-8 content requires manual review".into())
}

fn check_commit(commit: &str) -> Result<(), String> {
    let message = git(&["show", "-s", "--format=%B", commit])?;
    if let Some(reason) = violation(&message) {
        return Err(format!("commit {commit}: {reason} in message"));
    }
    let names = git(&[
        "diff-tree",
        "--root",
        "--no-commit-id",
        "--name-only",
        "-r",
        "-z",
        commit,
    ])?;
    for name in names.split('\0').filter(|name| !name.is_empty()) {
        if let Some(reason) = violation(name) {
            return Err(format!("commit {commit}: {reason} in filename"));
        }
    }
    let diff = git(&[
        "show",
        "--format=",
        "--root",
        "--first-parent",
        "--no-ext-diff",
        "--no-textconv",
        "--unified=0",
        commit,
    ])?;
    check_added_text(commit, &diff)
}

fn check_added_text(commit: &str, diff: &str) -> Result<(), String> {
    for (number, line) in diff
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with('+') && !line.starts_with("+++"))
    {
        if let Some(reason) = violation(&line[1..]) {
            return Err(format!(
                "commit {commit}: {reason} in added text at diff line {}",
                number + 1
            ));
        }
    }
    Ok(())
}

fn new_branch_history(remote: &str, commit: &str) -> Result<String, String> {
    let refs = git(&[
        "for-each-ref",
        "--format=%(objectname)",
        &format!("refs/remotes/{remote}/"),
    ])?;
    let mut args = vec!["rev-list", commit, "--not"];
    args.extend(refs.lines());
    git(&args)
}

fn check_updates(remote: &str, input: &str) -> Result<(), String> {
    let mut commits = std::collections::BTreeSet::new();
    for line in input.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 4 {
            return Err("invalid pre-push ref update".into());
        }
        if fields[2] == "refs/heads/main" {
            return Err("direct pushes to main are refused; use a Pull Request".into());
        }
        if fields[1].chars().all(|c| c == '0') {
            continue;
        }
        let history = if fields[3].chars().all(|c| c == '0') {
            new_branch_history(remote, fields[1])?
        } else {
            git(&["rev-list", &format!("{}..{}", fields[3], fields[1])])?
        };
        commits.extend(history.lines().map(str::to_owned));
    }
    for commit in commits {
        check_commit(&commit)?;
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let remote = std::env::args().nth(1).ok_or("missing destination remote")?;
    check_updates(&remote, &input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_generic_private_content() {
        for value in [
            ["", "private", "capture"].join("/"),
            ["C:", "private", "capture"].join("\\"),
            ["..", "capture"].join("/"),
            ["person".to_owned(), ["mail", "invalid"].join(".")].join("@"),
            [
                "https:".to_owned(),
                String::new(),
                ["service", "invalid"].join("."),
                "data".into(),
            ]
            .join("/"),
            ["service", "invalid"].join("."),
        ] {
            assert!(violation(&value).is_some(), "missed {value}");
        }
    }

    #[test]
    fn permits_repository_paths_and_approved_urls() {
        for value in [
            "./foo",
            "crates/cli/src/main.rs",
            "Cargo.toml",
            "// Comment",
            "cmd < /dev/null",
            "https://github.com/actions/checkout",
        ] {
            assert_eq!(violation(value), None, "rejected {value}");
        }
    }

    #[test]
    fn distinguishes_backslash_text_from_absolute_windows_paths() {
        for value in [r"\", r"\n", r"\\n", r"\foo", r"\\host", r"C:foo"] {
            assert_eq!(violation(value), None, "rejected {value}");
        }
        for value in [
            ["", "", "host", "share"].join("\\"),
            ["C:", "foo"].join("\\"),
            ["C:", "foo"].join("/"),
        ] {
            assert_eq!(
                violation(&value),
                Some("absolute filesystem path"),
                "missed {value}"
            );
        }
    }

    #[test]
    fn reports_one_based_diff_line_numbers() {
        let added = format!("+{}", ["", "private", "capture"].join("/"));
        let diff = [
            "diff --git a/file b/file",
            "--- a/file",
            "+++ b/file",
            "@@ -1 +1,2 @@",
            "-old text",
            "+safe text",
            &added,
        ]
        .join("\n");
        for (text, line) in [(added.as_str(), 1), (diff.as_str(), 7)] {
            assert_eq!(
                check_added_text("test", text),
                Err(format!(
                    "commit test: absolute filesystem path in added text at diff line {line}"
                ))
            );
        }
    }

    #[test]
    fn refuses_main_including_deletion() {
        assert!(
            check_updates("origin", "refs/heads/topic 0000 refs/heads/main 1234")
                .unwrap_err()
                .contains("Pull Request")
        );
    }
}
