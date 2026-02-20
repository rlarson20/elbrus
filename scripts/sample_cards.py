# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Produce a random sample of cards from a Scryfall bulk export.

Usage:
    uv run scripts/sample_cards.py                                    # 10% of latest
    uv run scripts/sample_cards.py --fraction 0.05                    # 5%
    uv run scripts/sample_cards.py --count 500 --seed 42              # exactly 500
    uv run scripts/sample_cards.py --output data/json/sample.json     # write to file
"""

from __future__ import annotations

import argparse
import glob
import json
import random
import sys
from pathlib import Path

DEFAULT_DATA_DIR = Path("data/json")


def find_latest_export(data_dir: Path) -> Path:
    """Find the most recently dated oracle-cards export in data_dir."""
    pattern = str(data_dir / "oracle-cards-*.json")
    candidates = sorted(glob.glob(pattern))
    # Exclude files that look like samples
    candidates = [c for c in candidates if "sample" not in Path(c).name]
    if not candidates:
        print(
            f"Error: No oracle-cards-*.json files found in {data_dir}",
            file=sys.stderr,
        )
        sys.exit(1)
    return Path(candidates[-1])


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Sample cards from a Scryfall oracle-cards bulk export.",
    )
    parser.add_argument(
        "input",
        nargs="?",
        default=None,
        help="Path to the bulk JSON file (default: latest in data/json/)",
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--fraction",
        type=float,
        default=0.10,
        help="Fraction of cards to sample (default: 0.10)",
    )
    group.add_argument(
        "--count",
        type=int,
        default=None,
        help="Exact number of cards to sample",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=None,
        help="Random seed for reproducibility",
    )
    parser.add_argument(
        "--output",
        "-o",
        default=None,
        help="Output file path (default: stdout)",
    )
    args = parser.parse_args()

    # Resolve input path
    input_path = Path(args.input) if args.input else find_latest_export(DEFAULT_DATA_DIR)
    print(f"Loading {input_path} …", file=sys.stderr)

    with open(input_path, "r", encoding="utf-8") as f:
        cards: list[dict] = json.load(f)

    total = len(cards)
    print(f"Total cards: {total:,}", file=sys.stderr)

    # Determine sample size
    if args.count is not None:
        n = min(args.count, total)
    else:
        n = max(1, int(total * args.fraction))

    # Sample
    if args.seed is not None:
        random.seed(args.seed)
    sampled = random.sample(cards, n)

    print(f"Sampled {len(sampled):,} cards ({len(sampled)/total:.1%})", file=sys.stderr)

    # Output
    output_json = json.dumps(sampled, ensure_ascii=False, indent=2)

    if args.output:
        out_path = Path(args.output)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(output_json, encoding="utf-8")
        print(f"Written to {out_path}", file=sys.stderr)
    else:
        sys.stdout.write(output_json)
        sys.stdout.write("\n")


if __name__ == "__main__":
    main()
