//! Optional GitHub publish helper (Module 9): creates a public gist with `.oclexpert` JSON, then opens an issue linking it.
//! Runs server-side to avoid browser CORS against `api.github.com`.

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubPublishOclexpertRecipeRequest {
    pub token: String,
    /// Target repository for the tracking issue, e.g. `owner/repo`.
    pub issue_repo: String,
    pub title: String,
    /// Markdown shown before the gist link.
    pub issue_body_intro: String,
    pub oclexpert_filename: String,
    pub oclexpert_content: String,
    pub gist_description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubPublishOclexpertRecipeResponse {
    pub gist_url: String,
    pub issue_url: String,
}

#[derive(Debug, Deserialize)]
struct GistCreateResponse {
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct IssueCreateResponse {
    html_url: String,
}

fn gh_headers(token: &str) -> Result<HeaderMap, String> {
    let mut m = HeaderMap::new();
    let auth = format!("Bearer {}", token.trim());
    m.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth).map_err(|_| "Invalid GitHub token characters.".to_string())?,
    );
    m.insert(ACCEPT, HeaderValue::from_static("application/vnd.github+json"));
    m.insert(USER_AGENT, HeaderValue::from_static("OCLive-Desktop/0.2"));
    Ok(m)
}

fn sanitize_gist_filename(name: &str) -> String {
    let t = name.trim();
    if t.is_empty() {
        return "recipe.oclexpert".to_string();
    }
    let mut out: String = t
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
        .collect();
    if !out.ends_with(".oclexpert") {
        out.push_str(".oclexpert");
    }
    out
}

#[tauri::command]
pub async fn github_publish_oclexpert_recipe(
    req: GithubPublishOclexpertRecipeRequest,
) -> Result<GithubPublishOclexpertRecipeResponse, String> {
    let token = req.token.trim();
    if token.is_empty() {
        return Err("GitHub token is empty.".to_string());
    }
    let issue_repo = req.issue_repo.trim();
    if issue_repo.is_empty() || !issue_repo.contains('/') {
        return Err("issueRepo must look like owner/repo.".to_string());
    }
    let parts: Vec<&str> = issue_repo.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() != 2 {
        return Err("issueRepo must be exactly owner/repo.".to_string());
    }
    let (owner, repo) = (parts[0], parts[1]);

    let client = Client::builder()
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;

    let gist_fname = sanitize_gist_filename(&req.oclexpert_filename);
    let gist_desc = req
        .gist_description
        .clone()
        .unwrap_or_else(|| "OCLive personality recipe (.oclexpert)".to_string());

    let gist_body = serde_json::json!({
        "description": gist_desc,
        "public": true,
        "files": {
            gist_fname.as_str(): { "content": req.oclexpert_content }
        }
    });

    let gist_res = client
        .post("https://api.github.com/gists")
        .headers(gh_headers(token)?)
        .json(&gist_body)
        .send()
        .await
        .map_err(|e| format!("GitHub gist request failed: {e}"))?;

    if !gist_res.status().is_success() {
        let txt = gist_res.text().await.unwrap_or_default();
        return Err(format!("GitHub gist error: {txt}"));
    }

    let gist: GistCreateResponse = gist_res
        .json()
        .await
        .map_err(|e| format!("GitHub gist JSON: {e}"))?;

    let gist_url = gist.html_url.trim().to_string();
    if gist_url.is_empty() {
        return Err("GitHub returned an empty gist URL.".to_string());
    }

    let issue_body = format!(
        "{}\n\n---\n**Attached recipe (public gist):** {}",
        req.issue_body_intro.trim(),
        gist_url
    );

    let issue_payload = serde_json::json!({
        "title": req.title.trim(),
        "body": issue_body,
    });

    let issue_url_ep = format!("https://api.github.com/repos/{owner}/{repo}/issues");
    let issue_res = client
        .post(&issue_url_ep)
        .headers(gh_headers(token)?)
        .json(&issue_payload)
        .send()
        .await
        .map_err(|e| format!("GitHub issue request failed: {e}"))?;

    if !issue_res.status().is_success() {
        let txt = issue_res.text().await.unwrap_or_default();
        return Err(format!(
            "GitHub issue error (gist was created at {gist_url}): {txt}"
        ));
    }

    let issue: IssueCreateResponse = issue_res
        .json()
        .await
        .map_err(|e| format!("GitHub issue JSON: {e}"))?;

    let issue_url = issue.html_url.trim().to_string();
    if issue_url.is_empty() {
        return Err(format!(
            "GitHub returned an empty issue URL (gist: {gist_url})."
        ));
    }

    Ok(GithubPublishOclexpertRecipeResponse {
        gist_url,
        issue_url,
    })
}
