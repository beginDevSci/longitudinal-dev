#!/usr/bin/env python3
"""
Demote guide headings to create proper Distill-style hierarchy.

Spine sections (keep as H2):
- Overview
- Conceptual Foundations
- Model Specification & Fit
- Worked Example
- Reference & Resources

Everything else under these becomes H3+, with children cascading down.
"""

import re
import sys
from pathlib import Path

# Sections to keep as H2 (the spine)
LGCM_SPINE = [
    "Overview",
    "Conceptual Foundations",
    "Model Specification & Fit",
    "Worked Example",
    "Reference & Resources",
]

LMM_SPINE = [
    "Overview",
    "Conceptual Foundations",
    "Model Specification & Fit",
    "Worked Example",
    "Reference & Resources",
]

def normalize_title(title: str) -> str:
    """Normalize title for comparison (strip, lowercase)."""
    return title.strip().lower()

def get_spine_for_file(filepath: str) -> list:
    """Get the appropriate spine based on filename."""
    if "lgcm" in filepath.lower():
        return LGCM_SPINE
    elif "lmm" in filepath.lower():
        return LMM_SPINE
    else:
        return LGCM_SPINE  # Default

def demote_headings(content: str, spine: list) -> str:
    """
    Demote headings that aren't in the spine.

    Strategy:
    - Track when we enter a demoted section (H2 not in spine)
    - While in demoted section, add one # to all headings
    - Reset when we hit a spine H2
    """
    lines = content.split('\n')
    result = []
    in_demoted_section = False
    spine_normalized = [normalize_title(s) for s in spine]

    heading_pattern = re.compile(r'^(#{2,6})\s+(.+)$')

    for line in lines:
        match = heading_pattern.match(line)

        if match:
            hashes = match.group(1)
            title = match.group(2)
            level = len(hashes)

            # Check if this is an H2
            if level == 2:
                title_normalized = normalize_title(title)

                # Is this a spine section?
                if title_normalized in spine_normalized:
                    in_demoted_section = False
                    result.append(line)  # Keep as-is
                else:
                    in_demoted_section = True
                    # Demote H2 to H3
                    result.append(f"### {title}")
            else:
                # H3, H4, H5, H6
                if in_demoted_section:
                    # Demote by one level (add one #)
                    new_hashes = '#' * (level + 1)
                    result.append(f"{new_hashes} {title}")
                else:
                    result.append(line)  # Keep as-is
        else:
            result.append(line)

    return '\n'.join(result)

def count_h2s(content: str) -> int:
    """Count H2 headings in content."""
    return len(re.findall(r'^## ', content, re.MULTILINE))

def main():
    if len(sys.argv) < 2:
        print("Usage: python demote_guide_headings.py <guide.md> [--dry-run]")
        sys.exit(1)

    filepath = sys.argv[1]
    dry_run = "--dry-run" in sys.argv

    path = Path(filepath)
    if not path.exists():
        print(f"Error: {filepath} not found")
        sys.exit(1)

    content = path.read_text()
    spine = get_spine_for_file(filepath)

    print(f"Processing: {filepath}")
    print(f"Spine sections: {spine}")
    print(f"H2 count before: {count_h2s(content)}")

    result = demote_headings(content, spine)

    print(f"H2 count after: {count_h2s(result)}")

    if dry_run:
        print("\n--- DRY RUN (first 100 heading lines) ---")
        for line in result.split('\n'):
            if line.startswith('#'):
                print(line[:80])
    else:
        path.write_text(result)
        print(f"Written to {filepath}")

if __name__ == "__main__":
    main()
