//! Pushes via `ntfy-dispatcher emit`; the dispatcher's rule file maps events to topics.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub struct Message {
    pub event: &'static str,
    pub title: String,
    pub body: String,
    pub priority: Option<u8>,
}

pub fn emit(command: &Path, m: &Message) -> Result<()> {
    let mut cmd = Command::new(command);
    cmd.args(["emit", "--event", m.event, "--title", &m.title, "--body", "-"]);
    if let Some(p) = m.priority {
        cmd.args(["--priority", &p.to_string()]);
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("starting {}", command.display()))?;
    child.stdin.take().expect("piped").write_all(m.body.as_bytes())?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        bail!("ntfy-dispatcher emit failed ({}): {}", out.status, String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}
