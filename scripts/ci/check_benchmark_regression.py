#!/usr/bin/env python3
"""
Benchmark Regression Gate — compares BenchmarkDotNet results against baselines.

Fails CI if any benchmark exceeds regression thresholds:
- Time: +20% (default) or configured via --threshold-time-pct
- Memory: +30% (default) or configured via --threshold-alloc-pct

Inputs:
- CSV from BenchmarkDotNet (--current-csv)
- Baseline markdown files (--baselines-dir, auto-scans docs/benchmarks/)
- Thresholds (configurable)

Output:
- JSON report: {status, pass/fail, violations, details}
- Exit code: 0 if pass, 1 if regression detected
"""

import argparse
import json
import re
import sys
import os
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple


@dataclass
class BenchmarkResult:
    """Single benchmark result from CSV."""
    method: str
    mean_microseconds: float
    allocated_bytes: float
    job: str

    def __repr__(self) -> str:
        return (
            f"BenchmarkResult(method={self.method}, "
            f"mean={self.mean_microseconds:.2f}µs, "
            f"allocated={self.allocated_bytes/1024:.1f}KB)"
        )


@dataclass
class BaselineEntry:
    """Single baseline benchmark entry from markdown."""
    method: str
    mean_microseconds: float
    allocated_bytes: float
    source_file: str


@dataclass
class RegressionViolation:
    """Single regression violation."""
    benchmark_name: str
    metric: str  # "time" or "allocation"
    baseline_value: float
    current_value: float
    pct_increase: float
    threshold_pct: float
    unit: str


