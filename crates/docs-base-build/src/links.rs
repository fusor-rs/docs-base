use crate::{Config, Error, Result};
use std::path::{Component, Path, PathBuf};

pub(crate) struct Links<'a> {
    pub(crate) config: &'a Config<'a>,
    pub(crate) current: &'a Path,
    pub(crate) pages: &'a [(PathBuf, &'a str)],
}

impl Links<'_> {
    pub(crate) fn resolve(&self, url: &str) -> Result<String> {
        if url.starts_with(['/', '#', '?']) || url.contains(':') {
            return Ok(url.to_owned());
        }
        let (path, suffix) = url
            .find(['#', '?'])
            .map_or((url, ""), |index| url.split_at(index));
        let parent = self
            .current
            .parent()
            .expect("a guide has a containing directory");
        let target = normalize(&parent.join(path));
        if let Some((_, slug)) = self.pages.iter().find(|(file, _)| *file == target) {
            return Ok(format!("{}{slug}{suffix}", self.config.base_path));
        }
        if let Some(repository) = self.config.repository {
            let relative = target
                .strip_prefix(self.config.root)
                .map_err(|_| Error::Invalid(format!("link escapes repository: {url}")))?;
            std::fs::metadata(&target).map_err(|source| Error::Io {
                path: target.clone(),
                source,
            })?;
            return Ok(format!(
                "{repository}{}{suffix}",
                relative.to_string_lossy().replace('\\', "/")
            ));
        }
        Ok(url.to_owned())
    }

    pub(crate) fn attributes(&self, url: &str) -> &'static str {
        if url.ends_with(".md") || url.starts_with(&format!("{}source/", self.config.base_path)) {
            " target=\"_blank\" rel=\"noopener\""
        } else if url.starts_with(self.config.base_path) {
            " data-fusor-link"
        } else {
            ""
        }
    }
}

pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component),
        }
    }
    normalized
}
