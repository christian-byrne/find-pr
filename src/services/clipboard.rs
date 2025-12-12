use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

const CLIPBOARD_COMMANDS: &[(&str, &[&str])] = &[
    ("pbcopy", &[]),
    ("wl-copy", &["-n"]),
    ("xclip", &["-selection", "clipboard"]),
    ("xsel", &["--clipboard", "--input"]),
    ("clip.exe", &[]),
];

pub fn copy_to_clipboard(payload: &str, enabled: bool) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    for (command, args) in CLIPBOARD_COMMANDS {
        if let Ok(mut child) = Command::new(command)
            .args(*args)
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin
                    .write_all(payload.as_bytes())
                    .context("failed to write clipboard payload")?;
            }
            let status = child
                .wait()
                .with_context(|| format!("waiting on {command}"))?;
            if status.success() {
                return Ok(());
            }
        }
    }

    Err(anyhow::anyhow!(
        "no clipboard utility (pbcopy/wl-copy/xclip/xsel/clip.exe) found"
    ))
}
