//! opskit: shared plumbing for the host ops tools (logscan, vulnwatch).
//!
//! Only things that are the same for every tool live here: the severity scale, text helpers,
//! mailing a Markdown report through sendmail, pushing through `ntfy-dispatcher`, a minimal
//! OpenAI-compatible chat client and atomic JSON state files. Policy (what to mail, when to
//! push, how to read a model's answer) stays in the tools.

pub mod llm;
pub mod mail;
pub mod notify;
pub mod secret;
pub mod severity;
pub mod state;

pub use severity::Severity;

pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

pub fn hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "localhost".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn truncate_respects_char_boundaries() {
        assert_eq!(super::truncate("abc", 5), "abc");
        assert_eq!(super::truncate("aä", 2), "a…");
    }
}
