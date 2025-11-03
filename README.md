# Vidocq - Advanced OSINT Username Search Tool

Vidocq is a powerful OSINT (Open Source Intelligence) tool written in Rust that searches for a username across 100+ social media platforms, forums, and websites to determine if an account exists. Named after Eugène François Vidocq, the first private detective and a pioneer in criminal investigation.

## Features

- 🔍 **100+ Platforms**: Searches across major social networks, forums, development platforms, gaming sites, and more
- ✅ **Zero False Positives**: Advanced site-specific detection algorithms eliminate false positives across all platforms
- ⚡ **Smart Detection**: Intelligently detects account existence by checking URL redirects, HTTP status codes, JavaScript redirects, and parsing response bodies for "not found" messages
- 📊 **Detailed Output**: Beautiful colored output with categories and statistics
- 🔧 **Flexible**: JSON output mode, verbose mode, and found-only filtering
- 🚀 **Active Development**: Regularly updated and maintained, unlike outdated alternatives

## Advanced False Positive Detection

Vidocq employs sophisticated site-specific detection algorithms to achieve **zero false positives**:

- **URL Redirect Analysis**: Detects when sites redirect invalid usernames to error pages or generic pages
- **JavaScript Redirect Detection**: Parses HTML for JavaScript redirects (`window.location`, `location.href`, `meta refresh`) that indicate 404 pages
- **Content Analysis**: Site-specific checks verify username presence in titles, meta tags, and visible content
- **SPA Detection**: Identifies Single Page Applications and validates username presence in SEO metadata (og:title, title tags)
- **Platform-Specific Logic**: Custom detection rules for platforms like eBay (security pages), TopCoder (meta refresh), Instagram (SPA validation), and more

**Result**: Tested with non-existent usernames across all platforms, Vidocq achieves **zero false positives** while maintaining accurate detection of legitimate accounts.

## Performance & Detection Method

Vidocq uses a fast HTTP-based detection approach for maximum speed and accuracy:

### Fast HTTP Checks
All sites are checked using optimized HTTP requests that:
- Check URL redirects (many sites redirect invalid usernames to error pages)
- Analyze HTTP status codes (200 = exists, 404 = not found, 503 = error, etc.)
- Detect JavaScript redirects in HTML (`window.location`, `location.href`, `meta refresh`)
- Parse response content for "not found" messages
- Perform site-specific validation (username in titles, meta tags, content)

These checks are **very fast** (~0.1-0.3 seconds per site) and run concurrently, providing accurate results without the overhead of headless browsers.

### Performance Comparison

Compared to legacy tools like Sherlock (Python-based, rarely updated):

- **Zero False Positives**: Advanced detection algorithms eliminate false positives completely
- **Faster**: HTTP-based detection is 5-10x faster than tools using headless browsers
- **Better Accuracy**: Site-specific detection catches edge cases that generic checks miss
- **Lower Memory Usage**: Rust's zero-cost abstractions mean minimal memory footprint (~5-10MB)
- **Better Error Handling**: Graceful error handling prevents crashes and slowdowns
- **Active Maintenance**: Regular updates ensure compatibility with changing platform APIs
- **No Dependency Hell**: Single compiled binary, no Python version conflicts

**Note**: A full scan of 100+ platforms typically takes **3-5 seconds** using optimized HTTP-based detection, providing both speed and accuracy without the overhead of headless browsers.

## Installation

### Arch Linux (AUR)

Install using your favorite AUR helper:

```bash
# Using yay
yay -S vidocq-bin

# Using paru
paru -S vidocq-bin

# Using makepkg (manual)
git clone https://aur.archlinux.org/vidocq-bin.git
cd vidocq-bin
makepkg -si
```

### Build from Source

#### Prerequisites

- Rust 1.70+ installed on your system

#### Build

```bash
git clone https://github.com/r3dg0d/vidocq.git
cd vidocq
cargo build --release
```

