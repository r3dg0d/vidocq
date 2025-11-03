# GitHub Setup Instructions (using GitHub CLI)

## Initial Setup and Push

### 1. Update Repository URLs

Before pushing, update these files with your actual GitHub username:

1. **Cargo.toml**: Replace `YOUR_USERNAME` with your GitHub username
2. **README.md**: Replace `YOUR_USERNAME` with your GitHub username in the git clone URLs
3. **PKGBUILD**: Replace `YOUR_USERNAME` with your GitHub username

### 2. Configure Git (if needed)

```bash
git config user.name "Your Name"
git config user.email "your.email@example.com"
```

### 3. Create Initial Commit

```bash
cd /home/r3dg0d/Documents/watson
git add -A
git commit -m "Initial commit: Vidocq OSINT tool v0.1.0

- Advanced OSINT username search across 100+ platforms
- Lightning fast Rust implementation with async/await
- Concurrent checking with configurable concurrency
- Smart detection with HTTP status and content analysis
- Beautiful colored output with JSON export
- Special Discord username checking
- AUR package support
- GitHub Actions CI/CD"
```

### 4. Create Repository and Push (using GitHub CLI)

```bash
# Create public repository and push
gh repo create vidocq --public --source=. --remote=origin --push \
  --description "Advanced OSINT tool for username searching across 100+ platforms - Lightning fast Rust alternative to Sherlock"

# Or for private repository
# gh repo create vidocq --private --source=. --remote=origin --push \
#   --description "Advanced OSINT tool for username searching across 100+ platforms - Lightning fast Rust alternative to Sherlock"
```

**Note**: Make sure you're authenticated with GitHub CLI (`gh auth login` if needed).

### 5. Update Repository Settings

After pushing:

1. Go to your repository settings on GitHub
2. Enable GitHub Actions (Settings → Actions → General)
3. Set up branch protection if desired
4. Add topics: `osint`, `rust`, `username-search`, `security`, `investigation`

## Updating the Repository

For future updates:

```bash
git add -A
git commit -m "Description of changes"
git push
```

## Creating Releases

1. Go to Releases → Draft a new release
2. Tag version: `v0.1.0`
3. Release title: `Vidocq v0.1.0`
4. Description: Use the changelog
5. Publish release

