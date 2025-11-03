# Installing Vidocq from AUR

This guide explains how to publish and install Vidocq from the Arch User Repository (AUR).

## For Users

### Installation

Once the AUR package is available, install it using:

```bash
# Using yay (recommended)
yay -S vidocq-bin

# Using paru
paru -S vidocq-bin

# Manual installation
git clone https://aur.archlinux.org/vidocq-bin.git
cd vidocq-bin
makepkg -si
```

## For Maintainers

### Initial AUR Package Setup

1. **Prepare the AUR repository**:
   ```bash
   # Clone the AUR repository (create it first on aur.archlinux.org)
   git clone ssh://aur@aur.archlinux.org/vidocq-bin.git
   cd vidocq-bin
   ```

2. **Copy the package files**:
   ```bash
   cp /path/to/vidocq/PKGBUILD .
   cp /path/to/vidocq/.SRCINFO .
   ```

3. **Update PKGBUILD**:
   - Replace `YOUR_USERNAME` with your actual GitHub username
   - Update maintainer information
   - Verify all URLs and paths

4. **Generate .SRCINFO**:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

5. **Test the package**:
   ```bash
   makepkg
   ```

6. **Push to AUR**:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Initial package release"
   git push
   ```

### Updating the Package

1. **Update version in PKGBUILD**:
   ```bash
   vim PKGBUILD  # Update pkgver
   ```

2. **Regenerate .SRCINFO**:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

3. **Test and push**:
   ```bash
   makepkg
   git add PKGBUILD .SRCINFO
   git commit -m "Update to v0.1.1"
   git push
   ```

