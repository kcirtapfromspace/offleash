# NixOS Development VM for Dog Walker

This directory contains configuration for a NixOS development VM.

## Setup with UTM (macOS)

### 1. Download NixOS ISO

```bash
# For Apple Silicon (ARM64)
curl -LO https://channels.nixos.org/nixos-24.05/latest-nixos-minimal-aarch64-linux.iso

# For Intel (x86_64)
curl -LO https://channels.nixos.org/nixos-24.05/latest-nixos-minimal-x86_64-linux.iso
```

### 2. Create VM in UTM

1. Open UTM and click "Create a New Virtual Machine"
2. Select "Virtualize" (faster) or "Emulate" (if different architecture)
3. Choose "Linux"
4. Select the downloaded NixOS ISO
5. Configure:
   - Memory: 4GB+ recommended
   - CPU: 2+ cores
   - Storage: 20GB+ (use VirtIO for better performance)
6. Enable "Open VM Settings before starting"
7. In Network settings, use "Shared Network" for internet access

### 3. Install NixOS

1. Boot the VM from ISO
2. Once booted, you'll be in a minimal NixOS installer
3. Run:

```bash
# Partition the disk (assuming /dev/vda)
sudo parted /dev/vda -- mklabel gpt
sudo parted /dev/vda -- mkpart ESP fat32 1MB 512MB
sudo parted /dev/vda -- set 1 esp on
sudo parted /dev/vda -- mkpart primary 512MB 100%

# Format
sudo mkfs.fat -F 32 -n boot /dev/vda1
sudo mkfs.ext4 -L nixos /dev/vda2

# Mount
sudo mount /dev/disk/by-label/nixos /mnt
sudo mkdir -p /mnt/boot
sudo mount /dev/disk/by-label/boot /mnt/boot

# Generate hardware config
sudo nixos-generate-config --root /mnt

# Copy our configuration (from host, or paste contents)
# Option 1: If you have shared folder access
# sudo cp /path/to/configuration.nix /mnt/etc/nixos/

# Option 2: Create manually
sudo nano /mnt/etc/nixos/configuration.nix
# Paste contents of configuration.nix

# Install
sudo nixos-install

# Set root password when prompted
# Reboot
sudo reboot
```

### 4. First Boot

After rebooting (remove ISO first):

```bash
# Login as 'dev' with password 'dev'

# Change password
passwd

# Setup Rust
rustup default stable

# Clone the project
git clone <your-repo-url> ~/dog_walker
cd ~/dog_walker

# The devenv will activate automatically via direnv
# Or manually: devenv shell
```

### 5. Verify Setup

```bash
# Check PostgreSQL is running
systemctl status postgresql

# Check database
psql -U dogwalker -d dog_walker -c "SELECT 1"

# Run the API
cd ~/dog_walker
cargo run --bin api
```

## Port Forwarding (UTM)

To access services from macOS host:

1. In UTM, go to VM Settings > Network
2. Add port forwards:
   - Host: 8080 → Guest: 8080 (API)
   - Host: 5433 → Guest: 5432 (PostgreSQL)

Then access from macOS:
- API: http://localhost:8080
- PostgreSQL: localhost:5433

## Shared Folders

For easier file editing from macOS:

1. In UTM, add a shared directory (Settings > Sharing)
2. In the VM, mount it:

```bash
sudo mkdir /mnt/shared
sudo mount -t 9p -o trans=virtio share /mnt/shared
```

Or use VSCode Remote SSH extension to edit files directly.

## Tips

- Use `nixos-rebuild switch` to apply configuration changes
- Use `nix-collect-garbage -d` to free disk space
- Use `home-manager` for user-specific configuration
