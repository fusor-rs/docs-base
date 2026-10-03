use docs_base::{Article, ArticleExtras, Location, Navigation, Shell, Site};
use fusor::{
    OwnerHandle,
    dom::{Content, FromInputs, JsValue},
};
use fusor_router::browser::declarative::Navigation as RouterNavigation;

include!(concat!(env!("OUT_DIR"), "/guides.rs"));

static SITE: Site = Site {
    name: "Example book",
    base_path: "/manual/",
    logo: "/manual/favicon.svg",
    version: "v0.1 dev",
    pages: PAGES,
};

pub struct App {
    navigation: Navigation,
    header: Content,
}

impl App {
    fn new() -> Self {
        Self {
            navigation: Navigation::new(&SITE),
            header: Content::new(|_| Header),
        }
    }
}

struct Header;

struct DocRoute {
    index: Option<usize>,
}

pub struct DocRouteInputs {
    pub navigation: Navigation,
}

impl FromInputs for DocRoute {
    type Inputs = DocRouteInputs;
    type Error = JsValue;

    fn from_inputs(inputs: Self::Inputs, owner: OwnerHandle) -> Result<Self, JsValue> {
        let router = RouterNavigation::from_owner(&owner)
            .ok_or_else(|| JsValue::from_str("missing documentation navigation"))?;
        let index = match router.location().get().segments() {
            Ok(segments) => {
                let slug = segments.join("/");
                PAGES.iter().position(|page| page.slug == slug)
            }
            // Malformed route encodings use the same missing-page view as unknown paths.
            Err(_) => None,
        };
        inputs
            .navigation
            .active
            .set(index.map_or(Location::Missing, Location::Guide));
        Ok(Self { index })
    }
}

fusor::bindings!(app);