class CsvParser:
    """Parse BenchmarkDotNet CSV output."""

    @staticmethod
    def parse(csv_path: str) -> Dict[str, BenchmarkResult]:
        """
        Parse BDN CSV file (handles quoted fields).
        Returns dict: benchmark_method -> BenchmarkResult
        """
        import csv
        results = {}
        try:
            with open(csv_path, 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                if not reader.fieldnames:
                    return results

                if 'Method' not in reader.fieldnames or 'Mean' not in reader.fieldnames:
                    raise ValueError("CSV missing required columns (Method, Mean)")

                for line_num, row in enumerate(reader, start=2):
                    try:
                        method = row.get('Method', '').strip()
                        mean_str = row.get('Mean', '').strip()
                        alloc_str = row.get('Allocated', '').strip()
                        job = row.get('Job', 'unknown').strip()

                        if not method or not mean_str:
                            continue

                        mean_us = CsvParser._parse_metric(mean_str)
                        allocated_bytes = CsvParser._parse_bytes(alloc_str) if alloc_str else 0.0

                        key = f"{method}_{job}"
                        results[key] = BenchmarkResult(
                            method=method,
                            mean_microseconds=mean_us,
                            allocated_bytes=allocated_bytes,
                            job=job
                        )
                    except (ValueError, IndexError) as e:
                        print(f"Warning: Could not parse line {line_num}: {e}", file=sys.stderr)
                        continue
        except FileNotFoundError:
            raise ValueError(f"CSV file not found: {csv_path}")

        return results

    @staticmethod
    def _parse_metric(s: str) -> float:
        """Parse metric string like '42,715.52 μs' or '39.42 μs' to microseconds."""
        s = s.strip()
        # Remove commas (European-style thousands sep in BDN output)
        s = s.replace(',', '')
        # Extract just the number part
        match = re.match(r'([\d.]+)', s)
        if not match:
            raise ValueError(f"Cannot parse metric: {s}")
        return float(match.group(1))

    @staticmethod
    def _parse_bytes(s: str) -> float:
        """Parse allocation like '27.7 KB' or '27699.16 KB' to bytes."""
        s = s.strip()
        s = s.replace(',', '')  # Remove thousands sep

        match = re.match(r'([\d.]+)\s*([KMGT]?B)', s)
        if not match:
            raise ValueError(f"Cannot parse bytes: {s}")

        value = float(match.group(1))
        unit = match.group(2).upper()

        multipliers = {'B': 1, 'KB': 1024, 'MB': 1024**2, 'GB': 1024**3}
        multiplier = multipliers.get(unit, 1)
        return value * multiplier


class MarkdownBaselineParser:
    """Parse baseline data from markdown files in docs/benchmarks/."""

    @staticmethod
    def parse_all(baselines_dir: str) -> Dict[str, BaselineEntry]:
        """
        Scan all .md files in baselines_dir and extract baseline benchmarks.
        Returns dict: benchmark_method -> BaselineEntry
        """
        baselines = {}
        baselines_path = Path(baselines_dir)

        if not baselines_path.exists():
            print(f"Warning: Baselines directory not found: {baselines_dir}", file=sys.stderr)
            return baselines

        for md_file in baselines_path.glob('*.md'):
            # Skip index, README, etc.
            if md_file.stem in ['index', 'README', 'readme']:
                continue

            found = MarkdownBaselineParser._parse_file(md_file)
            baselines.update(found)

        return baselines

    @staticmethod
    def _parse_file(md_path: Path) -> Dict[str, BaselineEntry]:
        """Parse single markdown baseline file with markdown table format."""
        results = {}
        try:
            with open(md_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()

            in_table = False
            header_indices = {}  # column_name -> column_index

            for line_idx, line in enumerate(lines):
                if not line.strip():
                    in_table = False
                    continue

                if not line.startswith('|'):
                    in_table = False
                    continue

                parts = [p.strip() for p in line.split('|')[1:-1]]

                # Skip separator rows
                if all('---' in p or '-' in p or p == '' for p in parts):
                    continue

                # Detect header row (contains column names)
                if not in_table and any('Benchmark' in p or 'Method' in p for p in parts):
                    # This is a header row
                    header_indices = {}
                    for col_idx, col_name in enumerate(parts):
                        col_name_lower = col_name.lower()
                        if 'benchmark' in col_name_lower or 'method' in col_name_lower:
                            header_indices['name'] = col_idx
                        elif 'mean' in col_name_lower:
                            header_indices['mean'] = col_idx
                        elif 'allocat' in col_name_lower:
                            header_indices['alloc'] = col_idx
                    in_table = True
                    continue

                # Parse data row
                if in_table and header_indices and len(parts) > max(header_indices.values()):
                    try:
                        name = parts[header_indices.get('name', 0)].strip()
                        if not name or '---' in name:
                            continue

                        mean_val = None
                        alloc_val = None

                        # Extract mean (e.g., "2.719", "42,715.52 μs")
                        if 'mean' in header_indices:
                            mean_str = parts[header_indices['mean']].strip()
                            try:
                                mean_val = CsvParser._parse_metric(mean_str)
                            except:
                                pass

                        # Extract allocation (e.g., "1.34 KB", "27.7 KB")
                        if 'alloc' in header_indices:
                            alloc_str = parts[header_indices['alloc']].strip()
                            try:
                                alloc_val = CsvParser._parse_bytes(alloc_str)
                            except:
                                pass

                        if mean_val is not None and mean_val > 0:
                            results[name] = BaselineEntry(
                                method=name,
                                mean_microseconds=mean_val,
                                allocated_bytes=alloc_val or 0.0,
                                source_file=md_path.name
                            )
                    except Exception as e:
                        continue
        except Exception as e:
            print(f"Warning: Could not parse baseline file {md_path}: {e}", file=sys.stderr)

        return results


class RegressionGate:
    """Check for performance regressions."""

    def __init__(
        self,
        threshold_time_pct: float = 20.0,
        threshold_alloc_pct: float = 30.0
    ):
        self.threshold_time_pct = threshold_time_pct
        self.threshold_alloc_pct = threshold_alloc_pct
        self.violations: List[RegressionViolation] = []

    def check(
        self,
        current: Dict[str, BenchmarkResult],
        baselines: Dict[str, BaselineEntry]
    ) -> Tuple[bool, List[RegressionViolation]]:
        """
        Compare current results against baselines.
        Returns (pass: bool, violations: List[RegressionViolation])
        """
        self.violations = []

        for baseline_name, baseline in baselines.items():
            # Try to match against current results
            # Strategies:
            # 1. Exact key match
            # 2. Match by method (extract method from "Name (Job-XXX)" format)
            # 3. Fuzzy match on method name prefix

            current_result = None

            # Extract method name from baseline_name (remove job suffix if present)
            baseline_method_clean = baseline_name.split('(')[0].strip()

            # Strategy 1: exact key match
            if baseline_name in current:
                current_result = current[baseline_name]
            else:
                # Strategy 2: match by method name (ignoring job suffix)
                for key, result in current.items():
                    result_method_clean = result.method.split('(')[0].strip()
                    if result_method_clean == baseline_method_clean:
                        current_result = result
                        break

                # Strategy 3: if still no match, try looser matching on method
                if not current_result:
                    for key, result in current.items():
                        if result.method == baseline.method:
                            current_result = result
                            break

            if not current_result:
                # This is expected - baselines may be from different suite
                continue

            # Check time regression
            if baseline.mean_microseconds > 0:
                time_increase_pct = (
                    (current_result.mean_microseconds - baseline.mean_microseconds) /
                    baseline.mean_microseconds * 100
                )
                if time_increase_pct > self.threshold_time_pct:
                    self.violations.append(RegressionViolation(
                        benchmark_name=baseline.method,
                        metric="time",
                        baseline_value=baseline.mean_microseconds,
                        current_value=current_result.mean_microseconds,
                        pct_increase=time_increase_pct,
                        threshold_pct=self.threshold_time_pct,
                        unit="μs"
                    ))

            # Check allocation regression
            if baseline.allocated_bytes > 0:
                alloc_increase_pct = (
                    (current_result.allocated_bytes - baseline.allocated_bytes) /
                    baseline.allocated_bytes * 100
                )
                if alloc_increase_pct > self.threshold_alloc_pct:
                    self.violations.append(RegressionViolation(
                        benchmark_name=baseline.method,
                        metric="allocation",
                        baseline_value=baseline.allocated_bytes / 1024,  # KB
                        current_value=current_result.allocated_bytes / 1024,  # KB
                        pct_increase=alloc_increase_pct,
                        threshold_pct=self.threshold_alloc_pct,
                        unit="KB"
                    ))

        return len(self.violations) == 0, self.violations


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark regression gate — compare BDN results vs baselines"
    )
    parser.add_argument(
        '--current-csv',
        type=str,
        default='src/Tests/Benchmarks/BenchmarkDotNet.Artifacts/results/results.csv',
        help='Path to current BenchmarkDotNet CSV result'
    )
    parser.add_argument(
        '--baselines-dir',
        type=str,
        default='docs/benchmarks/',
        help='Directory containing baseline markdown files'
    )
    parser.add_argument(
        '--threshold-time-pct',
        type=float,
        default=20.0,
        help='Time regression threshold percentage (default: 20%%)'
    )
    parser.add_argument(
        '--threshold-alloc-pct',
        type=float,
        default=30.0,
        help='Allocation regression threshold percentage (default: 30%%)'
    )
    parser.add_argument(
        '--output-json',
        type=str,
        default=None,
        help='Write JSON report to file instead of stdout'
    )
    parser.add_argument(
        '--test',
        action='store_true',
        help='Run self-test suite'
    )

    args = parser.parse_args()

    if args.test:
        return test_suite()

    # Real run
    try:
        print("Parsing current BenchmarkDotNet CSV...", file=sys.stderr)
        current = CsvParser.parse(args.current_csv)
        print(f"  Found {len(current)} benchmark results", file=sys.stderr)

        print("Parsing baseline markdown files...", file=sys.stderr)
        baselines = MarkdownBaselineParser.parse_all(args.baselines_dir)
        print(f"  Found {len(baselines)} baselines", file=sys.stderr)

        gate = RegressionGate(
            threshold_time_pct=args.threshold_time_pct,
            threshold_alloc_pct=args.threshold_alloc_pct
        )

        passed, violations = gate.check(current, baselines)

        # Build report
        report = {
            "status": "PASS" if passed else "FAIL",
            "passed": passed,
            "total_baselines": len(baselines),
            "total_current": len(current),
            "violations_count": len(violations),
            "threshold_time_pct": args.threshold_time_pct,
            "threshold_alloc_pct": args.threshold_alloc_pct,
            "violations": [
                {
                    "benchmark": v.benchmark_name,
                    "metric": v.metric,
                    "baseline_value": f"{v.baseline_value:.2f}",
                    "current_value": f"{v.current_value:.2f}",
                    "pct_increase": f"{v.pct_increase:.1f}%",
                    "threshold_pct": f"{v.threshold_pct:.1f}%",
                    "unit": v.unit
                }
                for v in violations
            ]
        }

        output = json.dumps(report, indent=2)

        if args.output_json:
            with open(args.output_json, 'w', encoding='utf-8') as f:
                f.write(output)
            print(f"Report written to: {args.output_json}", file=sys.stderr)
        else:
            print(output)

        if not passed:
            print("\n❌ Benchmark regressions detected:", file=sys.stderr)
            for v in violations:
                print(
                    f"  {v.benchmark_name} ({v.metric}): "
                    f"{v.baseline_value:.2f} → {v.current_value:.2f} {v.unit} "
                    f"(+{v.pct_increase:.1f}%, threshold: {v.threshold_pct:.1f}%)",
                    file=sys.stderr
                )
            return 1
        else:
            print("✅ No benchmark regressions detected", file=sys.stderr)
            return 0

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


def test_suite():
    """Run self-tests."""
    print("Running self-tests...", file=sys.stderr)

    passed = 0
    failed = 0

    # Test 1: Parse metric
    try:
        assert abs(CsvParser._parse_metric("42,715.52 μs") - 42715.52) < 0.01
        assert abs(CsvParser._parse_metric("39.42 μs") - 39.42) < 0.01
        print("  ✅ Test 1: _parse_metric PASSED", file=sys.stderr)
        passed += 1
    except AssertionError as e:
        print(f"  ❌ Test 1: _parse_metric FAILED", file=sys.stderr)
        failed += 1

    # Test 2: Parse bytes
    try:
        assert abs(CsvParser._parse_bytes("27.7 KB") - 27.7 * 1024) < 1
        assert abs(CsvParser._parse_bytes("27699.16 KB") - 27699.16 * 1024) < 1
        print("  ✅ Test 2: _parse_bytes PASSED", file=sys.stderr)
        passed += 1
    except AssertionError:
        print(f"  ❌ Test 2: _parse_bytes FAILED", file=sys.stderr)
        failed += 1

    # Test 3: Regression detection (time)
    try:
        gate = RegressionGate(threshold_time_pct=20.0, threshold_alloc_pct=30.0)
        current = {
            "test_benchmark_job": BenchmarkResult(
                method="test_benchmark",
                mean_microseconds=50.0,
                allocated_bytes=1024,
                job="job"
            )
        }
        baselines = {
            "test_benchmark": BaselineEntry(
                method="test_benchmark",
                mean_microseconds=40.0,
                allocated_bytes=1024,
                source_file="test.md"
            )
        }
        passed_test, vios = gate.check(current, baselines)
        # 50 is 25% higher than 40, should fail at 20% threshold
        assert not passed_test and len(vios) == 1 and vios[0].metric == "time"
        print("  ✅ Test 3: Regression detection (time) PASSED", file=sys.stderr)
        passed += 1
    except AssertionError:
        print(f"  ❌ Test 3: Regression detection (time) FAILED", file=sys.stderr)
        failed += 1

    # Test 4: No regression
    try:
        gate = RegressionGate(threshold_time_pct=20.0, threshold_alloc_pct=30.0)
        current = {
            "test_benchmark_job": BenchmarkResult(
                method="test_benchmark",
                mean_microseconds=42.0,
                allocated_bytes=1024,
                job="job"
            )
        }
        baselines = {
            "test_benchmark": BaselineEntry(
                method="test_benchmark",
                mean_microseconds=40.0,
                allocated_bytes=1024,
                source_file="test.md"
            )
        }
        passed_test, vios = gate.check(current, baselines)
        # 42 is only 5% higher than 40, should pass
        assert passed_test and len(vios) == 0
        print("  ✅ Test 4: No regression PASSED", file=sys.stderr)
        passed += 1
    except AssertionError:
        print(f"  ❌ Test 4: No regression FAILED", file=sys.stderr)
        failed += 1

    print(f"\nTest results: {passed} passed, {failed} failed", file=sys.stderr)
    return 0 if failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
