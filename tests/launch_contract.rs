use cov::launcher::{LaunchOptions, build_background_script, build_covit_argv, forced_quote};
use std::path::{Path, PathBuf};

#[test]
fn test_forced_quote() {
    assert_eq!(forced_quote("simple"), "'simple'");
    assert_eq!(forced_quote("path with spaces"), "'path with spaces'");
    assert_eq!(forced_quote("it's a path"), "'it'\"'\"'s a path'");
}

#[test]
fn test_build_covit_argv_save() {
    let opts = LaunchOptions {
        path: "dummy".into(),
        embed: false,
        output: "cover".into(),
        covit: PathBuf::from("/usr/bin/covit"),
        log: PathBuf::from("log.txt"),
        artist: None,
        album: None,
        identifier: None,
        country: None,
        resolution: None,
        sources: None,
        foreground: false,
    };

    let audio_path = Path::new("/music/artist/album/01 track.flac");
    let argv = build_covit_argv(audio_path, &opts);

    assert_eq!(argv[0], "/usr/bin/covit");
    assert_eq!(argv[1], "--address");
    assert_eq!(argv[2], "covers.musichoarders.xyz");
    assert_eq!(argv[3], "--input");
    assert_eq!(argv[4], "/music/artist/album/01 track.flac");
    assert_eq!(argv[5], "--primary-output");
    assert_eq!(argv[6], "cover");
    assert_eq!(argv[7], "--primary-overwrite");
    assert_eq!(argv[8], "--remote-agent");
    assert_eq!(argv[9], "Riley COV Toolkit");
    assert_eq!(argv[10], "--remote-text");
    assert_eq!(
        argv[11],
        "Using COV at https://covers.musichoarders.xyz/ \u{2014} select artwork to save it beside this album."
    );
    assert_eq!(argv.len(), 12);
}

#[test]
fn test_build_covit_argv_embed() {
    let opts = LaunchOptions {
        path: "dummy".into(),
        embed: true,
        output: "cover".into(),
        covit: PathBuf::from("/usr/bin/covit"),
        log: PathBuf::from("log.txt"),
        artist: Some("The Artist".into()),
        album: Some("The Album".into()),
        identifier: None,
        country: None,
        resolution: None,
        sources: None,
        foreground: false,
    };

    let audio_path = Path::new("/music/artist/album/01 track.flac");
    let argv = build_covit_argv(audio_path, &opts);

    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("cov"));
    let expected_embed_cmd = format!(
        "{} embed \"@covit_path@\" '/music/artist/album' --rescan-swinsian",
        forced_quote(&exe.to_string_lossy())
    );

    assert_eq!(
        argv[11],
        "Using COV at https://covers.musichoarders.xyz/ \u{2014} select artwork to save and embed it into this album."
    );
    assert_eq!(argv[12], "--primary-command");
    assert_eq!(argv[13], expected_embed_cmd);

    // Check queries
    assert_eq!(argv[14], "--query-artist");
    assert_eq!(argv[15], "The Artist");
    assert_eq!(argv[16], "--query-album");
    assert_eq!(argv[17], "The Album");
}

#[test]
fn test_build_background_script() {
    let argv = vec![
        "/usr/bin/covit".to_string(),
        "--address".to_string(),
        "covers.musichoarders.xyz".to_string(),
        "--input".to_string(),
        "/music/artist/album/01 track.flac".to_string(),
    ];
    let log_path = Path::new("/var/log/cov.log");

    let (bg_bash, applescript) = build_background_script(&argv, log_path);

    let expected_bash = ": > /var/log/cov.log; /usr/bin/env 'PATH=/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin' /usr/bin/nohup /usr/bin/covit --address covers.musichoarders.xyz --input '/music/artist/album/01 track.flac' >> /var/log/cov.log 2>&1 < /dev/null &";
    assert_eq!(bg_bash, expected_bash);

    let expected_applescript = "on run argv\n  do shell script (item 1 of argv)\nend run";
    assert_eq!(applescript, expected_applescript);
}
