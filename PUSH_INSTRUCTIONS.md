# Quick Push Instructions

## GitHub Push (using GitHub CLI)

1. **Update placeholders** in these files:
   - `Cargo.toml` - Replace `YOUR_USERNAME` with your GitHub username
   - `README.md` - Replace `YOUR_USERNAME` with your GitHub username  
   - `PKGBUILD` - Replace `YOUR_USERNAME` with your GitHub username

2. **Configure git** (if not already done):
   ```bash
   git config user.name "Your Name"
   git config user.email "your.email@example.com"
   ```

3. **Create initial commit**:
   ```bash
   cd /home/r3dg0d/Documents/watson
   git add -A
   git commit -m "Initial commit: Vidocq OSINT tool v0.1.0"
   ```

4. **Create repository and push using GitHub CLI**:
   ```bash
   # Create public repository and push
   gh repo create vidocq --public --source=. --remote=origin --push --description "Advanced OSINT tool for username searching across 100+ platforms - Lightning fast Rust alternative to Sherlock"
   
   # Or for private repository
   # gh repo create vidocq --private --source=. --remote=origin --push --description "..."
   ```

## AUR Push

1. **Create AUR account** (if you don't have one):
   - Go to https://aur.archlinux.org/register
   - Complete registration and add SSH key

2. **Clone AUR repository**:
   ```bash
   git clone ssh://aur@aur.archlinux.org/vidocq-bin.git /tmp/vidocq-aur
   cd /tmp/vidocq-aur
   ```

3. **Copy and update files**:
   ```bash
   cp /home/r3dg0d/Documents/watson/PKGBUILD .
   cp /home/r3dg0d/Documents/watson/.SRCINFO .
   # Edit PKGBUILD to replace YOUR_USERNAME with your GitHub username
   ```

4. **Regenerate .SRCINFO**:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

5. **Test and push**:
   ```bash
   makepkg
   git add PKGBUILD .SRCINFO
   git commit -m "Initial package release"
   git push
   ```

## After Publishing

Once published, users can install with:
```bash
yay -S vidocq-bin
# or
paru -S vidocq-bin
```

