use crate::config::Config;
use std::path::Path;

/// Result of a single diagnostic check.
#[derive(Debug, Clone)]
pub struct Check {
    pub label: String,
    pub ok: bool,
    pub detail: String,
}

fn check_exec(path: &Path, label: &str) -> Check {
    let detail = if path.is_file() {
        #[allow(unused_mut)]
        let mut extra = String::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(path)
                && meta.permissions().mode() & 0o111 == 0
            {
                extra = " (not executable)".to_string();
            }
        }
        format!("found at {}{}", path.display(), extra)
    } else {
        format!("not found at {}", path.display())
    };
    Check {
        label: label.to_string(),
        ok: path.is_file(),
        detail,
    }
}

/// Run all diagnostic checks.
pub fn run(cfg: &Config) -> Vec<Check> {
    let mut checks = vec![
        check_exec(&cfg.covit_path, "covit binary"),
        check_exec(Path::new("/usr/bin/open"), "open"),
        check_exec(Path::new("/usr/bin/osascript"), "osascript"),
        check_exec(Path::new("/usr/bin/pbpaste"), "pbpaste"),
    ];

    // Ghostty (optional)
    let ghostty_paths = [
        "/Applications/Ghostty.app/Contents/MacOS/ghostty",
        "/Applications/Ghostty.app/Contents/MacOS/Ghostty",
    ];
    let ghostty_found = ghostty_paths.iter().any(|p| Path::new(p).is_file());
    checks.push(Check {
        label: "Ghostty".to_string(),
        ok: ghostty_found,
        detail: if ghostty_found {
            "installed".to_string()
        } else {
            "not found (optional)".to_string()
        },
    });

    // library root (TUI) — optional for CLI-only use
    if let Some(ref root) = cfg.library_root {
        let exists = root.exists();
        checks.push(Check {
            label: "library root".to_string(),
            ok: exists,
            detail: if exists {
                format!("{} exists", root.display())
            } else {
                format!("{} does not exist", root.display())
            },
        });
    } else {
        checks.push(Check {
            label: "library root".to_string(),
            ok: true,
            detail: "not configured (TUI only)".to_string(),
        });
    }

    // Info lines
    checks.push(Check {
        label: "log path".to_string(),
        ok: true,
        detail: cfg.log_path.display().to_string(),
    });

    checks
}

/// Print a formatted report. Returns true if all checks pass.
pub fn print_report(checks: &[Check]) -> bool {
    let mut all_ok = true;
    for check in checks {
        let status = if check.ok { "PASS" } else { "FAIL" };
        if !check.ok {
            all_ok = false;
        }
        println!("{status:<5} {}: {}", check.label, check.detail);
    }
    if all_ok {
        println!("\nCOV toolkit is ready.");
    } else {
        println!("\nCOV toolkit has missing requirements.");
    }
    all_ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_run_has_checks() {
        let cfg = Config::default();
        let checks = run(&cfg);
        assert!(!checks.is_empty(), "should have at least some checks");
        // open, osascript, pbpaste should pass on macOS
        let open = checks.iter().find(|c| c.label == "open").unwrap();
        assert!(open.ok, "/usr/bin/open should exist on macOS");
        let osa = checks.iter().find(|c| c.label == "osascript").unwrap();
        assert!(osa.ok, "/usr/bin/osascript should exist on macOS");
    }

    #[test]
    fn test_print_report_synthetic() {
        let checks = vec![
            Check {
                label: "test".to_string(),
                ok: true,
                detail: "all good".to_string(),
            },
            Check {
                label: "test2".to_string(),
                ok: false,
                detail: "broken".to_string(),
            },
        ];
        // Just ensure it doesn't panic and returns false
        assert!(!print_report(&checks));
    }
}
