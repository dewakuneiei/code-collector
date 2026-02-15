# 🚀 Rust Code Collector

![Rust](https://img.shields.io/badge/Made_with-Rust-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20MacOS-lightgrey)

**Code Collector** is a high-performance desktop GUI tool designed to help developers extract, filter, and export code from their projects. It is perfect for preparing context for **LLMs (ChatGPT, Claude, DeepSeek)**, creating backups, or reviewing large codebases.

---

## 📥 Download

**Don't want to build from source?** Download the latest executable version here:

[![Download](https://img.shields.io/badge/Download-Windows_x64_.zip-brightgreen?style=for-the-badge&logo=windows)](https://github.com/dewakuneiei/code-collector/releases/latest)

*(Note: Currently built for Windows. Linux/Mac users, please build from source below.)*

---

## ✨ Features

- **⚡ Blazing Fast:** Built with Rust and `egui` for instant startup and low memory usage.
- **🌳 Tree View Navigation:** Explore your project with a familiar file explorer interface.
- **🎨 Themes:** Switch between **Light**, **Dark**, or **System** themes (Apple-style Light mode included).
- **🔍 Smart Search:** Instantly filter files across the entire project structure.
- **🔄 Smart Refresh:** Reload your project folder **without losing your selected files**.
- **🛡️ Integrity Check:** Detects if selected files have been deleted before exporting.
- **📝 Export Modes:**
  - **Single File:** Merges all code into one text file (great for AI context).
  - **Separate Files:** Copies selected files to a new folder while preserving structure.
- **🚫 Auto-Ignore:** Automatically skips clutter like `node_modules`, `.git`, `target`, `vendor`, etc.

---

## 🛠️ Build from Source

If you are a developer, you can clone and build the project yourself.

### Prerequisites
- Install [Rust & Cargo](https://rustup.rs/)

### 1. Clone the Repository
```bash
git clone https://github.com/dewakuneiei/code-collector.git
cd code-collector
