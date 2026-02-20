# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "httpx",
#     "tqdm",
# ]
# ///
"""Download the latest Scryfall 'Oracle Cards' bulk export.

Usage:
    uv run scripts/download_bulk.py
    uv run scripts/download_bulk.py --type "Default Cards"
"""

from __future__ import annotations

import argparse
import os
import sys

import httpx
from tqdm import tqdm

BULK_DATA_LIST = "https://api.scryfall.com/bulk-data"


def get_download_uri(choice: str = "Oracle Cards") -> str | None:
    """Fetch the download URI for the specified bulk data type from Scryfall."""
    with httpx.Client() as client:
        response = client.get(BULK_DATA_LIST)
        response.raise_for_status()
        data = response.json().get("data", [])

        for item in data:
            if item.get("name") == choice:
                return item.get("download_uri")
    return None


def download_bulk_data(bulk_type: str, dest_dir: str) -> None:
    uri = get_download_uri(bulk_type)
    if not uri:
        print(f"Error: Could not find download URI for '{bulk_type}'", file=sys.stderr)
        sys.exit(1)

    os.makedirs(dest_dir, exist_ok=True)

    file_name = os.path.join(dest_dir, uri.split("/")[-1])

    if os.path.exists(file_name):
        print(f"File already exists: {file_name}", file=sys.stderr)
        print(f"Re-downloading anyway…", file=sys.stderr)

    print(f"Downloading {uri}\n  → {file_name}", file=sys.stderr)

    with httpx.stream("GET", uri, timeout=None) as response:
        response.raise_for_status()
        total_size = int(response.headers.get("Content-Length", 0))

        with open(file_name, "wb") as f:
            with tqdm(
                total=total_size, unit="B", unit_scale=True, desc="Status"
            ) as pbar:
                for chunk in response.iter_bytes(chunk_size=8192):
                    f.write(chunk)
                    pbar.update(len(chunk))

    print(f"\nDownload complete: {file_name}", file=sys.stderr)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Download Scryfall bulk data exports.",
    )
    parser.add_argument(
        "--type",
        default="Oracle Cards",
        help="Scryfall bulk-data type name (default: 'Oracle Cards')",
    )
    parser.add_argument(
        "--dest",
        default="data/json",
        help="Destination directory (default: data/json)",
    )
    args = parser.parse_args()
    download_bulk_data(args.type, args.dest)


if __name__ == "__main__":
    main()
