#!/usr/bin/env python3
"""BeiFeng benchmark baseline, regression, and trend pipeline."""

import argparse
import json
import sys
from datetime import datetime, timedelta
from pathlib import Path

import run_benchmark

EVAL_DIR = Path(__file__).resolve().parent
BASELINES_DIR = EVAL_DIR / "baselines"
CURRENT_RESULTS = EVAL_DIR / "benchmark_results_current.json"
LEGACY_RESULTS = EVAL_DIR / "benchmark_results.json"
DEFAULT_URL = run_benchmark.DEFAULT_URL


def main() -> int:
    parser = argparse.ArgumentParser(description="BeiFeng benchmark automation pipeline")
    sub = parser.add_subparsers(dest="cmd", required=True)

    save = sub.add_parser("save-baseline", help="Save current benchmark results as baseline")
    save.add_argument("--results", default=None, help="Results JSON path to save")

    baseline = sub.add_parser("baseline", help="Alias for save-baseline")
    baseline.add_argument("--results", default=None, help="Results JSON path to save")

    regress = sub.add_parser("regress", help="Run benchmark and compare against baseline")
    regress.add_argument("--against", default=str(BASELINES_DIR / "latest.json"))
    regress.add_argument("--url", default=DEFAULT_URL)
    regress.add_argument("--threshold", type=float, default=5.0)

    trend = sub.add_parser("trend", help="Print score trends from saved baselines")
    trend.add_argument("--days", type=int, default=30)

    args = parser.parse_args()
    if args.cmd in ("save-baseline", "baseline"):
        return save_baseline(Path(args.results) if args.results else latest_results_path())
    if args.cmd == "regress":
        return regress_against(Path(args.against), args.url, args.threshold)
    if args.cmd == "trend":
        return trend_report(args.days)
    return 1


def latest_results_path() -> Path:
    if CURRENT_RESULTS.exists():
        return CURRENT_RESULTS
    return LEGACY_RESULTS


def save_baseline(results_path: Path) -> int:
    if not results_path.exists():
        print(f"ERROR: results file not found: {results_path}")
        return 1
    data = read_json(results_path)
    BASELINES_DIR.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now().strftime("%Y-%m-%d")
    dated = BASELINES_DIR / f"{stamp}.json"
    dated.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    (BASELINES_DIR / "latest.json").write_text(
        json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(f"Saved baseline: {dated}")
    print(f"Updated latest: {BASELINES_DIR / 'latest.json'}")
    return 0


def regress_against(baseline_path: Path, url: str, threshold: float) -> int:
    if not baseline_path.exists():
        print(f"ERROR: baseline file not found: {baseline_path}")
        return 1
    if not run_benchmark.check_health(url):
        print(f"ERROR: RAG service not reachable at {url}")
        return 1

    benchmark = read_json(run_benchmark.BENCHMARK_PATH)
    current = run_benchmark.run_evaluation(url, benchmark, enable_report=True)
    CURRENT_RESULTS.write_text(
        json.dumps(current, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    baseline = read_json(baseline_path)
    report = regression_report(baseline, current, baseline_path, threshold)
    print(report)
    return 1 if "Result: FAIL" in report else 0


def trend_report(days: int) -> int:
    if not BASELINES_DIR.exists():
        print(f"No baselines directory found: {BASELINES_DIR}")
        return 1
    cutoff = datetime.now() - timedelta(days=days)
    rows = []
    for path in sorted(BASELINES_DIR.glob("*.json")):
        if path.name == "latest.json":
            continue
        try:
            date = datetime.strptime(path.stem, "%Y-%m-%d")
        except ValueError:
            continue
        if date < cutoff:
            continue
        data = read_json(path)
        rows.append((path.stem, data))

    if not rows:
        print(f"No baselines found in the last {days} days.")
        return 0

    dimensions = sorted({k for _, data in rows for k in data.get("category_scores", {})})
    print(f"=== BeiFeng Benchmark Trend ({days} days) ===")
    print("Date        OVERALL  " + "  ".join(dim[:12].ljust(12) for dim in dimensions))
    print("-" * (20 + len(dimensions) * 14))
    for date, data in rows:
        values = [pct(data.get("overall_pct", 0))]
        for dim in dimensions:
            values.append(pct(category_pct(data, dim)))
        print(f"{date}  " + "  ".join(value.rjust(7) for value in values))
    return 0


def regression_report(baseline: dict, current: dict, baseline_path: Path, threshold: float) -> str:
    baseline_date = baseline_path.stem
    current_date = datetime.now().strftime("%Y-%m-%d")
    dimensions = sorted(
        set(baseline.get("category_scores", {})) | set(current.get("category_scores", {}))
    )
    lines = [
        "=== BeiFeng Benchmark Regression Report ===",
        f"Date: {current_date}  vs Baseline: {baseline_date}",
        "",
        "Dimension              Baseline   Current    Delta",
        "-------------------------------------------------",
    ]
    failed = False
    for dim in dimensions:
        b = category_pct(baseline, dim)
        c = category_pct(current, dim)
        delta = c - b
        ok = delta >= -threshold
        failed = failed or not ok
        lines.append(
            f"{dim.ljust(22)} {pct(b).rjust(8)}   {pct(c).rjust(8)}   {signed_pct(delta).rjust(8)} {'OK' if ok else 'FAIL'}"
        )
    overall_delta = current.get("overall_pct", 0) - baseline.get("overall_pct", 0)
    overall_ok = overall_delta >= -threshold
    failed = failed or not overall_ok
    lines.extend(
        [
            "-------------------------------------------------",
            f"{'OVERALL'.ljust(22)} {pct(baseline.get('overall_pct', 0)).rjust(8)}   {pct(current.get('overall_pct', 0)).rjust(8)}   {signed_pct(overall_delta).rjust(8)} {'OK' if overall_ok else 'FAIL'}",
            "",
            f"Result: {'FAIL' if failed else 'PASS'} (no dimension regressed > {threshold:.1f}%)",
        ]
    )
    return "\n".join(lines)


def category_pct(data: dict, category: str) -> float:
    scores = data.get("category_scores", {}).get(category, {})
    mx = scores.get("max", 0)
    if not mx:
        return 0.0
    return round(scores.get("earned", 0) / mx * 100, 1)


def pct(value: float) -> str:
    return f"{value:.1f}%"


def signed_pct(value: float) -> str:
    return f"{value:+.1f}%"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    sys.exit(main())
