mod checker;
mod sites;

use checker::{AccountChecker, SiteResult};
use sites::get_sites;
use clap::Parser;
use colored::*;
use futures::stream::{self, StreamExt};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser, Debug)]
#[command(name = "vidocq")]
#[command(about = "Advanced OSINT tool for username searching across 100+ platforms", long_about = None)]
struct Args {
    /// Username to search for
    #[arg(short, long)]
    username: String,

    /// Maximum number of concurrent requests
    #[arg(short, long, default_value_t = 20)]
    concurrency: usize,

    /// Show only found accounts
    #[arg(short, long)]
    found_only: bool,

    /// Output results as JSON
    #[arg(short, long)]
    json: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let start_time = Instant::now();

    let sites = get_sites();
    let checker = Arc::new(AccountChecker::new());
    let username = args.username.clone();

    println!("{}", format!("Searching for username: {}", username).bright_cyan().bold());
    println!("{}", format!("Checking {} platforms...", sites.len()).bright_white());

    // Create progress bar wrapped in Arc<Mutex> for sharing across async tasks
    let pb = Arc::new(Mutex::new(ProgressBar::new(sites.len() as u64)));
    pb.lock().unwrap().set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Process sites concurrently
    let results: Vec<SiteResult> = stream::iter(sites.iter())
        .map(|site| {
            let checker = Arc::clone(&checker);
            let username = username.clone();
            let site = site.clone();
            let pb = Arc::clone(&pb);
            async move {
                let result = checker.check_account(&site, &username).await;
                pb.lock().unwrap().inc(1);
                result
            }
        })
        .buffer_unordered(args.concurrency)
        .collect()
        .await;

    pb.lock().unwrap().finish_with_message("Complete!");

    // Filter and sort results
    let mut found_results: Vec<&SiteResult> = results
        .iter()
        .filter(|r| matches!(r.result, checker::CheckResult::Found))
        .collect();

    let mut not_found_results: Vec<&SiteResult> = results
        .iter()
        .filter(|r| matches!(r.result, checker::CheckResult::NotFound))
        .collect();

    let error_results: Vec<&SiteResult> = results
        .iter()
        .filter(|r| matches!(r.result, checker::CheckResult::Error(_)))
        .collect();

    found_results.sort_by(|a, b| a.category.cmp(&b.category).then(a.site.cmp(&b.site)));
    not_found_results.sort_by(|a, b| a.category.cmp(&b.category).then(a.site.cmp(&b.site)));

    // Output results
    if args.json {
        output_json(&results);
    } else {
        output_human_readable(&args, &found_results, &not_found_results, &error_results);
    }

    let duration = start_time.elapsed();
    println!("\n{}", format!("Completed in {:.2} seconds", duration.as_secs_f64()).bright_white());
}

fn output_json(results: &[SiteResult]) {
    let json = serde_json::to_string_pretty(results).unwrap();
    println!("{}", json);
}

fn output_human_readable(
    args: &Args,
    found: &[&SiteResult],
    not_found: &[&SiteResult],
    errors: &[&SiteResult],
) {
    if !args.found_only {
        println!("\n{}", "=".repeat(80).bright_white());
    }

    // Display found accounts
    if !found.is_empty() {
        println!("\n{}", format!("✓ FOUND ({})", found.len()).bright_green().bold());
        println!("{}", "=".repeat(80).bright_green());

        let mut current_category = String::new();
        for result in found {
            if result.category != current_category {
                current_category = result.category.clone();
                println!("\n{}", format!("[{}]", current_category).bright_cyan());
            }
            println!("  {} {} - {}", "✓".bright_green(), result.site.bright_white(), result.url.bright_blue().underline());
        }
    } else {
        println!("\n{}", "✗ No accounts found".bright_red().bold());
    }

    if args.found_only {
        return;
    }

    // Display not found accounts (if verbose)
    if args.verbose && !not_found.is_empty() {
        println!("\n{}", format!("✗ NOT FOUND ({})", not_found.len()).bright_yellow().bold());
        println!("{}", "=".repeat(80).bright_yellow());

        let mut current_category = String::new();
        for result in not_found {
            if result.category != current_category {
                current_category = result.category.clone();
                println!("\n{}", format!("[{}]", current_category).bright_cyan());
            }
            println!("  {} {} - {}", "✗".bright_yellow(), result.site.bright_white(), result.url.bright_blue().underline());
        }
    }

    // Display errors (if verbose)
    if args.verbose && !errors.is_empty() {
        println!("\n{}", format!("⚠ ERRORS ({})", errors.len()).bright_red().bold());
        println!("{}", "=".repeat(80).bright_red());

        for result in errors {
            if let checker::CheckResult::Error(e) = &result.result {
                println!("  {} {}: {}", "⚠".bright_red(), result.site.bright_white(), e.bright_black());
            }
        }
    }

    // Summary
    println!("\n{}", "=".repeat(80).bright_white());
    println!("{}", format!("Summary:").bright_white().bold());
    println!("  {}: {}", "Found".bright_green(), found.len().to_string().bright_green().bold());
    println!("  {}: {}", "Not Found".bright_yellow(), not_found.len().to_string().bright_yellow());
    println!("  {}: {}", "Errors".bright_red(), errors.len().to_string().bright_red());
    println!("  {}: {}", "Total".bright_white(), (found.len() + not_found.len() + errors.len()).to_string().bright_white());
}

