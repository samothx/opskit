//! Minimal OpenAI-compatible chat client (Venice, DeepSeek, …). Answer parsing is the caller's job.

use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use serde_json::{Value, json};

use crate::truncate;

pub struct ClientConfig {
    /// Base URL without /chat/completions, e.g. https://api.venice.ai/api/v1
    pub base_url: String,
    pub api_key: String,
    pub timeout_secs: u64,
    pub max_tokens: u32,
    /// Send `response_format: {"type": "json_object"}`.
    pub json_mode: bool,
    /// Merged into the request body as-is (provider-specific knobs). Must be an object or null.
    pub extra_body: Value,
    pub user_agent: String,
}

pub struct Client {
    agent: ureq::Agent,
    base: String,
    key: String,
    max_tokens: u32,
    json_mode: bool,
    extra: Value,
}

pub struct Reply {
    pub content: String,
    pub usage: Option<Value>,
    pub finish_reason: String,
}

impl Client {
    pub fn new(cfg: ClientConfig) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(cfg.timeout_secs)))
            .timeout_connect(Some(Duration::from_secs(10)))
            .http_status_as_error(false)
            .user_agent(cfg.user_agent)
            .build()
            .into();
        Self {
            agent,
            base: cfg.base_url.trim_end_matches('/').to_string(),
            key: cfg.api_key,
            max_tokens: cfg.max_tokens,
            json_mode: cfg.json_mode,
            extra: cfg.extra_body,
        }
    }

    fn check(resp: &mut ureq::http::Response<ureq::Body>) -> Result<Value> {
        let status = resp.status().as_u16();
        let body = resp.body_mut().with_config().limit(16 * 1024 * 1024).read_to_string().unwrap_or_default();
        if !(200..300).contains(&status) {
            bail!("HTTP {status}: {}", truncate(body.trim(), 600));
        }
        serde_json::from_str(&body).with_context(|| format!("response is not JSON: {}", truncate(&body, 300)))
    }

    pub fn models(&self) -> Result<Vec<String>> {
        let mut resp = self
            .agent
            .get(format!("{}/models", self.base))
            .header("Authorization", format!("Bearer {}", self.key))
            .call()
            .map_err(|e| anyhow!("GET /models: {e}"))?;
        let v = Self::check(&mut resp)?;
        let mut ids: Vec<String> = v["data"]
            .as_array()
            .map(|a| a.iter().filter_map(|m| m["id"].as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        ids.sort();
        Ok(ids)
    }

    pub fn chat(&self, model: &str, system: &str, user: &str) -> Result<Reply> {
        let mut body = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "max_tokens": self.max_tokens,
            "temperature": 0.2,
        });
        if self.json_mode {
            body["response_format"] = json!({"type": "json_object"});
        }
        if let (Some(b), Some(extra)) = (body.as_object_mut(), self.extra.as_object()) {
            for (k, v) in extra {
                b.insert(k.clone(), v.clone());
            }
        }
        let mut resp = self
            .agent
            .post(format!("{}/chat/completions", self.base))
            .header("Authorization", format!("Bearer {}", self.key))
            .send_json(&body)
            .map_err(|e| anyhow!("POST /chat/completions: {e}"))?;
        let v = Self::check(&mut resp)?;
        let choice = &v["choices"][0];
        let content = choice["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("no message content in response: {}", truncate(&v.to_string(), 300)))?;
        Ok(Reply {
            content: strip_think(content),
            usage: v.get("usage").cloned(),
            finish_reason: choice["finish_reason"].as_str().unwrap_or("").to_string(),
        })
    }
}

static THINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<think>.*?</think>").unwrap());

pub fn strip_think(s: &str) -> String {
    THINK.replace_all(s, "").trim().to_string()
}

/// The first `{` … last `}` of an answer (tolerates code fences and text around the JSON).
pub fn json_object(content: &str) -> Result<&str> {
    let start = content.find('{').ok_or_else(|| anyhow!("no JSON object in answer"))?;
    let end = content.rfind('}').ok_or_else(|| anyhow!("no JSON object in answer"))?;
    if end < start {
        bail!("no JSON object in answer");
    }
    Ok(&content[start..=end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_thinking() {
        assert_eq!(strip_think("<think>\na\n</think>\n{\"a\":1}"), "{\"a\":1}");
    }

    #[test]
    fn finds_json_object() {
        assert_eq!(json_object("```json\n{\"a\":{\"b\":1}}\n```").unwrap(), "{\"a\":{\"b\":1}}");
        assert!(json_object("none").is_err());
        assert!(json_object("} {").is_err());
    }
}
