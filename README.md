Rust Code Collector

A desktop GUI application built with Rust and egui. It allows you to select specific files from a project directory and export them either into a single text file (for LLM context/documentation) or replicate the folder structure for backup/migration.

Features

Tree View: Recursive file explorer with checkboxes.

Smart Filtering: Ignores node_modules, vendor, .git, etc.

Syntax Highlighting: Visual cues for Rust, JS, HTML, PHP, Laravel Blade, etc.

Export Mode 1 (Single File): Combines all selected code into full_code.txt.

Export Mode 2 (Separate Files): Copies selected files to a new directory while preserving the original folder structure.

Clipboard Support: One-click copy selected code.

Prerequisites

Before running, ensure you have Rust and Cargo installed.
Install Rust

Linux Users Only

You may need to install development libraries for the GUI windowing system:

sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev


How to Run

Clone the repository:

git clone [https://github.com/dewakuneiei/code-collector.git](https://github.com/dewakuneiei/code-collector.git)
cd code-collector


Run in Debug Mode (Faster compilation, slower app):

cargo run


Run in Release Mode (Optimized, smaller binary):
Use this for actual daily use.

cargo run --release


Project Structure

src/main.rs: Contains the entire application logic, UI rendering, and file handling.

Cargo.toml: Manages dependencies (eframe, egui, rfd, arboard).

Usage

Click Open Project and select your coding project folder.

Expand folders and check the boxes for the files you want to include.

Select your Export Mode at the bottom:

Single File: Best for pasting into ChatGPT/Claude.

Separate Files: Best for extracting specific parts of a codebase to a new location.

Click Save Selected or Copy to Clipboard.


---

### 3. Step-by-Step: What to do after Cloning

If you are setting this up for the very first time on a new machine (or if a friend clones your repo):

**Step 1: Install Rust**
If you haven't already:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh


Step 2: Clone the Repo

git clone <your-repo-url>
cd code_collector


Step 3: Check dependencies (Linux only)
If you are on Windows or macOS, you can skip this. If you are on Ubuntu/Debian/WSL, run the command found in the "Prerequisites" section of the README above.

Step 4: Build and Run
This command will automatically download all dependencies listed in Cargo.toml and compile the app:

cargo run --release
