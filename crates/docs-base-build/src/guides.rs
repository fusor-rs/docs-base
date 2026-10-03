use crate::{Error, Highlighter, Result, STYLESHEET, error, links::Links, markdown};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

/// Paths are relative to `root`, which also resolves `source=` code inclusions.
/// `base_path` starts and ends with `/`. `repository` is a source-tree URL ending
/// in `/`, used for relative links to files outside the published guides.
pub struct Config<'a> {
    pub root: &'a Path,
    pub content: &'a Path,
    pub navigation: &'a Path,
    pub references: Option<&'a Path>,
    pub base_path: &'a str,
    pub repository: Option<&'a str>,
}

pub struct CompiledGuides {
    pub source: TokenStream,
    sources: Vec<(PathBuf, String)>,
}

impl CompiledGuides {
    /// Publish the stylesheet and original Markdown into the app's public assets.
    /// Existing identical files are retained so watching builds do not loop.
    pub fn write_assets(&self, directory: &Path) -> Result<()> {
        println!(
            "cargo:rerun-if-changed={}",
            directory.join("docs-base.css").display()
        );
        error::write(&directory.join("docs-base.css"), STYLESHEET)?;
        for (path, markdown) in &self.sources {
            let output = directory.join("content").join(path);
            println!("cargo:rerun-if-changed={}", output.display());
            error::write(&output, markdown)?;
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Navigation {
    slug: String,
    group: String,
    parent: Option<String>,
    #[serde(default)]
    reference: bool,
}

struct Guide {
    navigation: Navigation,
    document: markdown::Document,
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    token: String,
    href: String,
    label: String,
}

/// Emit `PAGES: &[docs_base::Page]` and Cargo dependency notifications.
/// Rejects malformed Markdown, broken hierarchy, duplicate IDs and reference targets.
pub fn compile(config: &Config<'_>, highlighter: &Highlighter) -> Result<CompiledGuides> {
    validate_base_path(config.base_path)?;
    if config.repository.is_some_and(|url| {
        !(url.starts_with("https://") || url.starts_with("http://")) || !url.ends_with('/')
    }) {
        return Err("repository must be an HTTP(S) source-tree URL ending in /".into());
    }
    let root = std::fs::canonicalize(config.root).map_err(|source| Error::Io {
        path: config.root.to_owned(),
        source,
    })?;
    let config = Config {
        root: &root,
        ..*config
    };
    let guides = read_guides(&config, highlighter)?;
    let references = match config.references {
        Some(path) => read_json(&config.root.join(path))?,
        None => Vec::new(),
    };
    for reference in &references {
        validate_reference(&guides, reference, config.base_path)?;
    }
    let pages = guides
        .iter()
        .map(|guide| page_source(guide, &references, config.base_path));
    let source = quote! { pub static PAGES: &[::docs_base::Page] = &[#(#pages),*]; };
    let sources = guides
        .into_iter()
        .map(|guide| {
            (
                PathBuf::from(file_name(&guide.navigation.slug)),
                guide.source,
            )
        })
        .collect();
    Ok(CompiledGuides { source, sources })
}

fn read_guides(config: &Config<'_>, highlighter: &Highlighter) -> Result<Vec<Guide>> {
    let navigation: Vec<Navigation> = read_json(&config.root.join(config.navigation))?;
    validate_navigation(&navigation)?;
    let paths = navigation
        .iter()
        .map(|page| {
            (
                crate::links::normalize(
                    &config.root.join(config.content).join(file_name(&page.slug)),
                ),
                page.slug.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let documents = paths
        .iter()
        .map(|(path, _)| compile_document(path, &paths, config, highlighter))
        .collect::<Result<Vec<_>>>()?;
    Ok(navigation
        .into_iter()
        .zip(documents)
        .map(|(navigation, (document, source))| Guide {
            source,
            navigation,
            document,
        })
        .collect())
}

fn compile_document(
    path: &Path,
    paths: &[(PathBuf, &str)],
    config: &Config<'_>,
    highlighter: &Highlighter,
) -> Result<(markdown::Document, String)> {
    println!("cargo:rerun-if-changed={}", path.display());
    let source = error::read(path)?;
    let links = Links {
        config,
        current: path,
        pages: paths,
    };
    let document =
        markdown::parse(&source, config.root, highlighter, &links).map_err(|source| {
            Error::Document {
                path: path.to_owned(),
                source: Box::new(source),
            }
        })?;
    Ok((document, source))
}

fn validate_base_path(base_path: &str) -> Result<()> {
    if base_path != "/"
        && (!base_path.starts_with('/')
            || !base_path.ends_with('/')
            || base_path[1..base_path.len() - 1].split('/').any(|part| {
                part.is_empty()
                    || !part
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
            }))
    {
        return Err(
            "documentation base_path must be / or /path/ using letters, digits, hyphens or underscores"
                .into(),
        );
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    println!("cargo:rerun-if-changed={}", path.display());
    serde_json::from_str(&error::read(path)?).map_err(|source| Error::Json {
        path: path.to_owned(),
        source,
    })
}

fn file_name(slug: &str) -> String {
    format!("{}.md", if slug.is_empty() { "index" } else { slug })
}

fn validate_navigation(pages: &[Navigation]) -> Result<()> {
    if pages.first().is_none_or(|page| !page.slug.is_empty()) {
        return Err("navigation must start with the introduction (an empty slug)".into());
    }
    let mut slugs = BTreeMap::new();
    for page in pages {
        if page.slug == "index" {
            return Err("the index.md file belongs to the introduction; use an empty slug".into());
        }
        if !page.slug.is_empty()
            && page.slug.split('/').any(|part| {
                part.is_empty() || !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            })
        {
            return Err(format!("invalid guide slug: {}", page.slug).into());
        }
        if slugs.insert(page.slug.as_str(), page).is_some() {
            return Err(format!("duplicate page: {}", page.slug).into());
        }
    }
    for page in pages {
        let mut seen = BTreeSet::from([page.slug.as_str()]);
        let mut current = page;
        while let Some(parent) = &current.parent {
            if !seen.insert(parent) {
                return Err(format!("page hierarchy cycle: {parent}").into());
            }
            let next = slugs
                .get(parent.as_str())
                .ok_or_else(|| Error::Invalid(format!("unknown parent: {parent}")))?;
            if next.group != page.group {
                return Err(format!("parent and child must share group: {parent}").into());
            }
            current = next;
        }
    }
    Ok(())
}

fn validate_reference(guides: &[Guide], reference: &Reference, base_path: &str) -> Result<()> {
    let href = &reference.href;
    let (slug, id) = href
        .strip_prefix(base_path)
        .and_then(|path| path.split_once('#'))
        .ok_or_else(|| Error::Invalid(format!("reference must point to a docs section: {href}")))?;
    if !guides
        .iter()
        .any(|guide| guide.navigation.slug == slug && guide.document.anchors.contains(id))
    {
        return Err(format!("unknown reference target: {href}").into());
    }
    if reference.token.is_empty() {
        return Err(format!("reference token required: {href}").into());
    }
    Ok(())
}

fn page_source(guide: &Guide, references: &[Reference], base_path: &str) -> TokenStream {
    let Navigation {
        slug,
        group,
        parent,
        reference,
    } = &guide.navigation;
    let document = &guide.document;
    let (title, lead, search) = (&document.title, &document.lead, &document.search);
    let source = format!("{base_path}content/{}", file_name(slug));
    let parent = match parent {
        Some(parent) => quote! { Some(#parent) },
        None => quote! { None },
    };
    let references = if *reference { &[] } else { references };
    let sections = document
        .sections
        .iter()
        .map(|section| section_source(section, references));
    quote! {
        ::docs_base::Page {
            parent: #parent, reference: #reference, slug: #slug, title: #title,
            group: #group, lead: #lead, search: #search, source: #source,
            sections: &[#(#sections),*],
        }
    }
}

fn section_source(section: &markdown::Section, references: &[Reference]) -> TokenStream {
    let (id, title, html) = (&section.id, &section.title, &section.body.html);
    let mut seen = BTreeSet::new();
    let links = references
        .iter()
        .filter(|reference| {
            contains_token(&section.body.code, &reference.token) && seen.insert(&reference.href)
        })
        .map(|reference| {
            let (href, label) = (&reference.href, &reference.label);
            quote! { ::docs_base::Link { href: #href, label: #label } }
        });
    quote! { ::docs_base::Section { id: #id, title: #title, html: #html, references: &[#(#links),*] } }
}

fn contains_token(code: &str, token: &str) -> bool {
    code.match_indices(token).any(|(start, _)| {
        let continuation = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ':');
        !code[..start].chars().next_back().is_some_and(continuation)
            && (token.ends_with(':')
                || !code[start + token.len()..]
                    .chars()
                    .next()
                    .is_some_and(continuation))
    })
}
