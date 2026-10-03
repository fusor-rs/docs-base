use crate::{Navigation, navigation::NavGroup};
use fusor::{
    OwnerHandle, Signal,
    dom::{Content, FromInputs, JsValue, document},
    signal,
};
use wasm_bindgen::JsCast;

/// Documentation chrome. Caller-supplied children own routing and page lifetimes.
pub struct Shell {
    navigation: Navigation,
    header: Option<Content>,
    sidebar: Option<Content>,
    dark: Signal<bool>,
    theme_key: String,
    storage: Option<web_sys::Storage>,
}

pub struct ShellInputs {
    pub navigation: Navigation,
    pub header: Option<Content>,
    pub sidebar: Option<Content>,
}

impl FromInputs for Shell {
    type Inputs = ShellInputs;
    type Error = JsValue;

    fn from_inputs(inputs: Self::Inputs, _: OwnerHandle) -> Result<Self, JsValue> {
        let theme_key = format!("{}-docs-theme", inputs.navigation.site.name);
        let storage = match web_sys::window()
            .map(|window| window.local_storage())
            .transpose()
        {
            Ok(storage) => storage.flatten(),
            Err(error) => {
                web_sys::console::warn_1(&error);
                None
            }
        };
        let dark = match storage
            .as_ref()
            .map(|storage| storage.get_item(&theme_key))
            .transpose()
        {
            Ok(value) => value.flatten().is_some_and(|value| value == "dark"),
            Err(error) => {
                web_sys::console::warn_1(&error);
                false
            }
        };
        Ok(Self {
            navigation: inputs.navigation,
            header: inputs.header,
            sidebar: inputs.sidebar,
            dark: signal(dark),
            theme_key,
            storage,
        })
    }
}

impl Shell {
    fn theme(&self) {
        self.dark.update(|dark| *dark = !*dark);
        let Some(storage) = &self.storage else { return };
        let value = if self.dark.get() { "dark" } else { "light" };
        if let Err(error) = storage.set_item(&self.theme_key, value) {
            web_sys::console::warn_1(&error);
        }
    }

    fn keyboard(&self, event: web_sys::Event) {
        let Some(event) = event.dyn_ref::<web_sys::KeyboardEvent>() else {
            return;
        };
        if (event.meta_key() || event.ctrl_key()) && event.key() == "k" {
            event.prevent_default();
            self.navigation.menu.set(true);
            if let Err(error) = focus_search() {
                web_sys::console::warn_1(&error);
            }
        }
        if event.key() == "Escape" {
            self.navigation.open();
        }
    }

    fn has_matches(&self) -> bool {
        self.navigation
            .site
            .pages
            .iter()
            .enumerate()
            .any(|(index, _)| {
                self.navigation
                    .site
                    .matches(index, &self.navigation.query.get())
            })
    }
}

fn focus_search() -> Result<(), JsValue> {
    document()?
        .get_element_by_id("search")
        .ok_or_else(|| JsValue::from_str("documentation search input is missing"))?
        .dyn_into::<web_sys::HtmlElement>()?
        .focus()
}

fusor::template!("web/shell.html");
