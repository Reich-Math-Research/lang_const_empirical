# regime_diff.py: takes the output jsons of main.rs and formats
# them into an html file made to compare the differences in one
# linear form between different B_max values

import json
import time
from pathlib import Path
from collections import defaultdict

RESULTS_DIR = "lang_results"
OUTPUT_FILE = f"reports_and_galleries/diff_report_{time.strftime('%y-%m-%d-%H%M')}.html"

def load_json_data():
    data_map = defaultdict(list)
    results_path = Path(RESULTS_DIR)
    for json_file in results_path.glob("**/*.json"):
        with open(json_file, 'r', encoding='utf-8') as f:
            try:
                content = json.load(f)
                # Use the raw string expressions as the key for grouping
                # These are stored in content['args']['a1'] and ['a2']
                expr_a1 = content['args'].get('a1', 'Unknown')
                expr_a2 = content['args'].get('a2', 'Unknown')

                key = (expr_a1, expr_a2)
                data_map[key].append(content)
            except Exception as e:
                print(f"Skipping {json_file}: {e}")
                continue
    return data_map

def compare_regimes(old, new):
    old_regs = {(r['b1'], r['b2']): r for r in old['regimes']}
    new_regs = {(r['b1'], r['b2']): r for r in new['regimes']}

    all_keys = sorted(set(old_regs.keys()) | set(new_regs.keys()), key=lambda x: abs(x[0]))
    rows = []

    for k in all_keys:
        o, n = old_regs.get(k), new_regs.get(k)

        if not o:
            type_tag, color = "NEW", "#d4edda" # Green
            ds, de = n['eps_range'][0], n['eps_range'][1]
        elif not n:
            type_tag, color = "LOST", "#f8d7da" # Red
            ds, de = -o['eps_range'][0], -o['eps_range'][1]
        else:
            ds = n['eps_range'][0] - o['eps_range'][0]
            de = n['eps_range'][1] - o['eps_range'][1]
            if abs(ds) > 0.001 or abs(de) > 0.001:
                type_tag, color = "SHIFT", "#fff3cd" # Yellow
            else:
                continue # Skip identical regimes to keep report clean

        rows.append(f"""
            <tr style="background-color: {color};">
                <td>{k[0]}, {k[1]}</td>
                <td><strong>{type_tag}</strong></td>
                <td>{ds:+.4f}</td>
                <td>{de:+.4f}</td>
                <td>{n['B'] if n else 'N/A'}</td>
            </tr>
        """)
    return rows

def generate_report():
    groups = load_json_data()
    html_sections = []

    for (expr_a1, expr_a2), runs in groups.items():
            if len(runs) < 2: continue

            # Sort by B_max (stored as 'b1_max' in args or 'b_range' in metadata)
            runs.sort(key=lambda x: x['metadata']['b_range'][1])

            for i in range(len(runs) - 1):
                old, new = runs[i], runs[i+1]
                diff_rows = compare_regimes(old, new)

                if not diff_rows: continue

                section = f"""
                <div class="analysis-card">
                    <h3>Form: α₁ = <code style="background:#eee; padding:2px 5px;">{expr_a1}</code>,
                               α₂ = <code style="background:#eee; padding:2px 5px;">{expr_a2}</code></h3>
                    <p class="meta">
                        Comparison: B range [{old['metadata']['b_range'][1]}]
                        &rarr; [{new['metadata']['b_range'][1]}]
                    </p>
                <table>
                    <thead>
                        <tr>
                            <th>Regime (b1, b2)</th>
                            <th>Change Type</th>
                            <th>Δ ε_start</th>
                            <th>Δ ε_end</th>
                            <th>B_max</th>
                        </tr>
                    </thead>
                    <tbody>
                        {"".join(diff_rows)}
                    </tbody>
                </table>
            </div>
            """
            html_sections.append(section)

    full_html = f"""
    <!DOCTYPE html>
    <html>
    <head>
        <title>Lang Conjecture Delta Report</title>
        <style>
            body {{ font-family: sans-serif; background: #f0f2f5; padding: 40px; }}
            .analysis-card {{ background: white; padding: 20px; border-radius: 8px; margin-bottom: 30px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
            h3 {{ margin-top: 0; color: #1a73e8; }}
            .meta {{ color: #666; font-size: 0.9em; }}
            table {{ width: 100%; border-collapse: collapse; margin-top: 15px; }}
            th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #eee; }}
            th {{ background: #fafafa; }}
            tr:hover {{ filter: brightness(95%); }}
        </style>
        <meta charset="UTF-8">  <title>Lang Conjecture Delta Report</title>
    </head>
    <body>
        <h1>Regime Shift Analysis</h1>
        {"".join(html_sections) if html_sections else "<p>No significant regime shifts found between runs.</p>"}
    </body>
    </html>
    """

    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        f.write(full_html)
    print(f"Report generated: {OUTPUT_FILE}")

if __name__ == "__main__":
    generate_report()
