#!/usr/bin/env python3
"""Measure installed SDK startup in fresh processes with project caching disabled."""

import argparse
import json
import os
from pathlib import Path
import statistics
import shutil
import subprocess
import tempfile
import time


PROGRAM = '''import std.text.join;
flow main(args: Array<string>) -> i32 {
    return if join(["standard", "library"], " ") == "standard library" { 42 } else { 1 };
}
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--receipt-dir", type=Path, required=True)
    parser.add_argument("--runs", type=int, default=20)
    parser.add_argument("--sdk-cache", choices=["miss", "hit", "both"], default="both")
    parser.add_argument("--cpu", type=int)
    parser.add_argument("--max-ms", type=float, default=500)
    args = parser.parse_args()
    output = args.receipt_dir.resolve()
    output.mkdir(parents=True)
    binaries = {"current": args.binary.resolve()}
    if args.baseline:
        binaries["baseline"] = args.baseline.resolve()
    samples = []
    with tempfile.TemporaryDirectory(prefix="etas-std-startup-") as directory:
        application = Path(directory) / "app.es"
        application.write_text(PROGRAM)
        modes = ["miss", "hit"] if args.sdk_cache == "both" else [args.sdk_cache]
        variants = {f"{label}-{mode}": (binary, mode) for label, binary in binaries.items() for mode in modes}
        for label, (binary, mode) in variants.items():
            sdk_cache = Path(directory) / label
            environment = {**os.environ, "ETAS_STD_CACHE_ROOT": str(sdk_cache)}
            if mode == "hit":
                prime = subprocess.run([str(binary), "--no-config", "--cache", "off", "--format", "json", "run", str(application)],
                                       cwd=directory, env=environment, capture_output=True, text=True)
                assert prime.returncode == 0, prime.stderr
        # Interleave modes and binaries so warm-up and host-load changes affect
        # each variant throughout the same measurement interval.
        for index in range(args.runs):
            for label, (binary, mode) in variants.items():
                sdk_cache = Path(directory) / label
                environment = {**os.environ, "ETAS_STD_CACHE_ROOT": str(sdk_cache)}
                if mode == "miss":
                    shutil.rmtree(sdk_cache, ignore_errors=True)
                profile = output / f"{label}-{index}.profile.json"
                command = [str(binary), "--no-config", "--cache", "off", "--format", "json",
                           "--profile-out", str(profile), "run", str(application)]
                if args.cpu is not None:
                    command = ["taskset", "-c", str(args.cpu)] + command
                started = time.perf_counter()
                result = subprocess.run(command, cwd=directory, env=environment, capture_output=True, text=True)
                wall_ms = (time.perf_counter() - started) * 1000
                (output / f"{label}-{index}.output.json").write_text(result.stdout)
                (output / f"{label}-{index}.log").write_text(result.stderr)
                assert result.returncode == 0, (label, result.stderr, result.stdout)
                report = json.loads(result.stdout)
                assert report["value"]["value"] == "42" and not report["diagnostics"], report
                timing = json.loads(profile.read_text())
                spans = {span["name"]: round(span["duration_ns"] / 1e6, 3)
                         for span in timing["spans"]}
                counters = {counter["name"]: counter["value"] for counter in timing["counters"]}
                if label.startswith("current-"):
                    assert counters["std.object_pack_cache_hit"] == int(mode == "hit"), counters
                    assert counters["std.installed.source_parse_count"] == 0, counters
                    assert not any(name.startswith("std.restore.") for name in counters), counters
                samples.append({
                    "version": label,
                    "sdk_cache": mode,
                    "sdk_cache_hit": counters.get("std.object_pack_cache_hit"),
                    "wall_ms": round(wall_ms, 3),
                    "stages_ms": {name: spans.get(name, 0) for name in (
                        "std.load_artifact", "std.load_module_objects", "std.assemble_checked_objects",
                        "frontend.check", "interpreter.plan", "interpreter.eval")},
                    "loaded_body_modules": counters.get("std.loaded_body_modules"),
                    "loaded_body_bytes": counters.get("std.loaded_body_bytes"),
                    "parsed_standard_sources": counters.get("std.installed.source_parse_count"),
                    "loaded_public_behaviors": counters.get("std.loaded_public_behaviors"),
                })
    medians = {label: round(statistics.median(row["wall_ms"] for row in samples
                                            if row["version"] == label), 3)
               for label in variants}
    receipt = {"project_cache": "off", "sdk_cache": args.sdk_cache, "independent_processes": True, "cpu": args.cpu,
               "binaries": {label: str(binary) for label, binary in binaries.items()},
               "samples": samples, "wall_median_ms": medians,
               "max_ms": args.max_ms, "target_met": all(ms < args.max_ms for label, ms in medians.items() if label.startswith("current-"))}
    payload = json.dumps(receipt, indent=2) + "\n"
    (output / "receipt.json").write_text(payload)
    print(payload, end="")
    return 0 if receipt["target_met"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
