from __future__ import annotations

from pathlib import Path

from smoke import run_smoke

BASE_DIR = Path(__file__).resolve().parent
GENERATED_DIR = BASE_DIR / "generated"


def main() -> None:
    run_smoke(GENERATED_DIR)


if __name__ == "__main__":
    main()
