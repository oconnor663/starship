use nix::sys::stat::Mode;
use nix::unistd::{Gid, Uid};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

/// Checks if the current user can write to the `folder_path`.
///
/// It extracts Unix access rights from the directory and checks whether
/// 1) the current user is the owner of the directory and whether it has the write access
/// 2) the current user's primary group is the directory group owner whether if it has write access
/// 2a) (not implemented on macOS) one of the supplementary groups of the current user is the
/// directory group owner and whether it has write access
/// 3) 'others' part of the access mask has the write access
pub fn is_write_allowed(folder_path: &str) -> Result<bool, &'static str> {
    let meta = fs::metadata(folder_path).map_err(|_| "Unable to stat() directory")?;
    let perms = meta.permissions().mode();

    let euid = Uid::effective();
    if euid.is_root() {
        return Ok(true);
    }
    if meta.uid() == euid.as_raw() {
        Ok(perms & Mode::S_IWUSR.bits() as u32 != 0)
    } else if (meta.gid() == Gid::effective().as_raw())
        || (get_supplementary_groups().contains(&meta.gid()))
    {
        Ok(perms & Mode::S_IWGRP.bits() as u32 != 0)
    } else {
        Ok(perms & Mode::S_IWOTH.bits() as u32 != 0)
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn get_supplementary_groups() -> Vec<u32> {
    match nix::unistd::getgroups() {
        Err(_) => Vec::new(),
        Ok(v) => v.into_iter().map(|i| i.as_raw()).collect(),
    }
}

#[cfg(all(unix, target_os = "macos"))]
fn get_supplementary_groups() -> Vec<u32> {
    // at the moment nix crate does not provide it for macOS
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn read_only_test() {
        assert_eq!(is_write_allowed("/etc"), Ok(false));
        assert_eq!(
            is_write_allowed("/i_dont_exist"),
            Err("Unable to stat() directory")
        );
    }
}
