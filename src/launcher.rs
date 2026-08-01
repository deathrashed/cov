use crate::paths;
use anyhow::{Context, Result, bail};
use id3::TagLike;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub path: String,
    pub embed: bool,
    pub output: String,
    pub covit: PathBuf,
    pub log: PathBuf,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub identifier: Option<String>,
    pub country: Option<String>,
    pub resolution: Option<String>,
    pub sources: Option<String>,
    pub foreground: bool,
}

pub fn forced_quote(val: &str) -> String {
    format!("'{}'", val.replace('\'', "'\"'\"'"))
}

pub fn build_embed_command(album_dir: &Path) -> String {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("cov"));
    let exe_str = exe.to_string_lossy();
    // Use `exe embed` instead of `cov-embed` since it's a unified binary now
    format!(
        "{} embed \"@covit_path@\" {} --rescan-swinsian",
        forced_quote(&exe_str),
        forced_quote(&album_dir.to_string_lossy())
    )
}

pub fn build_covit_argv(audio_path: &Path, opts: &LaunchOptions) -> Vec<String> {
    let album_dir = audio_path.parent().unwrap_or(audio_path);

    let remote_text = if opts.embed {
        "Using COV at https://covers.musichoarders.xyz/ \u{2014} select artwork to save and embed it into this album."
    } else {
        "Using COV at https://covers.musichoarders.xyz/ \u{2014} select artwork to save it beside this album."
    };

    let mut argv = vec![
        opts.covit.to_string_lossy().to_string(),
        "--address".to_string(),
        "covers.musichoarders.xyz".to_string(),
        "--input".to_string(),
        audio_path.to_string_lossy().to_string(),
        "--primary-output".to_string(),
        opts.output.clone(),
        "--primary-overwrite".to_string(),
        "--remote-agent".to_string(),
        "Riley COV Toolkit".to_string(),
        "--remote-text".to_string(),
        remote_text.to_string(),
    ];

    if opts.embed {
        argv.push("--primary-command".to_string());
        argv.push(build_embed_command(album_dir));
    }

    let (tag_artist, tag_album) = query_tags(audio_path);
    if let Some(artist) = opts.artist.as_ref().or(tag_artist.as_ref()) {
        argv.push("--query-artist".to_string());
        argv.push(artist.clone());
    }
    if let Some(album) = opts.album.as_ref().or(tag_album.as_ref()) {
        argv.push("--query-album".to_string());
        argv.push(album.clone());
    }
    if let Some(ref identifier) = opts.identifier {
        argv.push("--query-identifier".to_string());
        argv.push(identifier.clone());
    }
    if let Some(ref country) = opts.country {
        argv.push("--query-country".to_string());
        argv.push(country.clone());
    }
    if let Some(ref resolution) = opts.resolution {
        argv.push("--query-resolution".to_string());
        argv.push(resolution.clone());
    }
    if let Some(ref sources) = opts.sources {
        argv.push("--query-sources".to_string());
        argv.push(sources.clone());
    }

    argv
}

fn query_tags(audio_path: &Path) -> (Option<String>, Option<String>) {
    if audio_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mp3"))
        && let Ok(tag) = id3::Tag::read_from_path(audio_path)
    {
        return (
            tag.artist().map(str::to_owned),
            tag.album().map(str::to_owned),
        );
    }
    let tagged_file = match lofty::probe::Probe::open(audio_path).and_then(|probe| probe.read()) {
        Ok(tagged_file) => tagged_file,
        Err(_) => return (None, None),
    };
    let Some(tag) = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
    else {
        return (None, None);
    };
    (
        tag.artist().map(|value| value.into_owned()),
        tag.album().map(|value| value.into_owned()),
    )
}

pub fn build_background_script(argv: &[String], log_path: &Path) -> (String, String) {
    let quoted_log = shlex_quote(&log_path.to_string_lossy());
    let joined_cmd = shlex_join(argv);

    let bg_bash_cmd = format!(
        ": > {}; /usr/bin/env 'PATH=/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin' /usr/bin/nohup {} >> {} 2>&1 < /dev/null &",
        quoted_log, joined_cmd, quoted_log
    );

    let applescript = r#"on run argv
  do shell script (item 1 of argv)
end run"#;

    (bg_bash_cmd, applescript.to_string())
}

fn shlex_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if !s.chars().any(|c| "|&;<>()$`\\\"' \t\n*?[#~=%".contains(c)) {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

fn shlex_join(argv: &[String]) -> String {
    argv.iter()
        .map(|s| shlex_quote(s))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn launch(opts: &LaunchOptions) -> Result<(PathBuf, Vec<String>)> {
    let covit_meta = fs::metadata(&opts.covit).context(format!(
        "COVIT is missing or not executable: {}",
        opts.covit.display()
    ))?;
    if !covit_meta.is_file() || covit_meta.permissions().mode() & 0o111 == 0 {
        bail!(
            "COVIT is missing or not executable: {}",
            opts.covit.display()
        );
    }

    let audio_path = paths::resolve_audio_path(&opts.path)?;
    let argv = build_covit_argv(&audio_path, opts);

    if opts.foreground {
        let status = Command::new(&argv[0])
            .args(&argv[1..])
            .env(
                "PATH",
                "/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
            )
            .status()
            .context("Failed to spawn COVIT")?;
        if !status.success() {
            bail!("COVIT exited with status {}", status);
        }
    } else {
        if let Some(parent) = opts.log.parent() {
            fs::create_dir_all(parent)?;
        }

        let (bg_bash_cmd, applescript) = build_background_script(&argv, &opts.log);

        let status = Command::new("/usr/bin/osascript")
            .arg("-e")
            .arg(&applescript)
            .arg("--")
            .arg(&bg_bash_cmd)
            .env(
                "PATH",
                "/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
            )
            .status()
            .context("Failed to spawn background launcher")?;

        if !status.success() {
            bail!("Background launcher exited with status {}", status);
        }
    }

    Ok((audio_path, argv))
}

#[cfg(test)]
mod tests {
    use super::*;
    use id3::TagLike;
    use tempfile::tempdir;

    fn option_value<'a>(argv: &'a [String], option: &str) -> Option<&'a str> {
        argv.iter()
            .position(|value| value == option)
            .and_then(|index| argv.get(index + 1))
            .map(String::as_str)
    }

    #[test]
    fn build_covit_argv_uses_complete_audio_tags_as_queries() {
        let temp = tempdir().unwrap();
        let audio_path = temp.path().join("01-hell-awaits.mp3");
        std::fs::File::create(&audio_path).unwrap();
        let mut tag = id3::Tag::new();
        tag.set_artist("Slayer");
        tag.set_album("Hell Awaits");
        tag.write_to_path(&audio_path, id3::Version::Id3v24)
            .unwrap();
        let options = LaunchOptions {
            path: audio_path.to_string_lossy().to_string(),
            embed: false,
            output: "cover".to_string(),
            covit: PathBuf::from("/usr/bin/covit"),
            log: temp.path().join("cov.log"),
            artist: None,
            album: None,
            identifier: None,
            country: None,
            resolution: None,
            sources: None,
            foreground: false,
        };

        let argv = build_covit_argv(&audio_path, &options);

        assert_eq!(option_value(&argv, "--query-artist"), Some("Slayer"));
        assert_eq!(option_value(&argv, "--query-album"), Some("Hell Awaits"));
    }
}
