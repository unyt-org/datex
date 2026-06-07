use std::{env, path::PathBuf, str::FromStr};

use pathdiff::diff_paths;
use proc_macro2::Span;

/// Gets the absolute file path of the source file where the macro is invoked.
pub fn get_project_relative_file_path() -> PathBuf {
    let root_path = PathBuf::from_str(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()),
    )
    .unwrap();
    let call_site = PathBuf::from(Span::call_site().file());

    if (call_site.is_relative()) {
        return call_site;
    }
    call_site.strip_prefix(&root_path).unwrap_or_else(|x| {
        panic!(
            "Failed to get project relative file path. Call site: {:?}, Root path: {:?}",
            call_site, root_path
        )
    }).to_path_buf()
}
