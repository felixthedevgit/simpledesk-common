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

/// Options the settings page no longer offers, with the value they always have. They are not
/// saved, and neither the UI, a config import nor a custom client config can change them.
pub const FIXED_OPTIONS: &[(&str, &str)] = &[
    // No direct IP access and no LAN discovery, only the server.
    ("direct-server", "N"),
    ("enable-lan-discovery", "N"),
    // Whoever connects cannot change the settings of this device.
    ("allow-remote-config-modification", "N"),
    // The one-time password and, once set, the permanent password; accepting by click works too.
    ("verification-method", "use-both-passwords"),
    ("approve-mode", ""),
];

/// The fixed value of `key`, if it is one of [`FIXED_OPTIONS`].
pub fn fixed_option(key: &str) -> Option<&'static str> {
    FIXED_OPTIONS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// The shortest permanent password the client takes, in characters.
pub const MIN_PERMANENT_PASSWORD_LEN: usize = 8;

/// Whether `password` may become the permanent password. An empty one removes it.
pub fn is_permanent_password_allowed(password: &str) -> bool {
    password.is_empty() || password.chars().count() >= MIN_PERMANENT_PASSWORD_LEN
}

/// Servers the client may talk to, comma separated. A domain also allows its subdomains, an IP
/// address only itself. An empty list (`SIMPLEDESK_ALLOWED_SERVERS=`) allows every server.
pub const ALLOWED_SERVERS: &str = build_env_or!(
    "SIMPLEDESK_ALLOWED_SERVERS",
    "klg-orga.de,felixthedev.com"
);

/// Options that name a server. A value that is not allowed is not saved and reads as not set.
pub const SERVER_OPTIONS: &[&str] = &["custom-rendezvous-server", "relay-server", "api-server"];

/// Whether the client may talk to the server at `address` (host or IP address, with or without
/// port, or a URL), according to [`ALLOWED_SERVERS`].
pub fn is_server_allowed(address: &str) -> bool {
    // hbb_common's own tests talk to 127.0.0.1 and example hosts.
    let list = if cfg!(test) { "" } else { ALLOWED_SERVERS };
    is_allowed_by(address, list)
}

/// Like [`is_server_allowed`], with an error that names the server.
pub fn check_server_allowed(address: &str) -> crate::ResultType<()> {
    if !is_server_allowed(address) {
        crate::bail!("Server {} is not allowed in SimpleDesk", address);
    }
    Ok(())
}

/// Whether `value` may be stored in or read from the option `key`: true unless `key` is one
/// of [`SERVER_OPTIONS`] and `value` names a server that is not allowed.
pub fn is_server_option_allowed(key: &str, value: &str) -> bool {
    value.is_empty() || !SERVER_OPTIONS.contains(&key) || is_server_allowed(value)
}

/// `value` of the option `key`, or nothing if it names a server that is not allowed.
pub fn allowed_server_option(key: &str, value: String) -> String {
    if is_server_option_allowed(key, &value) {
        value
    } else {
        crate::throttled_log!(
            std::time::Duration::from_secs(60),
            warn,
            "Ignoring {}: {} is not an allowed server",
            key,
            value
        );
        String::new()
    }
}

fn is_allowed_by(address: &str, list: &str) -> bool {
    let entries: Vec<&str> = list
        .split(',')
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .collect();
    if entries.is_empty() {
        return true;
    }
    let Some(host) = host_of(address) else {
        return false;
    };
    match host.parse::<std::net::IpAddr>() {
        Ok(ip) => entries
            .iter()
            .any(|e| host_of(e).and_then(|e| e.parse().ok()) == Some(ip)),
        Err(_) => entries.iter().any(|e| {
            match host_of(e.trim_start_matches("*.")) {
                Some(domain) if domain.parse::<std::net::IpAddr>().is_err() => {
                    host == domain || host.ends_with(&format!(".{}", domain))
                }
                _ => false,
            }
        }),
    }
}

/// The host in `address` (host or IP address, with or without port, or a URL), in lower case
/// and without a trailing dot. `None` unless it is a plain host name or IP address.
fn host_of(address: &str) -> Option<String> {
    let mut s = address.trim();
    if let Some(i) = s.find("://") {
        s = &s[i + 3..];
    }
    // The authority ends at the path, query or fragment; URL parsers take a backslash as a slash.
    let end = s
        .find(|c: char| matches!(c, '/' | '\\' | '?' | '#'))
        .unwrap_or(s.len());
    s = &s[..end];
    if let Some(i) = s.rfind('@') {
        s = &s[i + 1..];
    }
    let host = if let Some(rest) = s.strip_prefix('[') {
        rest.split_once(']')?.0
    } else if s.matches(':').count() > 1 {
        s
    } else {
        s.split(':').next().unwrap_or_default()
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host.parse::<std::net::IpAddr>().is_ok() {
        return Some(host);
    }
    let is_label = |label: &str| {
        !label.is_empty()
            && label
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    };
    let is_host_name = host.split('.').all(is_label);
    is_host_name.then_some(host)
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

    #[test]
    fn fixed_options_cannot_be_changed() {
        use crate::config::Config;

        Config::set_option("direct-server".to_owned(), "Y".to_owned());
        Config::set_option("allow-remote-config-modification".to_owned(), "Y".to_owned());
        assert_eq!(Config::get_option("direct-server"), "N");
        assert_eq!(Config::get_option("allow-remote-config-modification"), "N");
        assert_eq!(Config::get_options()["enable-lan-discovery"], "N");
    }

    #[test]
    fn permanent_password_has_at_least_8_characters() {
        use crate::config::Config;

        assert!(!Config::set_permanent_password("1234567"));
        assert!(!Config::set_permanent_password("ÄÖÜäöüß"));
        assert!(is_permanent_password_allowed("12345678"));
        assert!(is_permanent_password_allowed("ÄÖÜäöüßé"));
        assert!(is_permanent_password_allowed(""));
    }

    #[test]
    fn only_allowed_servers() {
        let list = "klg-orga.de, felixthedev.com, 192.0.2.10";
        for address in [
            "felixthedev.com",
            "remote.felixthedev.com",
            "Remote.FelixTheDev.com.",
            "remote.felixthedev.com:21116",
            "https://remote.felixthedev.com",
            "wss://a.b.klg-orga.de:443/ws/id",
            "192.0.2.10",
            "192.0.2.10:21116",
        ] {
            assert!(is_allowed_by(address, list), "{}", address);
        }
        for address in [
            "",
            "evilfelixthedev.com",
            "felixthedev.com.evil.example",
            "felixthedev.co",
            "https://felixthedev.com@evil.example",
            "https://evil.example\\.felixthedev.com",
            "evil.example/.felixthedev.com",
            "192.0.2.11",
            "[2001:db8::1]:21116",
            "felixthedev.com%2eevil.example",
        ] {
            assert!(!is_allowed_by(address, list), "{}", address);
        }
        assert!(is_allowed_by("anything.example", ""));
        assert!(is_allowed_by("anything.example", " , "));
        // A list that cannot be read allows nothing rather than everything.
        assert!(!is_allowed_by("felixthedev.com", "felixthedev.com;klg-orga.de"));
        assert!(is_allowed_by("[2001:db8::1]:21116", "2001:db8:0::1"));
        assert!(is_allowed_by("a.felixthedev.com", "*.felixthedev.com"));
    }
}
