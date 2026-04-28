use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum OsFamily {
    Debian, // Includes Ubuntu
    Fedora, // Includes RHEL, CentOS
    Alpine,
    Unknown,
}

pub fn detect_os() -> OsFamily {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("ID_LIKE=") {
                let id_like = line.trim_start_matches("ID_LIKE=").trim_matches('"');
                if id_like.contains("debian") || id_like.contains("ubuntu") {
                    return OsFamily::Debian;
                }
                if id_like.contains("fedora") || id_like.contains("rhel") || id_like.contains("centos") {
                    return OsFamily::Fedora;
                }
                if id_like.contains("alpine") {
                    return OsFamily::Alpine;
                }
            } else if line.starts_with("ID=") {
                let id = line.trim_start_matches("ID=").trim_matches('"');
                if id == "debian" || id == "ubuntu" {
                    return OsFamily::Debian;
                }
                if id == "fedora" || id == "rhel" || id == "centos" {
                    return OsFamily::Fedora;
                }
                if id == "alpine" {
                    return OsFamily::Alpine;
                }
            }
        }
    }
    OsFamily::Unknown
}

// Ensure it returns a string suitable for logging
impl std::fmt::Display for OsFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsFamily::Debian => write!(f, "Debian/Ubuntu"),
            OsFamily::Fedora => write!(f, "Fedora/RHEL"),
            OsFamily::Alpine => write!(f, "Alpine Linux"),
            OsFamily::Unknown => write!(f, "Unknown"),
        }
    }
}
