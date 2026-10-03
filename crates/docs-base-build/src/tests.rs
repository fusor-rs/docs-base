use crate::{Config, Highlighter, links::Links, markdown};
use std::path::Path;

#[test]
fn renders_markdown_and_preserves_explicit_and_repeated_heading_ids() {
    let document = parse(
        "# Documentation\n\nA **bold** and *emphasized* [guide](/docs/events).\n\n\
         ## First {#existing-link}\n\n1. One\n2. Two\n\n\
         | Name | Value |\n| --- | --- |\n| State | `Signal<T>` |\n\n\
         > A quote.\n\n- [x] Complete\n\n### Details\n\nText.\n\n### Details\n\nMore.\n\n\
         ### Read `Signal<T>`\n",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &Highlighter::default(),
    )
    .unwrap();
    assert_eq!(document.title, "Documentation");
    assert_eq!(
        document.lead,
        "<p>A <strong>bold</strong> and <em>emphasized</em> <a href=\"/docs/events\" title=\"\" data-fusor-link>guide</a>.</p>\n"
    );
    assert_eq!(document.sections[0].id, "existing-link");
    assert_eq!(document.sections[0].title, "First");
    let body = &document.sections[0].body.html;
    for expected in [
        "<ol>\n<li>One</li>\n<li>Two</li>\n</ol>",
        "<td><code class=\"inline-code\">Signal&lt;T&gt;</code></td>",
        "<blockquote>\n<p>A quote.</p>\n</blockquote>",
        "<input disabled=\"\" type=\"checkbox\" checked=\"\"/>",
        "<h3 id=\"details\">Details</h3>",
        "<h3 id=\"details-2\">Details</h3>",
        "<h3 id=\"read-signal-t\">Read <code class=\"inline-code\">Signal&lt;T&gt;</code></h3>",
    ] {
        assert!(body.contains(expected), "missing {expected}: {body}");
    }
    assert_eq!(
        document.anchors.into_iter().collect::<Vec<_>>(),
        ["details", "details-2", "existing-link", "read-signal-t"]
    );
    assert!(document.search.contains("Signal<T>"));
    assert!(
        document
            .search
            .starts_with("A bold and emphasized guide.\n")
    );
}

#[test]
fn renders_disclosures_but_escapes_authored_html_and_code() {
    let document = parse(
        "# Escaping\n\n## Example\n\n<details>\n<summary>Source</summary>\n\n\
         ```text title=Literal HTML\n<script>alert('hello')</script>\n```\n\n</details>\n\n\
         <img src=x onerror=alert(1)>\n\n`<App>` and ![Icon](/docs/favicon.svg).\n",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &Highlighter::default(),
    )
    .unwrap();
    let section = &document.sections[0];
    assert_eq!(section.body.code, "<script>alert('hello')</script>\n\n");
    assert!(
        section
            .body
            .html
            .contains("<details>\n<summary>Source</summary>")
    );
    assert!(
        section
            .body
            .html
            .contains("&lt;script&gt;alert(&#39;hello&#39;)&lt;/script&gt;")
    );
    assert!(
        section
            .body
            .html
            .contains("&lt;img src=x onerror=alert(1)&gt;")
    );
    assert!(
        section
            .body
            .html
            .contains("<code class=\"inline-code\">&lt;App&gt;</code>")
    );
    assert!(
        section
            .body
            .html
            .contains("<img src=\"/docs/favicon.svg\" alt=\"Icon\" />")
    );
}

#[test]
fn includes_source_files_in_rendering_and_search() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    let document = parse(
        "# Source\n\n## Counter\n\n```rust source=src/lib.rs title=src/watch.rs\n```\n",
        directory,
        &Highlighter::default(),
    )
    .unwrap();
    let authored = std::fs::read_to_string(directory.join("src/lib.rs")).unwrap();
    assert_eq!(document.sections[0].body.code, format!("{authored}\n"));
    assert!(document.search.contains(&authored));
    assert!(
        document.sections[0]
            .body
            .html
            .contains("aria-label=\"src/watch.rs\"")
    );
}

