# generate_gallery.py: formats the .png outputs of main.rs
# and puts them in a neat html file

import os
import re
from pathlib import Path
import time

# Configuration
RESULTS_DIR = "lang_results"
OUTPUT_FILE = f"gallery_{time.strftime('%y-%m-%d-%H%M')}.html"

def generate_html():
    results_path = Path(RESULTS_DIR)
    if not results_path.exists():
        print(f"Error: {RESULTS_DIR} directory not found.")
        return

    # Find all PNGs that are NOT "worst_case"
    # These are our main C(eps) curves
    image_paths = list(results_path.glob("**/*.png"))
    curve_images = [p for p in image_paths if "worst_case" not in p.name]

    # Sort by filename for consistency
    curve_images.sort()

    html_content = ["""
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Lang Conjecture - Linear Form Profiles</title>
        <style>
            body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f4f4f9; color: #333; margin: 0; padding: 20px; }
            h1 { text-align: center; color: #2c3e50; }
            .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(500px, 1fr)); gap: 25px; max-width: 1400px; margin: 0 auto; }
            .card { background: white; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); overflow: hidden; transition: transform 0.2s; }
            .card:hover { transform: translateY(-5px); }
            .card-header { padding: 15px; background: #2c3e50; color: white; font-weight: bold; font-size: 0.9rem; }
            .card img { width: 100%; height: auto; display: block; border-bottom: 1px solid #eee; }
            .card-meta { padding: 10px 15px; font-size: 0.8rem; color: #666; background: #fafafa; }
            .no-results { text-align: center; padding: 50px; color: #999; }
        </style>
    </head>
    <body>
        <h1>Linear Form Profiling Gallery</h1>
        <div class="grid">
    """]

    if not curve_images:
        html_content.append('<div class="no-results">No profile images found. Run the batch script first!</div>')
    else:
        for img_path in curve_images:
            # Extract form info from filename: a1_sqrt(2)_a2_sqrt(3)...
            filename = img_path.name

            # Use regex to pull out a1 and a2 for the label
            match = re.search(r'a1_(.*)_a2_(.*)_b1', filename)
            if match:
                label = f"α₁: {match.group(1)} | α₂: {match.group(2)}"
            else:
                label = filename.replace(".png", "")

            # Path relative to the HTML file
            rel_path = img_path.relative_to(Path("."))

            card_html = f"""
            <div class="card">
                <div class="card-header">{label}</div>
                <a href="{rel_path}" target="_blank">
                    <img src="{rel_path}" alt="C(epsilon) curve">
                </a>
                <div class="card-meta">Location: {img_path.parent}</div>
            </div>
            """
            html_content.append(card_html)

    html_content.append("""
        </div>
        <script>
            console.log("Gallery generated successfully.");
        </script>
    </body>
    </html>
    """)

    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        f.writelines(html_content)

    print(f"Dashboard generated! Open '{OUTPUT_FILE}' in your browser to see the results.")

if __name__ == "__main__":
    generate_html()
