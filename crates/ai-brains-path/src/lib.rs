mod alias;
mod canonical;
mod cap_open;
mod discovery;
mod display;
mod errors;
mod location;
mod project_path;
mod reparse;
mod symlink;
mod unc;
mod windows;
mod wsl;

pub use canonical::normalize_project_path;
pub use cap_open::{
    CapOpenError, list_entry_names, open_ambient_vault_dir, open_dir_component_nofollow,
    open_dir_nofollow_components, open_file_component_nofollow, read_file_nofollow_components,
};
pub use discovery::{extract_project_id_from_ledgerful, find_ledgerful_dir};
pub use display::display_path;
pub use errors::{PathError, Result};
pub use location::{
    normalize_for_location_compare, path_is_same_or_inside, paths_refer_to_same_location,
};
pub use project_path::ProjectPath;
pub use reparse::{is_reparse_or_symlink, refuse_if_reparse};
pub use symlink::resolve_best_effort;

#[deprecated(note = "use extract_project_id_from_ledgerful")]
#[allow(deprecated)]
pub use discovery::extract_project_id_from_changeguard;
#[deprecated(note = "use find_ledgerful_dir")]
#[allow(deprecated)]
pub use discovery::find_changeguard_dir;
