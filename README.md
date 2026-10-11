# opskit

Shared plumbing for a small family of single-binary host ops tools written in Rust:

- [logscan](https://github.com/samothx/logscan): daily security review of journal and web logs (declarative rules plus optional LLM skills)
- [vulnwatch](https://github.com/samothx/vulnwatch): inventory-based vulnerability digest (Debian/Arch trackers, OSV, Trivy, CISA KEV, EPSS)
- [sitewatch](https://github.com/samothx/sitewatch): website watchdog for stale cached pages, missing assets and incomplete renders

Only the parts that are the same for every tool live here. Policy (what to mail, when to push, how
to read a model's answer) stays in the tools.

| Module | What it does |
|---|---|
| `severity` | `info < low < medium < high < critical`, parsing, raising/lowering, CVSS bands |
| `mail` | Markdown report → multipart mail (text + HTML, quoted-printable) through `sendmail` |
| `notify` | Push messages through an `ntfy-dispatcher`-style command (event, title, body, priority) |
| `llm` | Minimal OpenAI-compatible chat client (`ureq`), plus helpers for thinking tags and JSON answers |
| `state` | Atomic JSON state files (write to a temp file, then rename) |
| `secret` | API keys and tokens: environment override, then a file (aware of systemd `LoadCredential=`) |

## Use

```toml
[dependencies]
opskit = { git = "https://github.com/samothx/opskit", tag = "v0.1.0" }
```

To work on opskit and a tool together, point the tool at a local checkout without committing it:

```bash
cargo build --config 'patch."https://github.com/samothx/opskit".opskit.path="../opskit"'
```

## Build

```bash
cargo test
```

Rust edition 2024. Not published on crates.io.
