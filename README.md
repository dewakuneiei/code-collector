```md
# 🚀 Rust Code Collector

A powerful desktop GUI tool built with **Rust** and **egui**, designed for developers who need quick code extraction, structured exporting, and clean project navigation. Perfect for preparing context for LLMs, creating backups, migrating code, or extracting modules from large codebases.

---

## ✨ Features

### 📂 Recursive Tree View
- Explore your project using a nested folder UI.
- Folders and files are displayed in a familiar tree structure.

### 🔍 Real-Time Search
- Filter files and directories instantly.
- Automatically expands folders to reveal matching items.

### 🧹 Smart Filtering
Ignores common clutter directories automatically:
```

.git, node_modules, vendor, target, dist, build, storage, **pycache**, .idea, .vscode, etc.

````

### 🎨 Syntax Highlighting (UI)
Colored file types for better readability:
- Rust (.rs)
- JavaScript / TypeScript
- HTML / CSS
- PHP
- Laravel Blade
- And more

---

## 📦 Export Modes

### **Mode 1 — Single File Export**
- Merges all selected files into `full_code.txt`.
- Ideal for ChatGPT, Claude, and documentation.
- ✔ Auto-opens the generated file in your default editor.

### **Mode 2 — Structured Folder Export**
- Copies selected files into a new directory.
- Preserves original folder structure (e.g., `src/main.rs` → `src/main.rs`).
- ✔ Auto-opens the output folder in your OS file explorer.

### 📋 Clipboard Support
- One-click **Copy Selected** → immediately placed in your system clipboard.

---

## 🛠 Prerequisites

Ensure Rust & Cargo are installed:

```bash
curl https://sh.rustup.rs -sSf | sh
````

### Linux Users Only

You may need additional GUI libraries:

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

---

## 🏃 How to Run

Clone the repository:

```bash
git clone https://github.com/yourusername/code-collector.git
cd code-collector
```

Run the application (Release Mode recommended):

```bash
cargo run --release
```

---

## 📁 Project Structure

```
src/
 └── main.rs      # Core application logic (UI, file handling, search, export)
Cargo.toml        # Dependencies: eframe, egui, rfd, arboard, open
```

---

## 🎮 Usage Guide

### 1. Open Your Project

Click **Open Folder** and select the root directory of your codebase.

### 2. Search & Select

* Use the search bar to filter files (e.g., "service", "controller", "auth").
* Check the boxes for everything you want to export.

### 3. Choose Export Mode

* **Single File** → for AI context or documentation.
* **Separate Files** → for refactoring or migration.

### 4. Perform Action

* **Save Selected** → export to file or folder.
* **Copy to Clipboard** → quick code extraction.

---

## 🔧 Configuration

To modify ignored directories, edit `src/main.rs`:

```rust
const IGNORE_DIRS: &[&str] = &[
    ".git", ".vscode", "node_modules", "vendor", "__pycache__",
    ".idea", "target", "dist", "build", "coverage", ".next",
    ".nuxt", "storage"
];
```

---

## 📜 License

MIT — You are free to use, modify, and distribute.

---

## ⭐ Support the Project

If this tool helps you, please consider giving the repository a **star** ⭐ on GitHub!

```

---

If you want, I can also generate:

✅ GitHub badges  
✅ Screenshots section layout  
✅ Logo banner  
✅ Better project name suggestions  

Just tell me!
```
