//! Highlight code at build time for guides and live-example source viewers.
use crate::Result;
use proc_macro2::TokenStream;
use quote::quote;
use std::fmt::Write;
use syntect::{
    easy::HighlightLines,
    highlighting::{Color, Style, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

const LIGHT_BACKGROUND: [u8; 3] = [247, 249, 248];
const DARK_BACKGROUND: [u8; 3] = [21, 34, 27];
const MINIMUM_CONTRAST: f64 = 4.5;

/// Reuse one highlighter for all guides and source viewers in a build.
pub struct Highlighter {
    syntaxes: SyntaxSet,
    themes: ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            syntaxes: two_face::syntax::extra_newlines(),
            themes: ThemeSet::load_defaults(),
        }
    }
}

impl Highlighter {
    fn syntax(&self, label: &str) -> &syntect::parsing::SyntaxReference {
        let label = label.to_lowercase();
        let extension = if label.contains(".html") || label.contains("html") {
            "html"
        } else if label.contains(".rs") || label.starts_with("rust") {
            "rs"
        } else if label.contains("toml") {
            "toml"
        } else if label.contains(".js") || label.starts_with("javascript") {
            "js"
        } else if label.contains(".ts") || label.starts_with("typescript") {
            "ts"
        } else if label.contains("json") {
            "json"
        } else if label.starts_with("terminal") {
            "sh"
        } else {
            &label
        };
        self.syntaxes
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text())
    }

    fn highlight(&self, code: &str, label: &str) -> Result<Vec<(String, String)>> {
        let syntax = self.syntax(label);
        let mut light = HighlightLines::new(syntax, &self.themes.themes["InspiredGitHub"]);
        let mut dark = HighlightLines::new(syntax, &self.themes.themes["base16-ocean.dark"]);
        let mut tokens: Vec<(String, String)> = Vec::new();
        for line in LinesWithEndings::from(code) {
            let light = light.highlight_line(line, &self.syntaxes)?;
            let dark = dark.highlight_line(line, &self.syntaxes)?;
            merge_themes(&light, &dark, &mut tokens);
        }
        Ok(tokens)
    }

    /// Emit highlighted spans for an application-owned `CodeToken` record.
    pub fn tokens(&self, code: &str, label: &str) -> Result<TokenStream> {
        let tokens = self.highlight(code, label)?;
        let tokens = tokens.iter().enumerate().map(|(id, (text, style))| {
            quote! { CodeToken { id: #id, text: #text, style: #style } }
        });
        Ok(quote! { &[#(#tokens),*] })
    }

    pub(crate) fn html(&self, code: &str, language: &str) -> Result<String> {
        let mut html = String::new();
        for (text, style) in self.highlight(code, language)? {
            write!(
                html,
                "<span class=\"syntax-token\" style=\"{style}\">{}</span>",
                super::html::escape(&text)
            )?;
        }
        Ok(html)
    }
}

fn merge_themes(
    light: &[(Style, &str)],
    dark: &[(Style, &str)],
    tokens: &mut Vec<(String, String)>,
) {
    let (mut light_index, mut dark_index) = (0, 0);
    let (mut light_offset, mut dark_offset) = (0, 0);
    // Themes can coalesce adjacent spans differently; intersect their ranges.
    while light_index < light.len() && dark_index < dark.len() {
        let (light_style, light_text) = light[light_index];
        let (dark_style, dark_text) = dark[dark_index];
        let length = (light_text.len() - light_offset).min(dark_text.len() - dark_offset);
        let text = &light_text[light_offset..light_offset + length];
        let style = format!(
            "--syntax-light:{};--syntax-dark:{}",
            accessible_color(light_style.foreground, LIGHT_BACKGROUND),
            accessible_color(dark_style.foreground, DARK_BACKGROUND),
        );
        if let Some((previous, _)) = tokens.last_mut().filter(|(_, previous)| *previous == style) {
            previous.push_str(text);
        } else {
            tokens.push((text.to_owned(), style));
        }
        light_offset += length;
        dark_offset += length;
        if light_offset == light_text.len() {
            light_index += 1;
            light_offset = 0;
        }
        if dark_offset == dark_text.len() {
            dark_index += 1;
            dark_offset = 0;
        }
    }
}

// Keep comments and punctuation readable against the actual docs backgrounds.
fn accessible_color(color: Color, background: [u8; 3]) -> String {
    fn luminance(rgb: [u8; 3]) -> f64 {
        rgb.into_iter()
            .zip([0.2126, 0.7152, 0.0722])
            .map(|(channel, weight)| {
                let value = f64::from(channel) / 255.0;
                weight
                    * if value <= 0.04045 {
                        value / 12.92
                    } else {
                        ((value + 0.055) / 1.055).powf(2.4)
                    }
            })
            .sum()
    }
    let mut rgb = [color.r, color.g, color.b];
    let background_luminance = luminance(background);
    loop {
        let foreground = luminance(rgb);
        if (foreground.max(background_luminance) + 0.05)
            / (foreground.min(background_luminance) + 0.05)
            >= MINIMUM_CONTRAST
        {
            break;
        }
        for channel in &mut rgb {
            *channel = if background_luminance > 0.5 {
                channel.saturating_sub(4)
            } else {
                channel.saturating_add(4)
            };
        }
    }
    format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}
