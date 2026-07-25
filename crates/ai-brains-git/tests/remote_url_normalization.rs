//! Normalization + hash stability for equivalent remote URL forms (R-GIT1).

#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_git::{hash_remote_url, normalize_remote_url};

#[test]
fn normalize_remote_url__ssh_https_scp_same_host_path__equal() {
    let https = "https://github.com/org/repo.git";
    let scp = "git@github.com:org/repo";
    let ssh = "ssh://git@github.com/org/repo";

    assert_eq!(normalize_remote_url(https), "github.com/org/repo");
    assert_eq!(normalize_remote_url(scp), "github.com/org/repo");
    assert_eq!(normalize_remote_url(ssh), "github.com/org/repo");

    let h = hash_remote_url(https).unwrap();
    assert_eq!(h, hash_remote_url(scp).unwrap());
    assert_eq!(h, hash_remote_url(ssh).unwrap());
}

#[test]
fn normalize_remote_url__with_without_git_suffix__equal() {
    let with = "https://github.com/org/repo.git";
    let without = "https://github.com/org/repo";
    let upper = "https://github.com/org/repo.GIT";

    assert_eq!(normalize_remote_url(with), normalize_remote_url(without));
    assert_eq!(normalize_remote_url(with), normalize_remote_url(upper));
    assert_eq!(
        hash_remote_url(with).unwrap(),
        hash_remote_url(without).unwrap()
    );
    assert_eq!(
        hash_remote_url(with).unwrap(),
        hash_remote_url(upper).unwrap()
    );
}

#[test]
fn normalize_remote_url__host_case_differences__equal() {
    let lower = "https://github.com/org/repo";
    let mixed = "https://GitHub.COM/org/repo";

    assert_eq!(normalize_remote_url(lower), normalize_remote_url(mixed));
    assert_eq!(
        hash_remote_url(lower).unwrap(),
        hash_remote_url(mixed).unwrap()
    );
}

#[test]
fn normalize_remote_url__trailing_slash__equal() {
    let plain = "https://github.com/org/repo";
    let slash = "https://github.com/org/repo/";
    let git_slash = "https://github.com/org/repo.git/";

    assert_eq!(normalize_remote_url(plain), normalize_remote_url(slash));
    assert_eq!(normalize_remote_url(plain), normalize_remote_url(git_slash));
    assert_eq!(
        hash_remote_url(plain).unwrap(),
        hash_remote_url(slash).unwrap()
    );
}

#[test]
fn normalize_remote_url__strips_credentials() {
    let with_user = "https://user:token@github.com/org/repo.git";
    let plain = "https://github.com/org/repo.git";

    assert_eq!(normalize_remote_url(with_user), "github.com/org/repo");
    assert_eq!(
        hash_remote_url(with_user).unwrap(),
        hash_remote_url(plain).unwrap()
    );
    assert!(!normalize_remote_url(with_user).contains("user"));
    assert!(!normalize_remote_url(with_user).contains("token"));
}

#[test]
fn normalize_remote_url__trims_whitespace() {
    let padded = "  https://github.com/org/repo.git  \n";
    assert_eq!(normalize_remote_url(padded), "github.com/org/repo");
}
