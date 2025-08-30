use std::env::*;
use std::error;
use std::fs::*;
use std::io;
use std::path::*;
use std::result;
use walkdir::*;

fn main() -> result::Result<(), Box<dyn error::Error>> {
    copy_assets_directory_to_build_output()?;
    Ok(())
}

fn copy_assets_directory_to_build_output() -> io::Result<()> {
    println!("cargo::rerun-if-changed=assets");
    #[expect(clippy::unwrap_used, reason = "These envvars are always provided by Cargo in a build script.")]
    let assets_dest_path = Path::new(&var("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join(var("PROFILE").unwrap())
        .join("assets");
    // Makes sure we always an empty assets directory to begin with at OUT_DIR.
    #[expect(unused_must_use, reason = "It's OK if assets_dest_path doesn't exist for it to be removed, because all we want is an empty assets directory. It's also unlikely that some files in assets_dest_path will be unremovable.")]
    remove_dir_all(&assets_dest_path);
    create_dir_all(&assets_dest_path)?;
    if !matches!(exists("assets"), Ok(true)) {
        return Ok(());
    }
    copy_dir_recursively_as_hard_links("assets", assets_dest_path)?;
    Ok(())
}

/// Hard links the contents of the `from` directory to the `to` directory
/// recursively, ignoring symlinks. Both directories must already exist.
fn copy_dir_recursively_as_hard_links(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    for entry_result in WalkDir::new(&from) {
        let entry = entry_result?;
        let src_path = entry.path();
        #[expect(clippy::unwrap_used, reason = "unwrap should never fail here. If it does then it's a bug and can't be handled anyway.")]
        let dest_path = to.as_ref().join(src_path.strip_prefix(&from).unwrap());
        if entry.file_type().is_dir() {
            create_dir_all(&dest_path)?;
        }
        #[expect(clippy::filetype_is_file, reason = "We are not trying to read the file, so this lint is not applicable here.")]
        if entry.file_type().is_file() {
            hard_link(src_path, dest_path)?;
        }
    }
    Ok(())
}
