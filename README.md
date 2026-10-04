# docs-base

Shared Markdown documentation components for [Fusor](https://fusor.build/) apps.
Fusor and Hypercmd keep their guides, branding, routes and interactive examples
in their own repositories. This repository owns the Markdown compiler, article
layout, navigation, search and theme controls.

Requires Rust 1.85 or later and Fusor `>=0.1.4, <0.2.0`. These internal packages
are consumed from Git and set `publish = false`. Pin both packages to the same commit
in a consuming application's manifest and commit its Cargo lockfile.
Compatible Fusor patch releases do not require a docs-base revision change.

| Package | Responsibility |
| --- | --- |
| `docs-base-build` | Markdown parsing, highlighting, validation, page records and static sources |
| `docs-base` | Shell, hierarchical navigation, article, table of contents, search and theme |

Markdown is compiled during the Cargo build. The browser receives rendered HTML
inside the Wasm application, without a Markdown parser or syntax highlighter at
runtime. This is a client-rendered site requiring JavaScript/Wasm and a host
configured to serve `index.html` for guide routes. It does not emit a separate
HTML document per route. Original Markdown and CSS are ordinary static assets.

## Run the example

Install Rust, Node 22 and `just`, then run:

```sh
just setup-browser
just book
just preview
```

Open <http://127.0.0.1:4173/manual/>. `just dev` serves the example with live
reload. [`examples/book`](examples/book) is a complete consumer, including the
build script, route adapter and HTML shell. It uses a configurable `/manual/`
base path. `FUSOR_BIN` can select a local Fusor CLI.

## Use it in an application

Add `docs-base` as a dependency and `docs-base-build` as a build dependency from
`https://github.com/fusor-rs/docs-base`, using the same `rev` for both. The app
also needs Fusor, `fusor-components`, `fusor-router` and `fusor-build`; the
[example manifest](examples/book/Cargo.toml) shows compatible versions and
features. Fusor checkouts must patch `fusor-core`, `fusor-components` and
`fusor-build` to the same checkout to keep compiler and runtime contracts aligned.
After bumping the checkout version, update those packages in the consuming
lockfile so Cargo selects the local patches instead of retaining the previous
registry versions.

The [build script](examples/book/build.rs) calls `docs_base_build::compile`
with content paths and a base path, writes the generated Rust to `OUT_DIR`,
and calls `write_assets` before `fusor_build::compile_app()`. Include the
generated Rust to get `PAGES: &[docs_base::Page]`.

Create a `Site` with the product name, logo, version, base path and pages.
Share one `Navigation` between `Shell` and the route outlet. The
[example app](examples/book/src/app.rs) maps the router's path to a page index
and updates `Navigation.active`. `Article` displays that index, or a missing
page for `None` or an out-of-bounds index. Use compiler-generated, nonempty pages.

Link `docs-base.css` before your brand stylesheet. `Shell` accepts header and
sidebar content; its children own routing. `ArticleExtras` accepts content
before and after sections, in the footer and below the table of contents.
Content follows the component's lifetime, so demos can own signals,
subscriptions and cleanup. Landing pages remain independent apps.

Set Fusor's `assets = "public"`, the matching `base-path`, and
`history-fallback = ["/"]`. Ignore generated `public/content/` and
`public/docs-base.css` when source Markdown lives elsewhere. Identical asset
files are retained; deleting generated assets triggers their recreation.

## Write guides

Every guide starts with one `# Title`, followed by introductory prose and
`## Sections`. The introduction is `index.md`; other slugs map to Markdown
paths such as `topics/authoring.md`. Section headings supply the table of
contents and may specify stable IDs: `## Install {#install}`. Other headings
receive unique IDs derived from their text.

Navigation metadata sets groups and reading order:

```json
[
  { "slug": "", "group": "Learn" },
  { "slug": "topics", "group": "Learn" },
  { "slug": "topics/authoring", "group": "Learn", "parent": "topics" }
]
```

The first page has an empty slug. Parents must exist in the same group and
form an acyclic hierarchy. Keep children after their parent for previous/next
reading order. `reference: true` marks lookup pages that omit contextual links.

Markdown links to registered files, such as `topics/authoring.md#code`, become
application routes. Set `repository` to a source-tree URL ending in `/` to
resolve other relative links such as `../README.md`; those targets must exist
inside `root`. Absolute paths, fragments and HTTP(S)/mailto links are preserved.
Image URLs are preserved; use public asset URLs for images.

Tables, task lists, quotes, strikethrough and fenced code are supported. A fence's
first word is its language. `title=` supplies a caption; an empty fence can
include a tracked source file relative to `root`:

````markdown
```rust source=src/app.rs title=Application state
```
````

Missing sources, nonempty inclusion fences and duplicate explicit heading IDs
fail the build. Raw HTML is displayed as text except standalone `<details>`,
`</details>` and plain-text `<summary>Label</summary>` lines. Put blank lines
around Markdown inside a disclosure. Use components only with build-generated HTML.

An optional references JSON file contains `{ "token", "href", "label" }`
records. Matching code tokens add contextual API links below sections. Targets
must point to existing guide anchors under `base_path`. Tokens ending in `:`
match directive families; others respect identifier boundaries.

## Branding and checks

Override CSS variables on `.site` and `.site.dark`: `--bg`, `--panel`, `--text`,
`--muted`, `--line`, `--accent` and `--wash`. Keep the default `--code` backgrounds
when using the provided highlighting colors, whose contrast is calculated for
those backgrounds. Product names namespace the saved theme in local storage.

Run `just check`, `just msrv`, `just browser` and `just packages` before merging.
Native checks cover formatting, tests, Clippy and rustdoc with warnings denied.
The browser gate executes the real example. Package checks build both archives,
including component templates and CSS. Native checks need neither Node nor browsers.

Use ordinary Rust and HTML, preserve Rust 1.85 compatibility, and keep product
behavior in consuming apps. Shared API changes need a design issue with a
concrete consumer; the extraction is described in
[issue #1](https://github.com/fusor-rs/docs-base/issues/1).