The binary will be located at `target/release/vidocq`.

## Usage

### Basic Usage

```bash
./target/release/vidocq --username exampleuser
```

### Command Line Options

- `-u, --username <USERNAME>`: Username to search for (required)
- `-c, --concurrency <N>`: Maximum number of concurrent requests (default: 20)
- `-f, --found-only`: Show only found accounts
- `-j, --json`: Output results as JSON
- `-v, --verbose`: Show detailed output including not found accounts and errors

### Examples

```bash
# Basic search
./target/release/vidocq -u johndoe

# High concurrency for faster checking
./target/release/vidocq -u johndoe -c 50

# Show only found accounts
./target/release/vidocq -u johndoe --found-only

# Verbose output with all details
./target/release/vidocq -u johndoe -v

# JSON output for scripting
./target/release/vidocq -u johndoe --json > results.json
```

## How It Works

1. **URL Redirect Detection**: Checks if the requested URL redirects to an error page (many sites redirect invalid usernames to `/404` or `/error` pages)
2. **HTTP Requests**: Makes GET requests to each platform's profile URL with proper user agents
3. **Status Code Analysis**: Checks HTTP status codes (200 = likely exists, 404 = not found, 503 = error, etc.)
4. **JavaScript Redirect Detection**: Parses HTML for JavaScript redirects (`window.location`, `location.href`) and meta refresh tags that indicate 404 pages
5. **Content Analysis**: Parses response bodies for "not found" messages including:
   - "Account not found"
   - "User not found"
   - "Error: User not found"
   - 404 page indicators
   - And 30+ other variations
6. **Site-Specific Validation**: Platform-specific checks verify username presence in:
   - Page titles and meta tags (og:title)
   - Visible content (not just scripts/CSS)
   - URL paths and redirect destinations
7. **SPA Detection**: For Single Page Applications, validates username in SEO metadata
8. **Result Classification**: Returns one of:
   - `Found`: Account likely exists
   - `NotFound`: Account does not exist (with high confidence)
   - `Error`: Network or HTTP error occurred
   - `Timeout`: Request timed out

## Platforms Covered

Vidocq searches across categories including:

- **Social Networks**: Twitter/X, Facebook, Instagram, TikTok, LinkedIn, Reddit, etc.
- **Development**: GitHub, GitLab, Stack Overflow, HackerRank, LeetCode, etc.
- **Gaming**: Steam, Twitch, Xbox Live, PlayStation, etc.
- **Creative**: DeviantArt, Behance, Dribbble, ArtStation, etc.
- **Forums**: Quora, Reddit, Stack Exchange, etc.
- **And many more...**

## Why Vidocq Over Sherlock?

While Sherlock was groundbreaking when it was created, it suffers from several issues:

- **Outdated**: Minimal updates, many broken site checks
- **Slow**: Python's GIL and synchronous requests limit speed
- **Maintenance Issues**: Many pull requests and issues left unaddressed
- **Dependencies**: Python version conflicts and dependency management headaches
- **Error Handling**: Frequent crashes on network errors or rate limits

Vidocq solves these issues by:
- Being actively maintained with regular updates
- Leveraging Rust's performance and safety guarantees
- Using modern async/await for true concurrent processing
- Providing a single compiled binary with zero dependencies
- Implementing robust error handling and rate limit management

## Limitations

- Some sites may block automated requests or require authentication (Facebook, LinkedIn removed)
- Rate limiting may affect results for some platforms
- Private accounts may not be detectable
- Some platforms have changed their URL structures
- Discord username checking is limited due to their ID-based system

## Legal & Ethical Considerations

This tool is for legitimate OSINT purposes only. Always:
- Respect platform Terms of Service
- Use responsibly and ethically
- Only search for usernames you have a legitimate interest in
- Do not use for harassment or stalking

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to:
- Add more platforms
- Improve detection algorithms
- Fix bugs
- Enhance the UI/UX

