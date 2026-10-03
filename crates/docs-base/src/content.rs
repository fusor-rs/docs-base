#[derive(Clone, Copy, PartialEq)]
pub struct Link {
    pub href: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Section {
    pub id: &'static str,
    pub title: &'static str,
    pub html: &'static str,
    pub references: &'static [Link],
}

/// A guide emitted by `docs-base-build`; its HTML is trusted build output.
pub struct Page {
    pub parent: Option<&'static str>,
    pub reference: bool,
    pub slug: &'static str,
    pub title: &'static str,
    pub group: &'static str,
    pub lead: &'static str,
    pub search: &'static str,
    pub source: &'static str,
    pub sections: &'static [Section],
}

/// Site settings shared by the shell and article components.
/// `base_path` matches the build configuration. Pages must be the nonempty,
/// validated list from `docs-base-build`, preserving its order and hierarchy.
pub struct Site {
    pub name: &'static str,
    pub base_path: &'static str,
    pub logo: &'static str,
    pub version: &'static str,
    pub pages: &'static [Page],
}

impl Site {
    pub(crate) fn href(&self, page: &Page) -> String {
        format!("{}{}", self.base_path, page.slug)
    }

    /// Ancestors are ordered from the topic root to the immediate parent.
    /// Panics if a parent is missing; the build compiler validates this invariant.
    pub(crate) fn ancestors(&self, index: usize) -> Vec<usize> {
        let mut ancestors = Vec::new();
        let mut current = self.pages.get(index).and_then(|page| page.parent);
        while let Some(slug) = current {
            let parent = self
                .pages
                .iter()
                .position(|page| page.slug == slug)
                .expect("the documentation compiler validates every parent");
            ancestors.push(parent);
            current = self.pages[parent].parent;
        }
        ancestors.reverse();
        ancestors
    }

    pub(crate) fn children(&self, parent: Option<&str>, group: &str) -> Vec<usize> {
        self.pages
            .iter()
            .enumerate()
            .filter(|(_, page)| page.parent == parent && page.group == group)
            .map(|(index, _)| index)
            .collect()
    }

    pub(crate) fn matches(&self, index: usize, query: &str) -> bool {
        let needle = query.trim().to_lowercase();
        let page = &self.pages[index];
        [page.title, page.group, page.search]
            .iter()
            .any(|text| text.to_lowercase().contains(&needle))
    }
}
