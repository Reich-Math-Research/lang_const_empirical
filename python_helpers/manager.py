# manager.py: runs main.rs repeatedly using a predetermined list of alpha values

import subprocess
import time
from pathlib import Path
from itertools import combinations, product

# ==============================================================================
# CONFIGURATION
# ==============================================================================

# Define the pairs of (a1, a2) you want to investigate.
# Add as many as you like! They will run sequentially.

from itertools import combinations, islice

def make_linear_forms(limit=20):
    atoms = [
        "sqrt(2)", "sqrt(3)", "sqrt(5)", "sqrt(7)", "sqrt(11)", "sqrt(13)",
        "(sqrt(2)+1)/2", "(sqrt(3)+1)/2", "(sqrt(5)+1)/2", "(sqrt(7)+1)/2",
        "(sqrt(11)+1)/2", "(sqrt(13)+1)/2",
        "(sqrt(2)-1)/2", "(sqrt(3)-1)/2", "(sqrt(5)-1)/2", "(sqrt(7)-1)/2",
        "(sqrt(11)-1)/2", "(sqrt(13)-1)/2",
        "(1+sqrt(5))/2", "(1+sqrt(3))/2", "(1+sqrt(7))/2",
    ]

    forms = []
    for a, b in islice(combinations(atoms, 2), limit):
        forms.append((a, b))
    return forms

LINEAR_FORMS = make_linear_forms(limit=200)

# Optional: cap the size if you want a quick run
# LINEAR_FORMS = LINEAR_FORMS[:200]

# Shared search parameters
B1_MIN = 1
B1_MAX = 100000000
PRECISION = 2000
EPS_MIN = -0.35
EPS_MAX = 0.35
EPS_STEPS = 2000

# Path to your compiled Rust binary
# Note: On Windows, this would be "target/release/lang_conjecture_search.exe"
RUST_BINARY = "./target/release/lang_conjecture"

# ==============================================================================
# RUNNER LOGIC
# ==============================================================================

def main():
    binary_path = Path(RUST_BINARY)
    if not binary_path.exists():
        print(f"❌ Error: Could not find binary at {RUST_BINARY}")
        print("👉 Please run `cargo build --release` first.")
        return

    log_file_path = Path("batch_run.log")
    print(f"🚀 Starting batch run. Detailed output is mirrored to '{log_file_path}'")

    with open(log_file_path, "a", encoding="utf-8") as log_file:
        total_forms = len(LINEAR_FORMS)

        for index, (a1, a2) in enumerate(LINEAR_FORMS, start=1):
            start_time = time.time()

            log_and_print(f"\n[{index}/{total_forms}] Processing a1='{a1}' and a2='{a2}'", log_file)

            # Construct the command. Clap converts snake_case fields to kebab-case (e.g. b1_min -> --b1-min)
            command = [
                str(binary_path),
                f"--a1={a1}",
                f"--a2={a2}",
                f"--b1-min={B1_MIN}",
                f"--b1-max={B1_MAX}",
                f"--prec={PRECISION}",
                f"--eps-min={EPS_MIN}",
                f"--eps-max={EPS_MAX}",
                f"--eps-steps={EPS_STEPS}",
            ]

            try:
                # Run the process and capture the output
                result = subprocess.run(
                    command,
                    capture_output=True,
                    text=True,
                    check=True
                )

                # Log success and output
                log_and_print(result.stdout, log_file)

                elapsed = time.time() - start_time
                log_and_print(f"✅ Completed in {elapsed:.2f} seconds.\n", log_file)

            except subprocess.CalledProcessError as e:
                log_and_print(f"❌ Error running form ({a1}, {a2}):", log_file)
                log_and_print(e.stderr, log_file)
                print("Skipping to next form...")
                continue

    print(f"\n🎉 All done! Check 'lang_results' folder for generated data.")


def log_and_print(message: str, file_handle):
    """Prints to the terminal and appends to a log file."""
    print(message)
    file_handle.write(message + "\n")
    file_handle.flush()


if __name__ == "__main__":
    main()
