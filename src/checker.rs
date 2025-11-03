use crate::sites::Site;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckResult {
    Found,
    NotFound,
    Error(String),
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteResult {
    pub site: String,
    pub url: String,
    pub category: String,
    pub result: CheckResult,
}

pub struct AccountChecker {
    client: Client,
}

impl AccountChecker {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn check_account(&self, site: &Site, username: &str) -> SiteResult {
        // Special handling for Discord since it uses IDs, not usernames in URLs
        if site.name == "Discord" {
            return self.check_discord_username(username).await;
        }
        
        // Remove or skip sites that are shut down
        if site.name == "Mixer" {
            return SiteResult {
                site: site.name.clone(),
                url: site.url.replace("{}", username),
                category: site.category.clone(),
                result: CheckResult::Error("Mixer was shut down in 2020.".to_string()),
            };
        }
        
        // Skip Spotify Artist - uses IDs, not usernames
        if site.name == "Spotify Artist" {
            return SiteResult {
                site: site.name.clone(),
                url: site.url.replace("{}", username),
                category: site.category.clone(),
                result: CheckResult::Error("Spotify Artist URLs use IDs, not usernames.".to_string()),
            };
        }
        
        let url = site.url.replace("{}", username);
        
        match self.check_url(&url, username, &site.name).await {
            CheckResult::Found => SiteResult {
                site: site.name.clone(),
                url: url.clone(),
                category: site.category.clone(),
                result: CheckResult::Found,
            },
            CheckResult::NotFound => SiteResult {
                site: site.name.clone(),
                url: url.clone(),
                category: site.category.clone(),
                result: CheckResult::NotFound,
            },
            CheckResult::Error(e) => SiteResult {
                site: site.name.clone(),
                url: url.clone(),
                category: site.category.clone(),
                result: CheckResult::Error(e),
            },
            CheckResult::Timeout => SiteResult {
                site: site.name.clone(),
                url: url.clone(),
                category: site.category.clone(),
                result: CheckResult::Timeout,
            },
        }
    }

    async fn check_url(&self, url: &str, _username: &str, site_name: &str) -> CheckResult {
        let response = match self.client.get(url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    return CheckResult::Timeout;
                }
                // Handle DNS errors and SSL errors more gracefully
                let error_msg = e.to_string();
                if error_msg.contains("dns error") || error_msg.contains("failed to lookup address") {
                    return CheckResult::Error(format!("DNS error: Site may be down or domain changed"));
                } else if error_msg.contains("certificate verify failed") || error_msg.contains("SSL") {
                    return CheckResult::Error(format!("SSL certificate error: Site may have certificate issues"));
                }
                return CheckResult::Error(format!("Network error: {}", e));
            }
        };

        let status = response.status();
        let body = response.text().await.unwrap_or_default().to_lowercase();

        // Check status code
        match status.as_u16() {
            200 => {
                // Account exists if status is 200 and doesn't contain "not found" messages
                if self.contains_not_found_message(&body) {
                    CheckResult::NotFound
                } else {
                    CheckResult::Found
                }
            }
            404 => CheckResult::NotFound,
            403 => {
                // 403 might mean account exists but is private, or account doesn't exist
                // Check body for not found messages
                if self.contains_not_found_message(&body) {
                    CheckResult::NotFound
                } else {
                    // Likely private/exists but blocked
                    CheckResult::Found
                }
            }
            302 | 301 | 307 | 308 => {
                // Redirect might indicate account exists or doesn't exist
                // Try to check the final location if possible
                CheckResult::Found
            }
            400 => {
                // Bad request - might be invalid username format or requires auth
                if self.contains_not_found_message(&body) {
                    CheckResult::NotFound
                } else {
                    CheckResult::Error(format!("HTTP 400 Bad Request (possibly requires authentication)"))
                }
            }
            429 => {
                // Rate limited - return error but don't fail completely
                CheckResult::Error(format!("HTTP 429 Rate Limited (try again later)"))
            }
            520 | 521 | 522 | 523 | 524 => {
                // Cloudflare errors - site might be down
                CheckResult::Error(format!("HTTP {} Cloudflare Error (site may be temporarily unavailable)", status))
            }
            999 => {
                // LinkedIn's anti-bot protection
                CheckResult::Error(format!("HTTP 999 Anti-bot protection (requires authentication)"))
            }
            _ => {
                // Check body for not found messages even with other status codes
                if self.contains_not_found_message(&body) {
                    CheckResult::NotFound
                } else if status.is_success() {
                    CheckResult::Found
                } else {
                    // Handle DNS/SSL errors more gracefully
                    if site_name == "MySpace" || site_name == "Ask.fm" {
                        CheckResult::Error(format!("HTTP {} (site may be unavailable or requires SSL verification)", status))
                    } else {
                        CheckResult::Error(format!("HTTP {}", status))
                    }
                }
            }
        }
    }

    fn contains_not_found_message(&self, body: &str) -> bool {
        let not_found_patterns = vec![
            "account not found",
            "user not found",
            "error: user not found",
            "profile not found",
            "page not found",
            "404",
            "user does not exist",
            "account does not exist",
            "this page doesn't exist",
            "this profile doesn't exist",
            "the page you requested was not found",
            "sorry, this page isn't available",
            "the link you followed may be broken",
            "couldn't find this account",
            "this account doesn't exist",
            "no such user",
            "user does not exist",
            "invalid username",
            "username does not exist",
            "this user does not exist",
            "user is not found",
            "profile not available",
            "account is not available",
            "this profile is not available",
            "user profile not found",
            "the user you are looking for",
            "doesn't have an account",
            "could not find user",
            "unable to find user",
            "user not available",
            "account unavailable",
            "profile unavailable",
            "not a registered user",
            "user not registered",
            "no account associated",
        ];

        let body_lower = body.to_lowercase();
        not_found_patterns.iter().any(|pattern| body_lower.contains(pattern))
    }

    async fn check_discord_username(&self, username: &str) -> SiteResult {
        // Discord uses user IDs in URLs, not usernames. 
        // We'll try to check via Discord's API validation endpoint.
        // Note: This is unreliable without authentication, but we'll attempt it.
        
        // Discord's username validation endpoint (used during registration)
        let validation_url = "https://discord.com/api/v9/unique-username/username-attempt-unauthed";

        let payload = serde_json::json!({
            "username": username
        });

        match self.client
            .post(validation_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                match status.as_u16() {
                    200 => {
                        // Discord's API returns 200 for valid username format checks
                        // The response body contains info about availability
                        let body = response.text().await.unwrap_or_default().to_lowercase();
                        
                        // Check if username is taken/exists
                        // Discord API typically returns "taken" or similar indicators
                        if body.contains("\"taken\":true") || body.contains("username_taken") 
                            || body.contains("\"available\":false") {
                            SiteResult {
                                site: "Discord".to_string(),
                                url: format!("https://discord.com/users/{}", username),
                                category: "Social".to_string(),
                                result: CheckResult::Found,
                            }
                        } else if body.contains("\"taken\":false") || body.contains("\"available\":true") {
                            // Username is available, so account doesn't exist
                            SiteResult {
                                site: "Discord".to_string(),
                                url: format!("https://discord.com/users/{}", username),
                                category: "Social".to_string(),
                                result: CheckResult::NotFound,
                            }
                        } else {
                            // Can't determine - Discord uses user IDs, not usernames in URLs
                            // Without proper API authentication, we can't reliably check
                            SiteResult {
                                site: "Discord".to_string(),
                                url: format!("https://discord.com/users/{}", username),
                                category: "Social".to_string(),
                                result: CheckResult::Error(
                                    "Discord uses user IDs, not usernames in URLs. Cannot reliably check without authentication.".to_string()
                                ),
                            }
                        }
                    }
                    400 | 422 => {
                        // Invalid username format
                        SiteResult {
                            site: "Discord".to_string(),
                            url: format!("https://discord.com/users/{}", username),
                            category: "Social".to_string(),
                            result: CheckResult::NotFound,
                        }
                    }
                    401 | 403 => {
                        // Rate limited or requires authentication
                        SiteResult {
                            site: "Discord".to_string(),
                            url: format!("https://discord.com/users/{}", username),
                            category: "Social".to_string(),
                            result: CheckResult::Error(
                                "Discord API requires authentication. Discord uses user IDs, not usernames in URLs.".to_string()
                            ),
                        }
                    }
                    _ => {
                        SiteResult {
                            site: "Discord".to_string(),
                            url: format!("https://discord.com/users/{}", username),
                            category: "Social".to_string(),
                            result: CheckResult::Error(format!(
                                "Discord API returned status: {} (Discord uses user IDs, not usernames in URLs)",
                                status
                            )),
                        }
                    }
                }
            }
            Err(e) => {
                SiteResult {
                    site: "Discord".to_string(),
                    url: format!("https://discord.com/users/{}", username),
                    category: "Social".to_string(),
                    result: CheckResult::Error(format!(
                        "Unable to check Discord: {} (Discord uses user IDs, not usernames in URLs)",
                        e
                    )),
                }
            }
        }
    }
}

impl Default for AccountChecker {
    fn default() -> Self {
        Self::new()
    }
}

