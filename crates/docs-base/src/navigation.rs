use crate::Site;
use fusor::{FromInputs, Memo, Signal, signal};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    Guide(usize),
    Custom,
    Missing,
}

/// Share one navigation state with the shell and the application's route outlet.
#[derive(Clone)]
pub struct Navigation {
    pub site: &'static Site,
    pub active: Signal<Location>,
    pub query: Signal<String>,
    pub menu: Signal<bool>,
}

impl Navigation {
    pub fn new(site: &'static Site) -> Self {
        Self {
            site,
            active: signal(Location::Missing),
            query: signal(String::new()),
            menu: signal(false),
        }
    }

    /// Clear search and close the mobile sidebar when a navigation link is used.
    pub fn open(&self) {
        self.query.set(String::new());
        self.menu.set(false);
    }

    pub fn is_reference(&self) -> bool {
        let Location::Guide(index) = self.active.get() else {
            return false;
        };
        self.site
            .pages
            .get(index)
            .is_some_and(|page| page.reference || page.group == "Reference")
    }

    pub(crate) fn groups(&self) -> Vec<&'static str> {
        let mut groups = Vec::new();
        for page in self.site.pages {
            if !groups.contains(&page.group) {
                groups.push(page.group);
            }
        }
        groups
    }
}

#[derive(FromInputs)]
pub(crate) struct NavGroup {
    #[input]
    item: Memo<&'static str>,
    #[input]
    navigation: Navigation,
}

impl NavGroup {
    fn entries(&self) -> Vec<usize> {
        self.navigation.site.children(None, self.item.get())
    }

    fn visible(&self) -> bool {
        self.navigation
            .site
            .pages
            .iter()
            .enumerate()
            .any(|(index, page)| {
                page.group == self.item.get()
                    && self
                        .navigation
                        .site
                        .matches(index, &self.navigation.query.get())
            })
    }
}

#[derive(FromInputs)]
struct NavEntry {
    #[input]
    item: Memo<usize>,
    #[input]
    navigation: Navigation,
    // A manual choice applies until navigation changes the active page.
    #[local(init = signal(None))]
    expanded: Signal<Option<(Location, bool)>>,
}

impl NavEntry {
    fn title(&self) -> &'static str {
        let page = &self.navigation.site.pages[self.item.get()];
        if page.slug.is_empty() {
            "Introduction"
        } else {
            page.title
        }
    }

    fn children(&self) -> Vec<usize> {
        let page = &self.navigation.site.pages[self.item.get()];
        self.navigation.site.children(Some(page.slug), page.group)
    }

    fn visible(&self) -> bool {
        let site = self.navigation.site;
        let query = self.navigation.query.get();
        site.matches(self.item.get(), &query)
            || site.pages.iter().enumerate().any(|(index, _)| {
                site.ancestors(index).contains(&self.item.get()) && site.matches(index, &query)
            })
    }

    fn is_open(&self) -> bool {
        if !self.navigation.query.get().trim().is_empty() {
            return true;
        }
        let active = self.navigation.active.get();
        if let Some((page, open)) = self.expanded.get() {
            if page == active {
                return open;
            }
        }
        let Location::Guide(index) = active else {
            return false;
        };
        index == self.item.get()
            || self
                .navigation
                .site
                .ancestors(index)
                .contains(&self.item.get())
    }

    fn toggle(&self) {
        self.expanded
            .set(Some((self.navigation.active.get(), !self.is_open())));
    }
}

fusor::template!("web/navigation.html");
