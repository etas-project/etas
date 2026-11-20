#!/usr/bin/env python3
"""Exercise an installed source SDK from a separate application directory."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--sysroot", type=Path)
    parser.add_argument("--receipt-dir", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    output = args.receipt_dir.resolve()
    output.mkdir(parents=True)
    common = [str(binary), "--no-config", "--cache", "off", "--format", "json"]
    if args.sysroot:
        sysroot = args.sysroot.resolve()
        manifest = sysroot / "std/manifest.json"
        common += ["--std-provider", "artifact", "--sysroot", str(sysroot)]
    else:
        manifest, = (binary.parent.parent / "lib/etas").glob("*/std/manifest.json")

    fixtures = Path(__file__).resolve().parent / "fixtures/std_sdk"
    receipt = {
        "binary_sha256": digest(binary),
        "sdk_manifest_sha256": digest(manifest),
        "discovery": "explicit" if args.sysroot else "installed-prefix",
        "sources": {str(path.relative_to(fixtures)): digest(path)
                    for path in sorted(fixtures.rglob("*")) if path.is_file()},
        "entries": [],
    }
    started = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="etas-std-sdk-consumer-") as directory:
        application = Path(directory)
        for path in fixtures.iterdir():
            if path.is_dir():
                shutil.copytree(path, application / path.name)
            else:
                shutil.copy2(path, application / path.name)

        def invoke(label, arguments, expected=None, profile=None):
            tick = time.monotonic()
            profile_options = ["--profile-out", str(profile)] if profile else []
            result = subprocess.run(common + profile_options + arguments, cwd=application, capture_output=True, text=True)
            (output / (label + ".json")).write_text(result.stdout)
            (output / (label + ".log")).write_text(result.stderr)
            assert result.returncode == 0, (label, result.returncode, result.stderr, result.stdout)
            report = json.loads(result.stdout)
            diagnostics = ([diagnostic for file in report["files"] for diagnostic in file["diagnostics"]]
                           if arguments[0] == "check" else report["diagnostics"])
            assert diagnostics == [], (label, diagnostics)
            if expected is not None:
                assert report["value"]["value"] == str(expected), (label, report["value"])
            return report, round(time.monotonic() - tick, 3)

        for filename in ["collections.es", "tasks.es"]:
            invoke("check-" + filename, ["check", filename])
        profile = output / "collections.profile.json" if sys.platform == "linux" and not args.sysroot else None
        _, seconds = invoke("collections", ["run", "collections.es"], 42, profile=profile)
        if profile:
            counters = {counter["name"] for counter in json.loads(profile.read_text())["counters"]}
            assert "std.restore.effects.replay_prepared_facts.duration_ns" in counters, counters
            assert "std.restore.effects.solve_summaries.duration_ns" not in counters, counters
            assert "std.restore.effects.materialize_facts.duration_ns" not in counters, counters
        receipt["entries"].append({"flow": "collections", "value": 42, "seconds": seconds})
        invoke("check-files", ["check", "files"])
        _, seconds = invoke("files", ["run", "files"], 42)
        receipt["entries"].append({"flow": "files", "value": 42, "seconds": seconds})

        for name, labels, count in [
            ("verify_fifo", ["20", "22"], 2),
            ("verify_stateful", ["22", "20"], 2),
            ("verify_yielding", None, 2),
            ("verify_try_join", None, 0),
            ("verify_lexical", None, 0),
        ]:
            checkpoints = output / (name + "-checkpoints")
            keys = output / (name + "-keys")
            report, seconds = invoke(name, [
                "run", "tasks.es", "--flow", name,
                "--checkpoint-dir", str(checkpoints), "--checkpoint-key-dir", str(keys),
            ], 42)
            saved = report["checkpoints"]
            assert len(saved) == count, (name, saved)
            if labels is not None:
                assert [checkpoint["label"] for checkpoint in saved] == labels, (name, saved)
            if name == "verify_try_join":
                cancelled = next(event for event in report["events"]
                                 if event["kind"] == "task_cancellation_requested")
                assert any(event["kind"] == "task_cleanup_completed"
                           and (event["scope"], event["task"]) == (cancelled["scope"], cancelled["task"])
                           for event in report["events"])
            row = {"flow": name, "value": 42, "seconds": seconds, "checkpoints": count, "resumes": []}
            for checkpoint in saved:
                checkpoint_id = str(checkpoint["id"])
                resume = output / (name + "-resume-" + checkpoint_id)
                shutil.copytree(checkpoints, resume)
                _, seconds = invoke(name + "-resume-" + checkpoint_id, [
                    "resume", checkpoint_id, "--checkpoint-dir", str(resume),
                    "--checkpoint-key-dir", str(keys),
                ], 42)
                row["resumes"].append({"checkpoint": checkpoint["id"], "value": 42, "seconds": seconds})
            receipt["entries"].append(row)
            receipt["elapsed_seconds"] = round(time.monotonic() - started, 3)
            (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
            print(name, "PASS", flush=True)


if __name__ == "__main__":
    main()
