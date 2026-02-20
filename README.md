# 🚀 Rust Code Collector

![Rust](https://img.shields.io/badge/Made_with-Rust-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20MacOS-lightgrey)

**Code Collector** is a high-performance desktop GUI tool built with Rust and `egui`. It allows developers to quickly browse, filter, and export source code for use with **LLMs (ChatGPT, Claude, DeepSeek)** or for project documentation.

---

## 📸 Showcase

| 🌙 Dark Theme | ☀️ Light Theme |
| :---: | :---: |
| ![Dark Mode](images/code_collector_dark.png) | ![Light Mode](images/code_collector_light.png) |

---

## ✨ Key Features

* **⚡ Blazing Fast:** Instant startup and ultra-low memory footprint.
* **🌳 Tree View:** Navigate your project with a familiar, interactive file explorer.
* **🔍 Smart Search:** Real-time filtering across your entire directory structure.
* **🎨 Custom Themes:** Toggle between Dark, Light (Apple-style), and System modes.
* **🔄 Smart Refresh:** Update your folder view without losing your current file selections.
* **🛡️ Integrity Check:** Validates that files still exist on disk before performing an export.
* **📝 Export Modes:**
    * **Single File:** Combines all selected code into one `.txt` file (optimized for AI prompts).
    * **Separate Files:** Replicates your project structure in a new destination folder.
* **🚫 Auto-Ignore:** Built-in filters for `node_modules`, `.git`, `target`, and more.

---

## 📥 Download

**Official Releases:** [![Download](https://img.shields.io/badge/Download-Windows_x64_.zip-brightgreen?style=for-the-badge&logo=windows)](https://github.com/dewakuneiei/code-collector/releases/latest)

---

## 🛠️ Build from Source

If you prefer to build the binary yourself, follow these steps:

### 1. Prerequisites
You must have the Rust toolchain installed. If you don't have it, get it at [rustup.rs](https://rustup.rs/).

### 2. Clone and Build
Open your terminal (or CMD on Windows) and run:

```bash
# Clone the repository
git clone https://github.com/dewakuneiei/code-collector.git

# Enter the project directory
cd code-collector

# Build and run the application in release mode
cargo run --release
