# Contributing to Marrow

Thank you for your interest in contributing to **Marrow**! We welcome contributions to cryptography primitives, UI optimizations, networking resilience, and documentation.

---

## Development Workflow

### 1. Environment Setup

Make sure you have installed:
* **Rust**: `1.78.0+`
* **Node.js**: `v20+` with `pnpm`
* **System dependencies for Tauri v2**:
  * **Linux**: `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
  * **Windows**: C++ Build Tools & Windows SDK

Enable local repository Git hooks:

```bash
git config core.hooksPath .githooks