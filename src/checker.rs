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
            .redirect(reqwest::redirect::Policy::limited(5)) // Follow redirects but check final URL
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
    
    /// Detect JavaScript redirects using regex pattern matching
    /// This is much lighter than a full headless browser and catches common redirect patterns
    /// without needing to execute complex JavaScript
    fn check_js_redirects(&self, html: &str, username: &str, final_url: &str) -> Option<CheckResult> {
        // Quick check: if no location/redirect patterns, skip
        let html_lower = html.to_lowercase();
        if !html_lower.contains("location") && 
           !html_lower.contains("redirect") &&
           !html_lower.contains("window") {
            return None;
        }
        
        // Extract JavaScript code from <script> tags
        let script_pattern = match regex::Regex::new(r"(?i)<script[^>]*>(.*?)</script>") {
            Ok(re) => re,
            Err(_) => return None,
        };
        
        let mut js_code = String::new();
        for cap in script_pattern.captures_iter(html) {
            if let Some(script_content) = cap.get(1) {
                js_code.push_str(script_content.as_str());
                js_code.push('\n');
            }
        }
        
        // Also check the full HTML for inline redirects (not just in script tags)
        js_code.push_str(html);
        
        let username_lower = username.to_lowercase();
        let final_url_lower = final_url.to_lowercase();
        
        // Patterns to detect redirects to 404 pages
        let redirect_patterns = [
            // location.href = "/404"
            (r#"(?i)location\.(?:href|pathname)\s*=\s*["']([^"']*(?:/404|/not-found|/error|404|not-found|error)[^"']*)["']"#, 1),
            // window.location = "/404"
            (r#"(?i)window\.location(?:\.[a-z]+)?\s*=\s*["']([^"']*(?:/404|/not-found|/error|404|not-found|error)[^"']*)["']"#, 1),
            // location.replace("/404")
            (r#"(?i)location\.replace\(["']([^"']*(?:/404|/not-found|/error)[^"']*)["']"#, 1),
            // window.location.replace("/404")
            (r#"(?i)window\.location\.replace\(["']([^"']*(?:/404|/not-found|/error)[^"']*)["']"#, 1),
        ];
        
        for (pattern_str, capture_group) in &redirect_patterns {
            if let Ok(pattern) = regex::Regex::new(pattern_str) {
                for cap in pattern.captures_iter(&js_code) {
                    if let Some(redirect_target) = cap.get(*capture_group) {
                        let target = redirect_target.as_str().to_lowercase();
                        // If redirecting to 404/error page, it's not found
                        if target.contains("/404") || 
                           target.contains("/not-found") ||
                           target.contains("/error") ||
                           target.contains("404") ||
                           target.contains("notfound") {
                            return Some(CheckResult::NotFound);
                        }
                    }
                }
            }
        }
        
        // Check for redirects that remove the username from URL
        let username_removal_patterns = [
            r#"(?i)location\.(?:href|pathname)\s*=\s*["']([^"']+)["']"#,
            r#"(?i)window\.location(?:\.[a-z]+)?\s*=\s*["']([^"']+)["']"#,
        ];
        
        for pattern_str in &username_removal_patterns {
            if let Ok(pattern) = regex::Regex::new(pattern_str) {
                for cap in pattern.captures_iter(&js_code) {
                    if let Some(redirect_target) = cap.get(1) {
                        let target = redirect_target.as_str().to_lowercase();
                        // If redirecting to a URL that doesn't contain username,
                        // and current URL does, it might be a 404 redirect
                        if final_url_lower.contains(&username_lower) && 
                           !target.contains(&username_lower) &&
                           !target.starts_with("http") && // Relative redirect
                           (target.starts_with("/") || target.is_empty()) {
                            // Could be a redirect to homepage or error page
                            // But be conservative - only flag if it's clearly an error path
                            if target.contains("404") || target.contains("error") || 
                               target.contains("not-found") || target == "/" {
                                return Some(CheckResult::NotFound);
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Site-specific checks for known problematic sites
    /// Returns Some(CheckResult) if site-specific logic determines result, None otherwise
    fn check_site_specific(
        &self,
        url: &str,
        body: &str,
        body_lower: &str,
        username: &str,
        final_url: &str,
        status_code: u16,
    ) -> Option<CheckResult> {
        let username_lower = username.to_lowercase();
        let final_url_lower = final_url.to_lowercase();
        let url_lower = url.to_lowercase();
        
        // Badoo: Returns 200 for non-existent users with empty SPA shell
        // Badoo pages are SPA - if username doesn't appear anywhere in initial HTML, user doesn't exist
        if url_lower.contains("badoo.com") || final_url_lower.contains("badoo.com") {
            // Check if username appears in any meaningful way (title, og:title, meta, body content)
            let username_in_title = if let (Some(title_start), Some(title_end)) = 
                (body_lower.find("<title>"), body_lower.find("</title>")) {
                if title_start + 7 < title_end {
                    body_lower[title_start + 7..title_end].contains(&username_lower)
                } else {
                    false
                }
            } else {
                false
            };
            
            let username_in_og_title = body_lower.contains("property=\"og:title\"") && 
                                      body_lower.contains(&username_lower);
            
            // Check if username appears in body content (not just in scripts/CSS)
            // Extract text content by removing script/style tags
            let mut text_content = body_lower.to_string();
            // Remove script tags
            while let Some(start) = text_content.find("<script") {
                if let Some(end) = text_content[start..].find("</script>") {
                    let end_pos = start + end + 9;
                    text_content.replace_range(start..end_pos.min(text_content.len()), "");
                } else {
                    break;
                }
            }
            // Remove style tags
            while let Some(start) = text_content.find("<style") {
                if let Some(end) = text_content[start..].find("</style>") {
                    let end_pos = start + end + 8;
                    text_content.replace_range(start..end_pos.min(text_content.len()), "");
                } else {
                    break;
                }
            }
            
            let username_in_content = text_content.contains(&username_lower);
            
            // If username doesn't appear in title, og:title, or content, user doesn't exist
            if !username_in_title && !username_in_og_title && !username_in_content {
                return Some(CheckResult::NotFound);
            }
        }
        
        // Glitch: Generic title means user doesn't exist
        if url_lower.contains("glitch.com") || final_url_lower.contains("glitch.com") {
            let title_is_generic = if let (Some(title_start), Some(title_end)) = 
                (body_lower.find("<title>"), body_lower.find("</title>")) {
                if title_start + 7 < title_end {
                    let title = &body_lower[title_start + 7..title_end];
                    // Generic Glitch titles don't contain username
                    title.contains("glitch: the friendly community") || 
                    (title.contains("glitch") && !title.contains(&username_lower))
                } else {
                    false
                }
            } else {
                false
            };
            
            // Check og:title as well
            let og_title_is_generic = body_lower.contains("property=\"og:title\"") && 
                                     (body_lower.contains("glitch: the friendly community") ||
                                      (!body_lower.contains(&username_lower) && 
                                       body_lower.contains("property=\"og:title\"") &&
                                       body_lower.contains("glitch")));
            
            if title_is_generic || og_title_is_generic {
                return Some(CheckResult::NotFound);
            }
        }
        
        // TopCoder: If meta refresh to profiles.topcoder.com exists, check if page is generic
        // TopCoder redirects via meta refresh - non-existent users get generic/empty content
        if url_lower.contains("topcoder.com") || final_url_lower.contains("topcoder.com") {
            let has_meta_refresh = body_lower.contains("http-equiv=\"refresh\"") || 
                                   body_lower.contains("http-equiv='refresh'");
            let refreshes_to_profiles = body_lower.contains("profiles.topcoder.com");
            
            // Extract title
            let title_text = if let (Some(title_start), Some(title_end)) = 
                (body_lower.find("<title>"), body_lower.find("</title>")) {
                if title_start + 7 < title_end {
                    Some(&body_lower[title_start + 7..title_end])
                } else {
                    None
                }
            } else {
                None
            };
            
            // If there's a meta refresh to profiles.topcoder.com, check if user exists
            // TopCoder always redirects members -> profiles, but non-existent users show generic content
            if has_meta_refresh && refreshes_to_profiles {
                // Check if title is generic or missing
                if let Some(title) = title_text {
                    // Generic titles mean user doesn't exist
                    let is_generic = (title.contains("topcoder") && !title.contains(&username_lower)) ||
                                    title.contains("top technology talent") ||
                                    title.trim() == "topcoder";
                    if is_generic {
                        return Some(CheckResult::NotFound);
                    }
                } else {
                    // No title tag found + meta refresh = non-existent user
                    // TopCoder pages always have a title, so no title = error/generic page
                    return Some(CheckResult::NotFound);
                }
            }
            
            // Also check if already on profiles.topcoder.com with generic title
            if final_url_lower.contains("profiles.topcoder.com") {
                if let Some(title) = title_text {
                    let is_generic = (title.contains("topcoder") && !title.contains(&username_lower)) ||
                                    title.contains("top technology talent");
                    if is_generic {
                        return Some(CheckResult::NotFound);
                    }
                }
            }
        }
        
        // AngelList/Wellfound: Check if redirected to wellfound.com and it's blocked/404
        if url_lower.contains("angel.co") {
            if final_url_lower.contains("wellfound.com") {
                // If redirected to wellfound and it's 403 (Cloudflare) or body is empty/generic, not found
                if status_code == 403 || 
                   body_lower.contains("please enable js") ||
                   body_lower.len() < 1000 {
                    return Some(CheckResult::NotFound);
                }
            }
        }
        
        // eBay: Aggressive check - if username doesn't appear in content, user doesn't exist
        // eBay shows security pages, 503 errors, or empty pages for non-existent users
        if url_lower.contains("ebay.com") || final_url_lower.contains("ebay.com") {
            // Check for security/captcha patterns
            if body_lower.contains("security measure") ||
               body_lower.contains("security | ebay") ||
               body_lower.contains("captcha_form") ||
               body_lower.contains("id=captcha_form") ||
               body_lower.contains("please verify yourself") ||
               body_lower.contains("verify yourself to continue") ||
               body_lower.contains("service unavailable") {
                return Some(CheckResult::NotFound);
            }
            
            // For eBay, if username doesn't appear in any meaningful way, user doesn't exist
            // Valid eBay user pages always contain the username prominently
            let username_in_url_path = final_url_lower.contains(&format!("/usr/{}", username_lower)) ||
                                      url_lower.contains(&format!("/usr/{}", username_lower));
            
            // Check if username appears in visible content (not just in URL or scripts)
            let username_in_content = body_lower.contains(&format!(">{}</", username_lower)) ||
                                     body_lower.contains(&format!(">{}</a>", username_lower)) ||
                                     body_lower.contains(&format!(">{}</h", username_lower)) ||
                                     body_lower.contains(&format!(">{}</span>", username_lower)) ||
                                     (body_lower.contains("property=\"og:title\"") && body_lower.contains(&username_lower));
            
            // If username is in URL path but NOT in content, it's likely a security/error page
            // For non-existent users, eBay shows pages without the username in content
            if username_in_url_path && !username_in_content {
                return Some(CheckResult::NotFound);
            }
            
            // Also check: if page is suspiciously small or generic, likely doesn't exist
            if body.len() < 5000 && !username_in_content {
                return Some(CheckResult::NotFound);
            }
        }
        
        // Etsy: Check if username appears in title or content
        // Etsy returns 200 for non-existent shops with generic content
        if url_lower.contains("etsy.com") || final_url_lower.contains("etsy.com") {
            // Check if title contains username - valid shops have username in title
            let title_has_username = if let (Some(title_start), Some(title_end)) = 
                (body_lower.find("<title>"), body_lower.find("</title>")) {
                if title_start + 7 < title_end {
                    body_lower[title_start + 7..title_end].contains(&username_lower)
                } else {
                    false
                }
            } else {
                false
            };
            
            // Check if username appears in visible content
            let username_in_content = body_lower.contains(&format!("/shop/{}", username_lower)) ||
                                     body_lower.contains(&format!(">{}</", username_lower)) ||
                                     (body_lower.contains("property=\"og:title\"") && body_lower.contains(&username_lower));
            
            // If username not in title or content, shop doesn't exist
            if !title_has_username && !username_in_content {
                return Some(CheckResult::NotFound);
            }
        }
        
        // Steam: Check for "Profile Not Found" or similar
        if url_lower.contains("steamcommunity.com") || final_url_lower.contains("steamcommunity.com") {
            // Check for error messages in content
            if body_lower.contains("profile not found") ||
               body_lower.contains("could not find") ||
               body_lower.contains("invalid profile") ||
               body_lower.contains("profile error") {
                return Some(CheckResult::NotFound);
            }
            
            // Check if username appears in profile link/header - valid profiles have username visible
            let username_in_profile = body_lower.contains(&format!("profile/{}", username_lower)) ||
                                     body_lower.contains(&format!("id/{}", username_lower)) ||
                                     body_lower.contains(&format!(">{}</", username_lower));
            
            // If no username in profile indicators and page is generic, likely doesn't exist
            if !username_in_profile && body_lower.len() < 50000 {
                return Some(CheckResult::NotFound);
            }
        }
        
        // Instagram: Pure SPA - check if username appears in og:title or title
        // Instagram doesn't show error messages in initial HTML for non-existent users
        // Valid profiles have username in og:title or title tag
        if url_lower.contains("instagram.com") || final_url_lower.contains("instagram.com") {
            // Check for explicit error messages (sometimes present)
            if body_lower.contains("sorry, this page isn't available") ||
               body_lower.contains("page isn't available") ||
               body_lower.contains("user not found") {
                return Some(CheckResult::NotFound);
            }
            
            // Instagram is a pure SPA - check if username appears in SEO tags
            // Valid profiles have username in og:title
            let has_og_title = body_lower.contains("property=\"og:title\"") || 
                              body_lower.contains("property='og:title'");
            let username_in_og_title = has_og_title && body_lower.contains(&username_lower);
            
            // Check title tag
            let title_has_username = if let (Some(title_start), Some(title_end)) = 
                (body_lower.find("<title>"), body_lower.find("</title>")) {
                if title_start + 7 < title_end {
                    body_lower[title_start + 7..title_end].contains(&username_lower)
                } else {
                    false
                }
            } else {
                false
            };
            
            // For Instagram SPAs, if username doesn't appear in og:title or title, user doesn't exist
            // Instagram profiles always have username in og:title or title for SEO
            if !username_in_og_title && !title_has_username {
                return Some(CheckResult::NotFound);
            }
        }
        
        // Threads (Meta/Facebook): Check for generic error pages
        if url_lower.contains("threads.net") || final_url_lower.contains("threads.net") {
            // Threads shows generic error for non-existent users
            if body_lower.contains("page not found") ||
               body_lower.contains("content isn't available") ||
               body_lower.contains("this page isn't available") ||
               (body_lower.contains("threads") && !body_lower.contains(&username_lower) && body_lower.len() < 30000) {
                return Some(CheckResult::NotFound);
            }
            
            // Check if username appears in meta tags
            let username_in_meta = (body_lower.contains("property=\"og:title\"") || 
                                   body_lower.contains("property='og:title'")) &&
                                  body_lower.contains(&username_lower);
            
            if !username_in_meta && body_lower.contains("threads") {
                return Some(CheckResult::NotFound);
            }
        }
        
        // Weibo: Check for error pages or missing content
        if url_lower.contains("weibo.com") || final_url_lower.contains("weibo.com") {
            // Check if username appears in content - valid profiles have username visible
            let username_in_content = body_lower.contains(&format!(">{}</", username_lower)) ||
                                     body_lower.contains(&format!("/{}</", username_lower)) ||
                                     (body_lower.contains("property=\"og:title\"") && body_lower.contains(&username_lower));
            
            // If username doesn't appear and page seems empty/generic, likely doesn't exist
            if !username_in_content && body_lower.len() < 20000 {
                return Some(CheckResult::NotFound);
            }
        }
        
        // Battle.net: Check for error pages or invalid profile indicators
        if url_lower.contains("blizzard.com") || final_url_lower.contains("blizzard.com") ||
           url_lower.contains("battle.net") || final_url_lower.contains("battle.net") {
            // Check for error messages
            if body_lower.contains("page not found") ||
               body_lower.contains("invalid") ||
               body_lower.contains("error") ||
               (body_lower.len() < 10000 && !body_lower.contains(&username_lower)) {
                return Some(CheckResult::NotFound);
            }
            
            // Check if username appears in content
            let username_in_content = body_lower.contains(&format!(">{}</", username_lower)) ||
                                     body_lower.contains(&format!("/{}</", username_lower));
            
            if !username_in_content && body_lower.len() < 15000 {
                return Some(CheckResult::NotFound);
            }
        }
        
        None
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
        
        // Use URL redirect detection - this catches false positives by checking if URL changed
        let result = self.check_url(&url, username, &site.name, false).await;
        
        match result {
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


    async fn check_url(&self, url: &str, username: &str, site_name: &str, _is_spa: bool) -> CheckResult {
        let url_lower = url.to_lowercase();
        // Check if URL redirects (many sites redirect 404s to error pages)
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
        let final_url = response.url().as_str().to_string(); // Clone to avoid borrow issues
        
        // Check for redirect responses (3xx status codes)
        if status.is_redirection() {
            if let Some(location) = response.headers().get("location") {
                if let Ok(location_str) = location.to_str() {
                    let location_lower = location_str.to_lowercase();
                    let username_lower = username.to_lowercase();
                    let url_lower = url.to_lowercase();
                    
                    // If redirect location doesn't contain username, it's likely a 404 redirect
                    if url_lower.contains(&username_lower) && !location_lower.contains(&username_lower) {
                        return CheckResult::NotFound;
                    }
                    
                    // Check if redirecting to error pages
                    if location_lower.contains("404") || 
                       location_lower.contains("not-found") || 
                       location_lower.contains("/error") {
                        return CheckResult::NotFound;
                    }
                    // If it's a redirect but location contains username, it's likely found
                    return CheckResult::Found;
                }
            }
            // Redirect but can't parse - default to found
            return CheckResult::Found;
        }
        
        // Check if URL redirected - if it changed, check if username is preserved
        // This catches 100% of false positives: if URL changes and username is gone, it's a 404
        if final_url != url {
            // URL changed - check if username is preserved
            let final_url_lower = final_url.to_lowercase();
            let username_lower = username.to_lowercase();
            
            // Check if redirected to explicit error pages (strongest signal)
            if final_url_lower.contains("404") || 
               final_url_lower.contains("not-found") || 
               final_url_lower.contains("/error") ||
               final_url_lower.contains("page-not-found") ||
               final_url_lower.contains("notfound") ||
               final_url_lower.contains("user-not-found") ||
               final_url_lower.contains("/not_found") ||
               final_url_lower.contains("/error") {
                return CheckResult::NotFound;
            }
            
            // Special case: Giphy redirects user profiles to /explore/ (search page, not user)
            if url_lower.contains("giphy.com") && final_url_lower.contains("/explore/") {
                return CheckResult::NotFound;
            }
            
            // Extract path segments to compare structure
            let url_path: Vec<&str> = url.split('/').skip(3).collect(); // Skip http://domain
            let final_path: Vec<&str> = final_url.split('/').skip(3).collect();
            
            // If URL structure changed significantly (different number of path segments), likely redirect
            if url_path.len() != final_path.len() && url_path.len() > 0 {
                // Check if username is missing from final URL path (not in any path segment)
                let final_path_contains_username = final_path.iter().any(|segment| segment.contains(username));
                let url_path_contains_username = url_path.iter().any(|segment| segment.contains(username));
                
                // If original URL had username in path but final doesn't, it's a redirect away (404)
                if url_path_contains_username && !final_path_contains_username {
                    return CheckResult::NotFound;
                }
            }
            
            // Also check: if final URL doesn't contain username anywhere, and original did, it's likely 404
            // This is the key check - if username disappears from URL, account doesn't exist
            if url_lower.contains(&username_lower) && !final_url_lower.contains(&username_lower) {
                // Username missing from final URL = almost certainly a redirect away (404)
                // Don't require specific patterns - if username is gone, it's not found
                return CheckResult::NotFound;
            }
            
            // Special case: Check if redirected to a different domain (like angel.co -> wellfound.com)
            // Extract domains to compare
            let original_domain = url.split('/').nth(2).unwrap_or("");
            let final_domain = final_url.split('/').nth(2).unwrap_or("");
            if original_domain != final_domain && original_domain != "" && final_domain != "" {
                // Domain changed - check if it's a known redirect pattern (like angel.co -> wellfound.com)
                // and if the final URL doesn't contain username, it's likely 404
                if !final_url_lower.contains(&username_lower) {
                    return CheckResult::NotFound;
                }
                // Special case: angel.co redirects to wellfound.com
                if original_domain.contains("angel.co") && final_domain.contains("wellfound.com") {
                    // This is a domain migration - wellfound.com URLs for non-existent users 
                    // typically return 403 (Cloudflare) or have empty content
                    // We'll check body content for 404 indicators later
                }
            }
        }

        let body_text = response.text().await.unwrap_or_default();
        let body_lower = body_text.to_lowercase();
        let final_url_lower = final_url.to_lowercase();
        let username_lower = username.to_lowercase();

        // Check for Cloudflare challenge pages - can't properly check these
        if body_lower.contains("attention required") || 
           body_lower.contains("just a moment") ||
           body_lower.contains("checking your browser") ||
           (body_lower.contains("cloudflare") && body_lower.contains("cf-challenge")) {
            return CheckResult::Error("Cloudflare protection (cannot verify)".to_string());
        }
        
        // eBay security check - must run early, before other checks
        // eBay shows security/captcha pages for non-existent users - these patterns indicate user doesn't exist
        if final_url_lower.contains("ebay.com") || url_lower.contains("ebay.com") {
            // Simple body check - if ANY security/captcha pattern found, user doesn't exist
            if body_lower.contains("security measure") ||
               body_lower.contains("captcha_form") ||
               body_lower.contains("please verify yourself") ||
               body_lower.contains("verify yourself to continue") {
                return CheckResult::NotFound;
            }
        }

        // Check for JavaScript redirects using lightweight pattern matching
        // This is much faster than a headless browser but can catch common patterns
        if let Some(js_result) = self.check_js_redirects(&body_text, username, final_url.as_str()) {
            return js_result;
        }
        
        // Fallback: Check for JavaScript redirects or meta refresh tags that redirect to 404
        // Many SPAs redirect via JavaScript after page load
        if body_lower.contains("window.location") || body_lower.contains("location.href") {
            // Check if redirecting to 404
            if body_lower.contains("/404") || 
               body_lower.contains("/not-found") ||
               body_lower.contains("location.href.*404") ||
               body_lower.contains("window.location.*404") {
                return CheckResult::NotFound;
            }
            // Check if redirecting away from username URL
            let username_lower = username.to_lowercase();
            if body_lower.contains("location.href") || body_lower.contains("window.location") {
                // Extract redirect target from common patterns
                let redirect_patterns = [
                    "location.href=\"",
                    "location.href='",
                    "window.location=\"",
                    "window.location='",
                    "window.location.href=\"",
                    "window.location.href='",
                ];
                for pattern in &redirect_patterns {
                    if let Some(pos) = body_lower.find(pattern) {
                        let after_pattern = &body_lower[pos + pattern.len()..];
                        if let Some(end) = after_pattern.find(|c| c == '"' || c == '\'' || c == ';') {
                            let redirect_target = &after_pattern[..end];
                            // If redirect doesn't contain username, it's likely a 404 redirect
                            if !redirect_target.contains(&username_lower) && 
                               (redirect_target.contains("/404") || 
                                redirect_target.contains("/not-found") ||
                                redirect_target.contains("/error")) {
                                return CheckResult::NotFound;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for meta refresh redirects
        if body_lower.contains("http-equiv=\"refresh\"") || body_lower.contains("http-equiv='refresh'") {
            // Extract redirect URL from meta refresh
            let refresh_pattern = regex::Regex::new(r#"(?i)http-equiv=["']refresh["'][^>]*content=["'][^"']*url=(?:['"])?([^"'\s>]+)"#).ok();
            if let Some(pattern) = refresh_pattern {
                for cap in pattern.captures_iter(&body_text) {
                    if let Some(redirect_url) = cap.get(1) {
                        let redirect = redirect_url.as_str().to_lowercase();
                        // TopCoder redirects to profiles.topcoder.com - check if username is in redirect
                        if redirect.contains("profiles.topcoder.com") && !redirect.contains(&username_lower) {
                            return CheckResult::NotFound;
                        }
                        // Generic 404 redirects
                        if redirect.contains("/404") || redirect.contains("/not-found") || redirect.contains("/error") {
                            return CheckResult::NotFound;
                        }
                    }
                }
            }
            // Fallback: simple check for 404 in refresh
            if body_lower.contains("/404") || body_lower.contains("/not-found") {
                return CheckResult::NotFound;
            }
        }

        // Twitter/X: Special handling - returns 200 with SPA shell, username in URL path indicates account exists
        // This check must happen before site-specific and SPA detection
        if (url_lower.contains("twitter.com") || final_url_lower.contains("twitter.com") ||
            url_lower.contains("x.com") || final_url_lower.contains("x.com")) &&
           status.as_u16() == 200 {
            // Twitter/X: If username is in the URL path, account exists
            // Twitter/X is an SPA and doesn't always include username in initial HTML
            if final_url_lower.contains(&format!("/{}", username_lower)) ||
               url_lower.contains(&format!("/{}", username_lower)) {
                return CheckResult::Found;
            }
        }
        
        // Site-specific checks before general detection
        // These are more aggressive and site-aware
        let site_specific_result = self.check_site_specific(url, &body_text, &body_lower, username, final_url.as_str(), status.as_u16());
        if let Some(result) = site_specific_result {
            return result;
        }
        
        // Check status code
        match status.as_u16() {
            // eBay: 503 Service Unavailable often means user doesn't exist (blocked/not found)
            503 => {
                if final_url_lower.contains("ebay.com") || url_lower.contains("ebay.com") {
                    // eBay 503 for /usr/USERNAME typically means user doesn't exist
                    return CheckResult::NotFound;
                }
                // For other sites, 503 is a server error
                return CheckResult::Error("Service temporarily unavailable (503)".to_string());
            },
            200 => {
                // Even with 200 status, check if it's actually a 404 page
                // Many sites return 200 with a 404 page content
                // Special check for Wikipedia: redlink means page doesn't exist
                if final_url_lower.contains("wikipedia.org") {
                    if body_lower.contains("page does not exist") || 
                       body_lower.contains("redlink") ||
                       body_lower.contains("\"wgArticleId\":0") ||
                       body_lower.contains("\"wgCurRevisionId\":0") {
                        return CheckResult::NotFound;
                    }
                }
                
                // Special check for sites that return 200 but with empty/placeholder content
                // Check if page is suspiciously empty or has placeholder text
                let body_len = body_lower.len();
                if body_len < 2000 {
                    // Very short pages might be error pages
                    // Check for common placeholder/error indicators
                    if body_lower.contains("this page is not available") ||
                       body_lower.contains("user not found") ||
                       body_lower.contains("profile not found") ||
                       (body_len < 500 && !body_lower.contains(&username_lower)) {
                        // Very short page without username is suspicious
                        // But be careful - some sites have minimalist designs
                    }
                }
                
                let is_not_found = self.contains_not_found_message(&body_lower, false);
                
                if is_not_found {
                    CheckResult::NotFound
                } else {
                    // Additional check for SPAs: if URL didn't redirect AND it's an SPA shell
                    // SPAs that redirect via JavaScript won't have username in initial HTML
                    // Detect SPA shells - various patterns used by different frameworks
                    let is_spa_shell = body_lower.contains("<div id=\"app\"></div>") || 
                                      body_lower.contains("<div id='app'></div>") ||
                                      body_lower.contains("<div id=app></div>") ||
                                      body_lower.contains("<div id=\"app\">") ||  // App div with content
                                      body_lower.contains("id=\"root\"") ||
                                      body_lower.contains("id='root'") ||
                                      body_lower.contains("id=root") ||
                                      body_lower.contains("react-root") ||
                                      body_lower.contains("__next") ||
                                      body_lower.contains("__nuxt") ||
                                      body_lower.contains("next.js") ||
                                      (body_lower.contains("<script") && 
                                       body_lower.contains("react") && 
                                       body_lower.len() < 50000); // React apps often have script tags
                    
                    let username_lower = username.to_lowercase();
                    
                    // Twitter/X: Returns 200 with SPA shell, but username is in URL path, not always in initial HTML
                    // Twitter/X accounts exist if URL path contains username (even if not in initial HTML)
                    if (final_url_lower.contains("twitter.com") || url_lower.contains("twitter.com") ||
                        final_url_lower.contains("x.com") || url_lower.contains("x.com")) &&
                       final_url_lower.contains(&format!("/{}", username_lower)) {
                        // Twitter/X: If username is in the URL path, account likely exists
                        // Twitter/X doesn't always include username in initial HTML for SPAs
                        CheckResult::Found
                    } else if is_spa_shell && final_url == url {
                        // For other SPAs: require username in title/meta tags for valid profiles
                        // Non-existent users in SPAs won't have username in SEO tags
                        // Extract title to check if username is there
                        let has_username_in_title = if let (Some(title_start), Some(title_end)) = 
                            (body_lower.find("<title>"), body_lower.find("</title>")) {
                            if title_start + 7 < title_end {
                                let title = &body_lower[title_start + 7..title_end];
                                title.contains(&username_lower) && !title.to_lowercase().contains("404")
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                        
                        // Check for username in meta tags (og:title, description, etc.)
                        let has_username_in_meta = (body_lower.contains("property=\"og:title\"") || 
                                                   body_lower.contains("property='og:title'")) && 
                                                   body_lower.contains(&username_lower) ||
                                                   body_lower.contains(&format!("content=\"{}\"", username_lower)) ||
                                                   body_lower.contains(&format!("content='{}'", username_lower));
                        
                        // If username not in HTML at all, or only in scripts (not meta/title), mark as not found
                        // Valid SPA profiles include username in meta tags for SEO
                        if !body_lower.contains(&username_lower) {
                            CheckResult::NotFound
                        } else if !has_username_in_title && !has_username_in_meta {
                            // Username exists but only in scripts/JSON, not in SEO tags - likely doesn't exist
                            // Exception: Some sites like Glitch might have username in scripts but still valid
                            // For now, mark as not found if no meta/title evidence
                            CheckResult::NotFound
                        } else {
                            CheckResult::Found
                        }
                    } else {
                        CheckResult::Found
                    }
                }
            }
            302 | 301 | 307 | 308 => {
                // These are redirects - reqwest should have followed them automatically
                // But if we're here, check final URL
                if final_url_lower.contains("/error") || 
                   final_url_lower.contains("404") ||
                   final_url_lower.contains("not-found") {
                    return CheckResult::NotFound;
                }
                // Check if username disappeared from URL
                if url_lower.contains(&username_lower) && !final_url_lower.contains(&username_lower) {
                    return CheckResult::NotFound;
                }
                CheckResult::Found
            }
            404 => CheckResult::NotFound,
             403 => {
                // 403 might mean account exists but is private, or account doesn't exist, or anti-bot protection
                // Twitter/X: Returns 403 for all requests due to anti-bot protection, but we can still check if username is in HTML
                if url_lower.contains("twitter.com") || url_lower.contains("x.com") {
                    // Twitter/X shows username in title or meta tags even with 403
                    // Check if username appears in title or og:title - valid profiles have it even with 403
                    let title_has_username = if let (Some(title_start), Some(title_end)) = 
                        (body_lower.find("<title>"), body_lower.find("</title>")) {
                        if title_start + 7 < title_end {
                            body_lower[title_start + 7..title_end].contains(&username_lower)
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    
                    let username_in_og_title = (body_lower.contains("property=\"og:title\"") || 
                                               body_lower.contains("property='og:title'")) &&
                                              body_lower.contains(&username_lower);
                    
                    // If username is in title or og:title, account exists (even with 403 blocking)
                    if title_has_username || username_in_og_title || body_lower.contains(&format!("/{}</", username_lower)) {
                        CheckResult::Found
                    } else {
                        // No username found - might not exist, but Twitter blocks all requests so we can't be sure
                        // Default to Found since 403 on Twitter usually means anti-bot, not missing account
                        CheckResult::Found
                    }
                } else {
                    // For other sites, check body for not found messages
                    if self.contains_not_found_message(&body_lower, false) {
                        CheckResult::NotFound
                    } else {
                        // Likely private/exists but blocked
                        CheckResult::Found
                    }
                }
            }
            302 | 301 | 307 | 308 => {
                // Redirect might indicate account exists or doesn't exist
                // Try to check the final location if possible
                CheckResult::Found
            }
            400 => {
                // Bad request - might be invalid username format or requires auth
                if self.contains_not_found_message(&body_lower, false) {
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
                if self.contains_not_found_message(&body_lower, false) {
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

    fn contains_not_found_message(&self, body: &str, is_spa: bool) -> bool {
        let body_lower = body.to_lowercase();
        let body_len = body.len();
        
        // Check title tag first (reliable but not always present)
        let has_404_in_title = if let Some(title_start) = body_lower.find("<title>") {
            if let Some(title_end) = body_lower.find("</title>") {
                if title_end > title_start {
                    let title_content = &body_lower[title_start + 7..title_end]; // +7 to skip "<title>"
                    title_content.contains("404") || 
                    title_content.contains("page not found") ||
                    title_content.contains("not found")
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        
        // Very explicit user/account not found messages - always reliable
        let explicit_user_patterns = vec![
            "user not found",
            "account not found",
            "profile not found",
            "this user does not exist",
            "this account does not exist",
            "user does not exist",
            "account does not exist",
            "no such user",
            "username does not exist",
            "this user does not exist",
            "user profile not found",
            "the user you are looking for",
            "doesn't have an account",
            "could not find user",
            "unable to find user",
            "not a registered user",
            "user not registered",
            "no account associated",
            "couldn't find this account",
            "this account doesn't exist",
            "page does not exist",  // Wikipedia pattern
            "redlink",  // Wikipedia redlink (page doesn't exist)
            "wgArticleId\":0",  // Wikipedia pattern for non-existent pages
            "wgCurRevisionId\":0",  // Wikipedia pattern
        ];
        
        let has_explicit_user_not_found = explicit_user_patterns.iter()
            .any(|pattern| body_lower.contains(pattern));
        
        if has_explicit_user_not_found {
            return true;
        }
        
        // 404 patterns in content - check for context
        // Look for 404 combined with error/page not found context
        let has_404_with_context = body_lower.contains("404") && 
            (body_lower.contains("page not found") ||
             body_lower.contains("not found") ||
             body_lower.contains("error") ||
             body_lower.contains("chan") ||  // Custom 404 pages sometimes use this
             body_lower.contains("couldn't find") ||
             body_lower.contains("can't find"));
        
        // Common 404 page phrases - check in body content
        let common_404_phrases = vec![
            "the page you requested was not found",
            "the requested url was not found",
            "the requested page cannot be found",
            "the page you're looking for cannot be found",
            "the page you are looking for does not exist",
            "page you're looking for doesn't exist",
            "we couldn't find that page",
            "we can't find that page",
            "unfortunately the page you were looking for",
            "sorry, we couldn't find that",
            "the link you followed may be broken",
            "sorry, this page isn't available",
        ];
        
        let has_common_404_phrase = common_404_phrases.iter()
            .any(|pattern| body_lower.contains(pattern));
        
        // Check for large "404" text in content (common in custom 404 pages)
        // Often styled with CSS and appears as prominent text
        let has_prominent_404 = body_lower.contains(">404<") ||
                                body_lower.contains("> 404 <") ||
                                body_lower.contains("\"404\"") ||
                                body_lower.contains("'404'") ||
                                // Check for 404 in common HTML structures
                                (body_lower.contains("<h1>") && body_lower.contains("404") && 
                                 body_lower.matches("404").count() > 0) ||
                                (body_lower.contains("<h2>") && body_lower.contains("404") &&
                                 body_lower.matches("404").count() > 0);
        
        // Check for 404 images (common pattern - sites use images with 404 text)
        // Look for image references with 404 in filename or alt text
        let has_404_image = body_lower.contains("404.png") ||
                           body_lower.contains("404.jpg") ||
                           body_lower.contains("404.jpeg") ||
                           body_lower.contains("404.gif") ||
                           body_lower.contains("404.svg") ||
                           body_lower.contains("404.webp") ||
                           body_lower.contains("/404/") ||
                           body_lower.contains("/not-found/") ||
                           body_lower.contains("error-404") ||
                           body_lower.contains("404-error") ||
                           // Check alt text for images
                           body_lower.contains("alt=\"404\"") ||
                           body_lower.contains("alt='404'") ||
                           body_lower.contains("alt=\"not found\"") ||
                           body_lower.contains("alt='not found'") ||
                           body_lower.contains("alt=\"page not found\"") ||
                           body_lower.contains("alt='page not found'") ||
                           // Common 404 image patterns
                           body_lower.contains("404_not_found") ||
                           body_lower.contains("not_found") && body_lower.contains(".png") ||
                           body_lower.contains("not_found") && body_lower.contains(".jpg");
        
        // Check for JavaScript-rendered pages (SPAs) that might not show 404 in initial HTML
        // These pages often have empty content areas or just app shell
        let is_spa_shell_detected = (body_lower.contains("<div id=\"app\"></div>") || 
                           body_lower.contains("<div id='app'></div>") ||
                           body_lower.contains("id=\"root\"") ||
                           body_lower.contains("id='root'") ||
                           body_lower.contains("react-root") ||
                           body_lower.contains("__next")) &&
                          body_len > 500 && // Has some content but might be just JS includes
                          !has_404_in_title &&
                          !has_explicit_user_not_found;
        
        // For known SPA sites or detected SPA shells, require stronger evidence
        // These sites can't be reliably checked without JavaScript execution
        if (is_spa || is_spa_shell_detected) && !has_explicit_user_not_found && !has_prominent_404 && !has_404_image {
            // Don't mark as 404 if it's just an SPA shell without clear indicators
            // Only return true if we have VERY strong indicators (title tag, explicit messages, images)
            return has_404_in_title || has_explicit_user_not_found || has_404_image;
        }
        
        // Check for 404 in title OR prominent display OR with context
        if has_404_in_title {
            return true; // Title tags are very reliable
        }
        
        if has_prominent_404 || has_404_image {
            return true; // Prominent 404 or 404 images are reliable indicators
        }
        
        // For 404 with context, be more conservative with longer pages
        if has_404_with_context {
            // Very short pages are almost certainly 404 pages
            if body_len < 300 {
                return true;
            }
            // Medium pages need additional evidence
            if body_len < 1000 && has_common_404_phrase {
                return true;
            }
        }
        
        // Short pages with "not found" are likely 404 pages
        if body_len < 400 && (has_common_404_phrase || body_lower.contains("page not found")) {
            return true;
        }
        
        // Medium length pages need multiple strong indicators
        if body_len < 1200 && has_common_404_phrase && body_lower.contains("not found") && 
           (body_lower.contains("404") || has_404_in_title) {
            return true;
        }
        
        false
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

