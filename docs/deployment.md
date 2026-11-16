# Deployment Guide

TaskMaster compiles down to a single self-contained binary embedding all frontend assets. You can run it locally, in containers, on Linux VPS instances, or deploy it to cloud platform providers. Configuration templates are available in the [`deploy/`](../deploy/) directory.

---

## 📦 Pre-Compiled Release Binaries

Pre-compiled standalone binaries are built and attached to every [GitHub Release](https://github.com/itsvrushabh/todo/releases):

| Platform | Architecture | Release Archive |
|---|---|---|
| **Linux** | `x86_64` (AMD64) | `taskmaster-linux-amd64.tar.gz` |
| **Linux** | `aarch64` (ARM64) | `taskmaster-linux-arm64.tar.gz` |
| **macOS** | Apple Silicon (`aarch64`) | `taskmaster-macos-arm64.tar.gz` |
| **macOS** | Intel (`x86_64`) | `taskmaster-macos-x86_64.tar.gz` |
| **Windows** | `x86_64` (AMD64) | `taskmaster-windows-amd64.zip` |

Extract the archive and move `todo` (or `todo.exe`) to your `$PATH` (e.g. `/usr/local/bin` on Linux/macOS).

---

## 📦 Package Managers

### 1. Arch Linux (AUR)
Install using your favorite AUR helper:
```bash
# Using yay
yay -S todo-bin

# Using paru
paru -S todo-bin
```

Or clone and build manually from the AUR:
```bash
git clone https://aur.archlinux.org/todo-bin.git
cd todo-bin
makepkg -si
```

### 2. Homebrew (macOS & Linux)
Install via the official tap:
```bash
brew tap itsvrushabh/tap
brew install todo
```

Or install directly from the repository formula:
```bash
brew install https://raw.githubusercontent.com/itsvrushabh/todo/main/Formula/todo.rb
```

---

## 🐳 Docker Deployment

### 1. Docker Compose (Recommended)
Launch the container with persistent storage volume:
```bash
docker compose up -d
```

View logs:
```bash
docker compose logs -f
```

Stop the container:
```bash
docker compose down
```

### 2. Standalone Docker Container
```bash
# Build multi-stage optimized image
docker build -t taskmaster-todo:latest .

# Run with persistent volume mount
docker run -d \
  --name taskmaster-app \
  -p 8000:8000 \
  -v $(pwd)/data:/data \
  -e TODO_STORAGE_PATH=/data/todos.json \
  --restart unless-stopped \
  taskmaster-todo:latest
```

---

## ☁️ Cloud Service Providers

### 1. Fly.io
Deploy with a persistent volume to keep tasks safe across deployments:
```bash
# Launch app using deploy/fly.toml
fly launch --config deploy/fly.toml --no-deploy

# Create persistent storage volume
fly volumes create todo_data --size 1 --region iad

# Deploy
fly deploy --config deploy/fly.toml
```

### 2. Render
1. Create a new **Web Service** on [Render](https://render.com).
2. Connect your GitHub repository.
3. Select **Docker** environment or use [`deploy/render.yaml`](../deploy/render.yaml).
4. Attach a persistent disk mounted to `/data` and set `TODO_STORAGE_PATH=/data/todos.json`.

### 3. Railway / Heroku / Generic PaaS
Deploy using the included [`deploy/Procfile`](../deploy/Procfile):
```
web: todo serve --host 0.0.0.0 --port ${PORT:-8000}
```

---

## 🐧 Linux VPS (Systemd Service)

Deploy directly on Ubuntu, Debian, Arch Linux, or any systemd-based Linux VPS:

```bash
# 1. Build release binary and install to /usr/local/bin
cargo build --release
sudo cp target/release/todo /usr/local/bin/

# 2. Create dedicated data directory
sudo mkdir -p /var/lib/taskmaster
sudo chown -R www-data:www-data /var/lib/taskmaster

# 3. Install and start systemd service
sudo cp deploy/todo.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now todo

# 4. Check service status
sudo systemctl status todo
```
