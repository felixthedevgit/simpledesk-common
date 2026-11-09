//! SimpleDesk: values built into the program instead of RustDesk's own infrastructure.
//! Each one can be replaced at build time with an environment variable of the same name,
//! for example `SIMPLEDESK_SERVER=desk.example.org cargo build`.

macro_rules! build_env_or {
    ($name:literal, $default:literal) => {
        match option_env!($name) {
            Some(value) => value,
            None => $default,
        }
    };
}

/// ID server and relay, as a host name (the default ports are used).
pub const SERVER: &str = build_env_or!("SIMPLEDESK_SERVER", "remote.felixthedev.com");

/// Public key of the server.
pub const KEY: &str = build_env_or!(
    "SIMPLEDESK_KEY",
    "kyaC2jNyp5a9Sz2q0D5VG0Ii3ca4xhChn0g5O2aOzyI="
);

/// API server for login, address book and the web panel.
pub const API_SERVER: &str =
    build_env_or!("SIMPLEDESK_API_SERVER", "https://remote.felixthedev.com");

/// GitHub repository with the releases for the update check, as "owner/repo".
pub const UPDATE_REPO: &str = build_env_or!("SIMPLEDESK_UPDATE_REPO", "felixthedevgit/simpledesk");

/// Project page, used where RustDesk linked to rustdesk.com.
pub const HOME_URL: &str = build_env_or!(
    "SIMPLEDESK_HOME_URL",
    "https://github.com/felixthedevgit/simpledesk"
);

/// The latest release of [`UPDATE_REPO`] in the GitHub API.
pub fn latest_release_api_url() -> String {
    format!("https://api.github.com/repos/{}/releases/latest", UPDATE_REPO)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_points_to_rustdesk() {
        for value in [SERVER, KEY, API_SERVER, UPDATE_REPO, HOME_URL] {
            assert!(!value.to_lowercase().contains("rustdesk.com"), "{}", value);
        }
        assert!(latest_release_api_url().starts_with("https://api.github.com/repos/"));
        assert_eq!(UPDATE_REPO.split('/').count(), 2);
    }
}
