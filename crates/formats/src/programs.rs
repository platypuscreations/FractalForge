// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractal types whose definition comes from an entry in a text file:
//! formulas (`.frm`), IFS (`.ifs`) and, later, L-systems (`.l`). This
//! hides their differences from the file-handling code.

use std::path::{Path, PathBuf};

use fractalforge_core::{FractalType, Program, ifs, types};
use fractalforge_formula::read_text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Formula,
    Ifs,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::Formula, Kind::Ifs];

    /// The kind a fractal type needs, if it needs one.
    pub fn of_type(t: &FractalType) -> Option<Kind> {
        Self::ALL.into_iter().find(|k| k.fractal() == t)
    }

    /// The kind of a file, from its extension (formulas by default).
    pub fn of_path(path: &Path) -> Kind {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        Self::ALL
            .into_iter()
            .find(|k| ext.eq_ignore_ascii_case(k.extension()))
            .unwrap_or(Kind::Formula)
    }

    pub fn fractal(self) -> &'static FractalType {
        match self {
            Kind::Formula => &types::FORMULA,
            Kind::Ifs => &types::IFS,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Kind::Formula => "frm",
            Kind::Ifs => "ifs",
        }
    }

    /// What one entry is called in messages.
    pub fn noun(self) -> &'static str {
        match self {
            Kind::Formula => "formula",
            Kind::Ifs => "IFS",
        }
    }

    /// The names of a file's entries, in order.
    pub fn entry_names(self, text: &str) -> Vec<String> {
        match self {
            Kind::Formula => fractalforge_formula::parser::entries(text)
                .into_iter()
                .map(|e| e.name)
                .collect(),
            Kind::Ifs => ifs::entries(text).into_iter().map(|e| e.name).collect(),
        }
    }

    /// Loads entry `name` from a file's text, with any warnings.
    pub fn load(self, text: &str, name: &str) -> Result<(Program, Vec<String>), String> {
        match self {
            Kind::Formula => fractalforge_formula::load(text, name),
            Kind::Ifs => ifs::load(text, name).map(|p| (p, Vec::new())),
        }
    }

    /// Searches a folder's files of this kind for entry `name` (in name
    /// order). Returns the file and its text.
    pub fn find_in_dir(self, dir: &Path, name: &str) -> Option<(PathBuf, String)> {
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .ok()?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| Kind::of_path(p) == self && p.extension().is_some())
            .collect();
        files.sort();
        files.into_iter().find_map(|p| {
            let text = read_text(&p).ok()?;
            self.entry_names(&text)
                .iter()
                .any(|n| n.eq_ignore_ascii_case(name))
                .then_some((p, text))
        })
    }
}

/// Loads entry `name` (or the file's first entry) from a `.frm` or `.ifs`
/// file.
pub fn load_file(path: &Path, name: Option<&str>) -> Result<(Program, Vec<String>), String> {
    let kind = Kind::of_path(path);
    let text = read_text(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let name = match name {
        Some(n) => n.to_owned(),
        None => kind
            .entry_names(&text)
            .into_iter()
            .next()
            .ok_or_else(|| format!("{}: no {} entries", path.display(), kind.noun()))?,
    };
    kind.load(&text, &name)
        .map_err(|e| format!("{}: {e}", path.display()))
}