#[test]
fn rejects_broken_document_contracts() {
    let highlighter = Highlighter::default();
    for (source, message) in [
        ("No heading", "start each guide"),
        ("# Title\n\n# Another", "one # Title"),
        (
            "# Title\n\n## A {#same}\n\n### B {#same}",
            "duplicate heading id: same",
        ),
        ("# Title\n\n## A {#bad/id}", "invalid heading id"),
        (
            "# Title\n\n[Click](javascript:alert%281%29)",
            "unsupported documentation link scheme",
        ),
        (
            "# Title\n\n```rust source=missing.rs\nlet value = 1;\n```",
            "must be empty",
        ),
        ("# Title\n\n```rust source=missing.rs\n```", "missing.rs:"),
    ] {
        let error = parse(source, Path::new(env!("CARGO_MANIFEST_DIR")), &highlighter)
            .err()
            .expect("invalid guide must fail");
        assert!(error.to_string().contains(message), "{source}: {error}");
    }
}

fn parse(
    source: &str,
    directory: &Path,
    highlighter: &Highlighter,
) -> crate::Result<markdown::Document> {
    let config = Config {
        root: directory,
        content: Path::new("."),
        navigation: Path::new("navigation.json"),
        references: None,
        base_path: "/docs/",
        repository: None,
    };
    let path = directory.join("index.md");
    let links = Links {
        config: &config,
        current: &path,
        pages: &[],
    };
    markdown::parse(source, directory, highlighter, &links)
}

#[test]
fn rejects_invalid_navigation_and_reference_targets() {
    let root = std::env::temp_dir().join(format!(
        "docs-base-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("index.md"), "# Intro\n\n## Read {#read}\n").unwrap();
    let mut config = Config {
        root: &root,
        content: Path::new("."),
        navigation: Path::new("navigation.json"),
        references: None,
        base_path: "/manual/",
        repository: None,
    };
    let highlighter = Highlighter::default();
    assert_invalid_navigation(&config, &highlighter);
    std::fs::write(
        root.join("navigation.json"),
        r#"[{"slug":"","group":"Learn"}]"#,
    )
    .unwrap();
    config.references = Some(Path::new("references.json"));
    std::fs::write(
        root.join("references.json"),
        r#"[{"token":"Read","href":"/manual/#missing","label":"Read"}]"#,
    )
    .unwrap();
    let error = crate::compile(&config, &highlighter)
        .err()
        .expect("invalid reference");
    assert_eq!(
        error.to_string(),
        "unknown reference target: /manual/#missing"
    );
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn rejects_base_paths_that_cannot_be_used_for_local_routes() {
    let highlighter = Highlighter::default();
    for base_path in [
        "",
        "docs",
        "/docs",
        "//",
        "/../",
        "/docs//",
        "/docs?query/",
        "/docs/#fragment/",
    ] {
        let config = Config {
            root: Path::new(env!("CARGO_MANIFEST_DIR")),
            content: Path::new("."),
            navigation: Path::new("navigation.json"),
            references: None,
            base_path,
            repository: None,
        };
        let error = crate::compile(&config, &highlighter)
            .err()
            .expect("invalid base path");
        assert_eq!(
            error.to_string(),
            "documentation base_path must be / or /path/ using letters, digits, hyphens or underscores"
        );
    }
}

fn assert_invalid_navigation(config: &Config<'_>, highlighter: &Highlighter) {
    for (navigation, expected) in [
        ("[]", "navigation must start"),
        (
            r#"[{"slug":"topic","group":"Learn"}]"#,
            "navigation must start",
        ),
        (
            r#"[{"slug":"","group":"Learn"},{"slug":"","group":"Learn"}]"#,
            "duplicate page",
        ),
        (
            r#"[{"slug":"","group":"Learn","parent":"missing"}]"#,
            "unknown parent",
        ),
        (
            r#"[{"slug":"","group":"Learn","parent":""}]"#,
            "hierarchy cycle",
        ),
        (
            r#"[{"slug":"","group":"Learn"},{"slug":"topic","group":"API","parent":""}]"#,
            "share group",
        ),
        (
            r#"[{"slug":"","group":"Learn"},{"slug":"../escape","group":"Learn"}]"#,
            "invalid guide slug",
        ),
        (
            r#"[{"slug":"","group":"Learn"},{"slug":"index","group":"Learn"}]"#,
            "empty slug",
        ),
    ] {
        std::fs::write(config.root.join("navigation.json"), navigation).unwrap();
        let error = crate::compile(config, highlighter)
            .err()
            .expect("invalid navigation");
        assert!(error.to_string().contains(expected), "{error}");
    }
}
