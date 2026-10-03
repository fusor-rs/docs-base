use crate::{Link, Page, Section, Site, markdown::markdown};
use fusor::{
    FromInputs, Memo, OwnerHandle,
    dom::{Content, JsValue},
};

static MISSING: Page = Page {
    parent: None,
    reference: false,
    slug: "missing",
    title: "Page not found",
    group: "Documentation",
    lead: "<p>That page is not in this edition of the docs. \
        Choose a topic in the sidebar or return to the introduction.</p>",
    search: "",
    source: "",
    sections: &[],
};

/// Optional, application-owned content mounted with the article's lifetime.
#[derive(Clone, Default)]
pub struct ArticleExtras {
    pub before: Option<Content>,
    pub after: Option<Content>,
    pub footer: Option<Content>,
    pub aside: Option<Content>,
}

pub struct Article {
    site: &'static Site,
    page: &'static Page,
    index: Option<usize>,
    extras: ArticleExtras,
}

pub struct ArticleInputs {
    pub site: &'static Site,
    pub index: Option<usize>,
    pub extras: ArticleExtras,
}

impl fusor::dom::FromInputs for Article {
    type Inputs = ArticleInputs;
    type Error = JsValue;

    fn from_inputs(inputs: Self::Inputs, _: OwnerHandle) -> Result<Self, JsValue> {
        let index = inputs
            .index
            .filter(|index| *index < inputs.site.pages.len());
        let page = index
            .and_then(|index| inputs.site.pages.get(index))
            .unwrap_or(&MISSING);
        fusor::dom::document()?.set_title(&format!("{} · {}", page.title, inputs.site.name));
        Ok(Self {
            site: inputs.site,
            page,
            index,
            extras: inputs.extras,
        })
    }
}

impl Article {
    fn breadcrumbs(&self) -> Vec<Link> {
        let Some(index) = self.index else {
            return Vec::new();
        };
        self.site
            .ancestors(index)
            .into_iter()
            .map(|index| Link {
                href: self.site.pages[index].slug,
                label: self.site.pages[index].title,
            })
            .collect()
    }

    fn previous(&self) -> usize {
        self.index.unwrap_or(0).saturating_sub(1)
    }

    fn next(&self) -> usize {
        self.index
            .unwrap_or(0)
            .saturating_add(1)
            .min(self.site.pages.len() - 1)
    }
}

#[derive(FromInputs)]
struct Breadcrumb {
    #[input]
    item: Memo<Link>,
    #[input]
    base_path: &'static str,
}

#[derive(FromInputs)]
struct GuideSection {
    #[input]
    item: Memo<Section>,
}

#[derive(FromInputs)]
struct Toc {
    #[input]
    item: Memo<Section>,
}

#[derive(FromInputs)]
struct RelatedLink {
    #[input]
    item: Memo<Link>,
}

fusor::template!("web/article.html");
