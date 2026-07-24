mod alias;
mod canonical;
mod discovery;
mod display;
mod errors;
mod location;
mod project_path;
mod symlink;
mod unc;
mod windows;
mod wsl;

pub use canonical::normalize_project_path;
pub use discovery::{extract_project_id_from_ledgerful, find_ledgerful_dir};
pub use display::display_path;
pub use errors::{PathError, Result};
pub use location::{
    normalize_for_location_compare, path_is_same_or_inside, paths_refer_to_same_location,
};
pub use project_path::ProjectPath;
pub use symlink::resolve_best_effort;

#[deprecated(note = "use extract_project_id_from_ledgerful")]
#[allow(deprecated)]
pub use discovery::extract_project_id_from_changeguard;
#[deprecated(note = "use find_ledgerful_dir")]
#[allow(deprecated)]
pub use discovery::find_changeguard_dir;
