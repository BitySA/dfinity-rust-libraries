#!/bin/bash

# Arrays to track results
successes=()
failures=()

# Get all workspace members (local packages only)
members=($(cargo metadata --format-version 1 --no-deps \
    | jq -r '.packages[] | select(.source == null) | .name'))

# Temporary directory for logging results between parallel jobs
tmpdir=$(mktemp -d)

# Run each publish in parallel
for crate in "${members[@]}"; do
    (
        echo "Dry-run publishing $crate..."
        if cargo publish --dry-run --allow-dirty -p "$crate" >"$tmpdir/$crate.log" 2>&1; then
            echo "$crate" > "$tmpdir/$crate.success"
        else
            echo "$crate" > "$tmpdir/$crate.fail"
        fi
    ) &
done

# Wait for all background jobs to finish
wait

# Collect results
for crate in "${members[@]}"; do
    if [[ -f "$tmpdir/$crate.success" ]]; then
        successes+=("$crate")
    elif [[ -f "$tmpdir/$crate.fail" ]]; then
        failures+=("$crate")
    fi
done

# Show summary
echo
echo "===== Dry-run publish summary ====="
echo "✅ Successes (${#successes[@]}): ${successes[*]}"
echo "❌ Failures  (${#failures[@]}): ${failures[*]}"

# Clean up
rm -rf "$tmpdir"
