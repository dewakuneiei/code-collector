import os
import tkinter as tk
from tkinter import messagebox, ttk
import pyperclip  # ต้องลง pip install pyperclip ก่อน (ถ้าไม่มีจะใช้ clipboard ของ windows แทน)

# --- CONFIGURATION ---
TARGETS = {
    'HTML': {'ext': ['.html'], 'color': '#e34c26', 'bg': '#ffebe6'},
    'CSS': {'ext': ['.css'], 'color': '#264de4', 'bg': '#e6f0ff'},
    'JavaScript': {'ext': ['.js'], 'color': '#f0db4f', 'bg': '#fffae6'}
}
IGNORE_DIRS = ['.git', '.vscode', 'node_modules', '__pycache__', '.idea']
OUTPUT_FILENAME = 'full_code.txt'

class ModernCodeCollector:
    def __init__(self, root):
        self.root = root
        self.root.title("Project Code Collector (AI Helper)")
        self.root.geometry("700x800")
        
        # Style Configuration
        style = ttk.Style()
        style.theme_use('clam')
        style.configure("TButton", padding=6, relief="flat", background="#ccc")
        
        self.project_path = os.getcwd()
        self.file_vars = {} 
        self.category_vars = {} # Control 'Select All' per category

        # --- 1. HEADER ---
        header_frame = tk.Frame(root, bg="#333", pady=15)
        header_frame.pack(fill="x")
        
        lbl_title = tk.Label(header_frame, text=f"📂 Project: {os.path.basename(self.project_path)}", 
                             font=("Segoe UI", 14, "bold"), bg="#333", fg="white")
        lbl_title.pack()
        
        self.lbl_status = tk.Label(header_frame, text="Scanning...", font=("Segoe UI", 10), bg="#333", fg="#aaa")
        self.lbl_status.pack()

        # --- 2. FILE LIST (Scrollable) ---
        container = tk.Frame(root)
        container.pack(fill="both", expand=True, padx=10, pady=10)

        self.canvas = tk.Canvas(container, bg="white", highlightthickness=0)
        scrollbar = ttk.Scrollbar(container, orient="vertical", command=self.canvas.yview)
        
        self.scrollable_frame = tk.Frame(self.canvas, bg="white")
        
        self.scrollable_frame.bind(
            "<Configure>", lambda e: self.canvas.configure(scrollregion=self.canvas.bbox("all"))
        )

        self.canvas.create_window((0, 0), window=self.scrollable_frame, anchor="nw", width=660) # Set width to avoid horizontal scroll
        self.canvas.configure(yscrollcommand=scrollbar.set)

        self.canvas.pack(side="left", fill="both", expand=True)
        scrollbar.pack(side="right", fill="y")

        # --- 3. BOTTOM ACTIONS ---
        bottom_frame = tk.Frame(root, pady=15, bg="#f0f0f0")
        bottom_frame.pack(fill="x")

        # Copy to Clipboard Button
        btn_copy = tk.Button(bottom_frame, text="📋 Copy to Clipboard", 
                             bg="#ff9800", fg="white", font=("Segoe UI", 11, "bold"),
                             relief="flat", padx=20, pady=10, command=self.copy_to_clipboard)
        btn_copy.pack(side="left", padx=20, expand=True, fill="x")

        # Save File Button
        btn_save = tk.Button(bottom_frame, text=f"💾 Save as {OUTPUT_FILENAME}", 
                             bg="#4caf50", fg="white", font=("Segoe UI", 11, "bold"),
                             relief="flat", padx=20, pady=10, command=self.generate_file)
        btn_save.pack(side="right", padx=20, expand=True, fill="x")

        # --- LOGIC ---
        self.scan_and_build_ui()
        self.update_counter()

    def scan_and_build_ui(self):
        # 1. Collect Files
        categorized_files = {k: [] for k in TARGETS.keys()}
        total_files = 0

        for root, dirs, files in os.walk(self.project_path):
            dirs[:] = [d for d in dirs if d not in IGNORE_DIRS]
            
            for file in files:
                ext = os.path.splitext(file)[1]
                
                # Check which category this file belongs to
                for cat, config in TARGETS.items():
                    if ext in config['ext']:
                        if file == OUTPUT_FILENAME or file.endswith('.py'): continue
                        
                        full_path = os.path.join(root, file)
                        rel_path = os.path.relpath(full_path, self.project_path)
                        categorized_files[cat].append(rel_path)
                        total_files += 1
                        break

        if total_files == 0:
            tk.Label(self.scrollable_frame, text="No code files found!", fg="red", bg="white").pack(pady=20)
            return

        # 2. Build UI Section by Section
        for cat_name, files in categorized_files.items():
            if not files: continue
            
            files.sort()
            config = TARGETS[cat_name]

            # Section Frame
            section_frame = tk.LabelFrame(self.scrollable_frame, text=f"  {cat_name}  ", 
                                          font=("Segoe UI", 11, "bold"), fg=config['color'], bg="white",
                                          bd=2, relief="groove", pady=5)
            section_frame.pack(fill="x", padx=10, pady=5)

            # "Select All" for this category
            cat_var = tk.IntVar(value=1)
            self.category_vars[cat_name] = cat_var
            
            # Sub-frame for header to align checkbox
            header_sub = tk.Frame(section_frame, bg=config['bg'])
            header_sub.pack(fill="x")
            
            chk_all_cat = tk.Checkbutton(header_sub, text=f"Select All {cat_name}", variable=cat_var,
                                         bg=config['bg'], font=("Segoe UI", 9, "bold"),
                                         command=lambda c=cat_name: self.toggle_category(c))
            chk_all_cat.pack(anchor="w", padx=5, pady=2)

            # List files
            for f_path in files:
                var = tk.IntVar(value=1)
                # Store tuple: (variable, category_name)
                self.file_vars[f_path] = {'var': var, 'cat': cat_name}
                
                chk = tk.Checkbutton(section_frame, text=f_path, variable=var, bg="white", 
                                     font=("Consolas", 10), command=self.update_counter)
                chk.pack(anchor="w", padx=20)

    def toggle_category(self, cat_name):
        """When a category header is clicked, toggle all its children."""
        new_state = self.category_vars[cat_name].get()
        for f_path, data in self.file_vars.items():
            if data['cat'] == cat_name:
                data['var'].set(new_state)
        self.update_counter()

    def update_counter(self):
        selected = sum(d['var'].get() for d in self.file_vars.values())
        total = len(self.file_vars)
        self.lbl_status.config(text=f"Selected: {selected} / {total} files")

    def get_combined_content(self):
        content_blocks = []
        selected_files = [f for f, d in self.file_vars.items() if d['var'].get() == 1]
        
        if not selected_files:
            return None

        for rel_path in selected_files:
            full_path = os.path.join(self.project_path, rel_path)
            try:
                with open(full_path, 'r', encoding='utf-8') as infile:
                    code = infile.read()
                    block = (f"{'='*50}\n"
                             f"FILE: {rel_path}\n"
                             f"{'='*50}\n"
                             f"{code}\n\n")
                    content_blocks.append(block)
            except Exception as e:
                print(f"Error reading {rel_path}: {e}")
        
        return "".join(content_blocks)

    def generate_file(self):
        content = self.get_combined_content()
        if not content:
            messagebox.showwarning("Warning", "No files selected!")
            return
            
        try:
            with open(OUTPUT_FILENAME, 'w', encoding='utf-8') as f:
                f.write(content)
            messagebox.showinfo("Success", f"Saved to {OUTPUT_FILENAME}")
        except Exception as e:
            messagebox.showerror("Error", str(e))

    def copy_to_clipboard(self):
        content = self.get_combined_content()
        if not content:
            messagebox.showwarning("Warning", "No files selected!")
            return

        self.root.clipboard_clear()
        self.root.clipboard_append(content)
        self.root.update()
        messagebox.showinfo("Copied", "Code copied to clipboard! \nYou can paste it into AI now.")

if __name__ == "__main__":
    try:
        # Check if user needs high DPI awareness (makes text sharp on Windows)
        from ctypes import windll
        windll.shcore.SetProcessDpiAwareness(1)
    except:
        pass

    root = tk.Tk()
    app = ModernCodeCollector(root)
    root.mainloop()