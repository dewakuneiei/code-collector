# 📂 AI Code Collector

**AI Code Collector** is a simple Python GUI tool designed to help developers quickly gather code from a web project (HTML, CSS, JS) and merge it into a single text format. 

This is perfect for preparing context to send to AI tools like **ChatGPT, Claude, or Gemini** without manually copying and pasting dozens of files.

## ✨ Features

*   **GUI Interface:** No complex command lines. Visual selection of files.
*   **Smart Categorization:** Automatically groups files into HTML, CSS, and JavaScript.
*   **Clipboard Support:** Copy all selected code to your clipboard with one click.
*   **File Exclusion:** Automatically ignores `.git`, `node_modules`, and system files.
*   **Formatted Output:** Adds clear headers (`===== FILE: path/to/file =====`) so the AI understands the file structure.

## 🚀 Prerequisites

*   **Python 3.x** installed on your system.
*   (Optional) `pyperclip` for better clipboard support.

## 🛠️ Installation

1.  Clone this repository or download the script.
    ```bash
    git clone https://github.com/your-username/ai-code-collector.git
    cd ai-code-collector
    ```

2.  (Optional) Install the clipboard library for the "Copy" button to work perfectly:
    ```bash
    pip install pyperclip
    ```
    *(Note: If you don't install this, the tool will try to use the default Windows clipboard, which usually works fine).*

## 📖 How to Use

1.  **Place the script:** Copy the `collect_gui.py` file into the **root folder** of your web project (the folder where your `index.html` is).
2.  **Run the script:**
    ```bash
    python collect_gui.py
    ```
3.  **Select Files:**
    *   A window will pop up showing all your project files.
    *   Uncheck any files you don't want to include (like large libraries or unfinished tests).
4.  **Generate:**
    *   Click **"📋 Copy to Clipboard"** to paste directly into ChatGPT.
    *   Click **"💾 Save as .txt"** to create a file named `full_code.txt`.

## 📝 Output Format Example

When you paste the result to an AI, it will look like this:

```text
==================================================
FILE: index.html
==================================================
<!DOCTYPE html>
<html>
... code ...
</html>

==================================================
FILE: css/style.css
==================================================
body {
    background-color: #fff;
}
