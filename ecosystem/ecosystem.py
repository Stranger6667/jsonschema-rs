"""Compare canonicalization of real-world API schemas between two builds.

    python ecosystem/ecosystem.py fetch --corpus DIR
    python ecosystem/ecosystem.py run --binary BIN --corpus DIR --output results.jsonl
    python ecosystem/ecosystem.py report --old OLD.jsonl --new NEW.jsonl \
        --old-binary OLD_BIN --new-binary NEW_BIN --corpus DIR --output comment.md --full-output full.md

`run` and `report` cap each process's memory with util-linux `prlimit`, so they need Linux.
"""

from __future__ import annotations

import argparse
import difflib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import urllib.request
from collections import Counter, defaultdict
from collections.abc import Sequence
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path

CORPUS_COMMIT = "8726f8205a01a5215d2009a21321d90bda1dfd6d"
CORPUS_URL = "https://github.com/schemathesis/schemathesis/raw/{commit}/corpus/data/{name}.tar.gz"
CORPUS_ARCHIVES = ("openapi-3.0", "openapi-3.1", "swagger-2.0")
MARKER = "<!-- canonical-ecosystem -->"
# GitHub rejects comments above 65536 characters.
COMMENT_CHARS = 60_000
# GitHub drops a job summary above 1 MiB.
FULL_REPORT_CHARS = 900_000
COMMENT_DIFFS = 20
FULL_DIFFS = 200
# Room for the changed-outputs section's own summary, table header and wrapper lines.
SECTION_OVERHEAD_CHARS = 1_000
MAX_DIFF_LINES = 80
MAX_ROWS = 30
MAX_DETAIL_CHARS = 200
MEMORY_LIMIT_BYTES = 3 * 1024**3
TIMEOUT = 60
CRASH_STATUSES = {"crash", "timeout"}

Key = tuple[str, str]


@dataclass(frozen=True, slots=True)
class Result:
    status: str
    hash: str
    # Why a document crashed or timed out; empty for schemas.
    detail: str = ""


@dataclass(frozen=True, slots=True)
class Diff:
    key: Key
    lines: tuple[str, ...]


def fetch(corpus: Path) -> None:
    for name in CORPUS_ARCHIVES:
        destination = corpus / name
        if destination.exists():
            continue
        partial = corpus / f"{name}.partial"
        shutil.rmtree(partial, ignore_errors=True)
        partial.mkdir(parents=True)
        url = CORPUS_URL.format(commit=CORPUS_COMMIT, name=name)
        with urllib.request.urlopen(url, timeout=120) as response:
            payload = response.read()
        with tarfile.open(fileobj=io.BytesIO(payload), mode="r:gz") as archive:
            archive.extractall(partial, filter="data")
        # The rename is atomic, so `destination` exists only once the archive extracted fully.
        partial.rename(destination)
        print(f"{name}: {sum(1 for _ in destination.rglob('*.json'))} documents")


def documents(corpus: Path) -> list[Path]:
    # Only the extracted archives, never a `.partial` directory a killed run left behind.
    return sorted(path for name in CORPUS_ARCHIVES for path in (corpus / name).rglob("*.json"))


def limited(command: list[str | Path]) -> list[str | Path]:
    # A `preexec_fn` limit is unsafe from the worker threads that spawn these processes.
    return ["prlimit", f"--as={MEMORY_LIMIT_BYTES}", *command]


def failure(document: str, status: str, detail: str) -> str:
    return json.dumps({"document": document, "pointer": "-", "status": status, "hash": "-", "detail": detail})


def run_document(binary: Path, corpus: Path, document: Path) -> list[str]:
    name = document.relative_to(corpus).as_posix()
    try:
        completed = subprocess.run(limited([binary, document]), capture_output=True, encoding="utf-8", timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return [failure(name, "timeout", f"after {TIMEOUT}s")]
    if completed.returncode != 0:
        return [failure(name, "crash", crash_detail(completed))]
    # Not `splitlines`: it also splits on U+2028 and similar, which JSON strings carry unescaped.
    return [json.dumps({"document": name, **json.loads(line)}) for line in completed.stdout.split("\n") if line]


def crash_detail(completed: subprocess.CompletedProcess[str]) -> str:
    code = completed.returncode
    cause = f"signal {-code}" if code < 0 else f"exit {code}"
    # A Rust panic ends with a `note:` line about backtraces; the message is the line before it.
    messages = [line for line in completed.stderr.split("\n") if line.strip() and not line.startswith("note:")]
    message = messages[-1].strip() if messages else ""
    if len(message) > MAX_DETAIL_CHARS:
        message = message[:MAX_DETAIL_CHARS] + "..."
    return f"{cause}: {message}"


def run(binary: Path, corpus: Path, output: Path) -> None:
    with ThreadPoolExecutor(os.cpu_count()) as pool:
        lines = pool.map(lambda document: run_document(binary, corpus, document), documents(corpus))
        output.write_text("".join(f"{line}\n" for chunk in lines for line in chunk), encoding="utf-8")


def load(path: Path) -> dict[Key, Result]:
    results = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        entry = json.loads(line)
        results[(entry["document"], entry["pointer"])] = Result(entry["status"], entry["hash"], entry.get("detail", ""))
    return results


def canonical_form(binary: Path, corpus: Path, document: str, pointer: str) -> list[str]:
    try:
        completed = subprocess.run(
            limited([binary, corpus / document, pointer]), capture_output=True, encoding="utf-8", timeout=TIMEOUT
        )
    except subprocess.TimeoutExpired:
        return [f"<timeout after {TIMEOUT}s>"]
    if completed.returncode != 0:
        return [f"<crash: exit {completed.returncode}>", *completed.stderr.split("\n")]
    return completed.stdout.rstrip("\n").split("\n")


def longest_backtick_run(text: str) -> int:
    return max((len(run) for run in re.findall(r"`+", text)), default=0)


def code(text: str) -> str:
    delimiter = "`" * (longest_backtick_run(text) + 1)
    padding = " " if text.startswith("`") or text.endswith("`") else ""
    return f"{delimiter}{padding}{text}{padding}{delimiter}"


def fence(lines: Sequence[str]) -> str:
    return "`" * max(3, longest_backtick_run("\n".join(lines)) + 1)


def number(value: int) -> str:
    return f"{value:,}"


def target(key: Key) -> str:
    return f"{code(key[0])} {code(key[1])}"


def row(*cells: str) -> str:
    # A pipe ends a table cell even inside a code span.
    return "| " + " | ".join(cell.replace("|", "\\|") for cell in cells) + " |"


def details(summary: str, body: list[str]) -> list[str]:
    return ["<details>", f"<summary>{summary}</summary>", "", *body, "", "</details>", ""]


def more(total: int) -> list[str]:
    return ["", f"and {number(total - MAX_ROWS)} more"] if total > MAX_ROWS else []


def table(title: str, header: tuple[str, ...], rows: list[tuple[str, ...]], intro: tuple[str, ...] = ()) -> list[str]:
    body = [*intro, row(*header), "|" + "---|" * len(header)]
    body += [row(*cells) for cells in rows[:MAX_ROWS]] + more(len(rows))
    return details(f"{title} ({number(len(rows))})", body)


def changes(old: dict[Key, Result], new: dict[Key, Result]) -> tuple[list[Key], list[Key]]:
    # Document-level `-` rows are compared by `preparation_changes` and `crash_transitions`.
    keys = sorted(key for key in old.keys() & new.keys() if key[1] != "-")
    status_changes = [key for key in keys if old[key].status != new[key].status]
    output_changes = [key for key in keys if old[key].status == new[key].status and old[key].hash != new[key].hash]
    return status_changes, output_changes


def preparation(results: dict[Key, Result]) -> dict[str, str]:
    states: dict[str, str] = {}
    for (document, _), result in results.items():
        if result.status in CRASH_STATUSES:
            states[document] = result.status
        elif result.status == "prepare-error":
            states[document] = "prepare-error"
        else:
            states.setdefault(document, "prepared")
    return states


def preparation_changes(old: dict[Key, Result], new: dict[Key, Result]) -> list[tuple[str, str, str]]:
    # A document that fails to prepare has one `-` row in place of its schemas, so no schema pairs up.
    # Both runs read one corpus, so a document missing from a run has no schemas there and prepared fine.
    before, after = preparation(old), preparation(new)
    pairs = (
        (document, before.get(document, "prepared"), after.get(document, "prepared")) for document in before | after
    )
    return sorted(pair for pair in pairs if pair[1] != pair[2] and CRASH_STATUSES.isdisjoint(pair[1:]))


def crashes_by_document(results: dict[Key, Result]) -> dict[str, Result]:
    return {document: result for (document, _), result in results.items() if result.status in CRASH_STATUSES}


def crash_transitions(
    old: dict[Key, Result], new: dict[Key, Result]
) -> tuple[list[tuple[str, Result]], list[tuple[str, Result]]]:
    # A crashed document has one `-` row in place of its schemas, so compare documents, not schemas.
    crashed_before, crashed_after = crashes_by_document(old), crashes_by_document(new)
    started = [(document, crashed_after[document]) for document in sorted(crashed_after.keys() - crashed_before.keys())]
    stopped = [
        (document, crashed_before[document]) for document in sorted(crashed_before.keys() - crashed_after.keys())
    ]
    return started, stopped


def by_document(changed: list[Key]) -> list[tuple[str, list[str]]]:
    grouped: dict[str, list[str]] = defaultdict(list)
    for document, pointer in changed:
        grouped[document].append(pointer)
    return sorted(grouped.items(), key=lambda item: (-len(item[1]), item[0]))


def render_diffs(changed: list[Key], old_binary: Path, new_binary: Path, corpus: Path, limit: int) -> list[Diff]:
    diffs = []
    # One schema per document first, so the diffs cover as many documents as possible.
    for document, pointers in by_document(changed)[:limit]:
        key = (document, pointers[0])
        lines = list(
            difflib.unified_diff(
                canonical_form(old_binary, corpus, *key),
                canonical_form(new_binary, corpus, *key),
                "old",
                "new",
                n=2,
                lineterm="",
            )
        )
        if len(lines) > MAX_DIFF_LINES:
            lines = lines[:MAX_DIFF_LINES] + [f"... {len(lines) - MAX_DIFF_LINES} more lines"]
        diffs.append(Diff(key, tuple(lines)))
    return diffs


def changed_section(changed: list[Key], diffs: list[Diff], max_diffs: int, room: int) -> list[str]:
    ranked = by_document(changed)
    counts = ["| Document | Changed |", "|---|---:|"]
    counts += [row(code(document), str(len(pointers))) for document, pointers in ranked[:MAX_ROWS]]
    counts += more(len(ranked))
    used = sum(len(line) + 1 for line in counts)
    blocks: list[str] = []
    shown = 0
    for diff in diffs[:max_diffs]:
        delimiter = fence(diff.lines)
        block = [target(diff.key), f"{delimiter}diff", *diff.lines, delimiter, ""]
        size = sum(len(line) + 1 for line in block)
        if used + size > room:
            break
        used += size
        blocks += block
        shown += 1
    return details(
        f"Changed outputs ({number(len(changed))} schemas in {number(len(ranked))} documents, {shown} diffs shown)",
        counts + [""] + blocks,
    )


def report(old: dict[Key, Result], new: dict[Key, Result], diffs: list[Diff], max_diffs: int, budget: int) -> str:
    keys = sorted(old.keys() & new.keys())
    status_changes, output_changes = changes(old, new)
    crashes, resolved = crash_transitions(old, new)
    preparations = preparation_changes(old, new)
    schemas = [key for key in keys if key[1] != "-"]
    documents_count = len({document for document, _ in schemas})

    lines = [MARKER, "## `canonical-ecosystem` results", ""]
    if crashes:
        lines.append(f"❌ **{len(crashes)}** documents crash or time out that did not before.")
    parts = []
    if status_changes or output_changes:
        changed_documents = len({document for document, _ in status_changes + output_changes})
        parts.append(
            f"**{number(len(output_changes))}** outputs and **{number(len(status_changes))}** statuses changed "
            f"in **{number(changed_documents)}** documents"
        )
    if preparations:
        parts.append(f"**{number(len(preparations))}** documents changed whether they prepare")
    if resolved:
        parts.append(f"**{number(len(resolved))}** documents no longer crash or time out")
    if parts:
        lines.append(f"ℹ️ {'; '.join(parts)} ({number(len(schemas))} schemas checked).")
    elif not crashes:
        lines.append(
            f"✅ No canonical output changes across {number(len(schemas))} schemas from {number(documents_count)} documents."
        )
    lines.append("")

    old_counts = Counter(old[key].status for key in keys)
    new_counts = Counter(new[key].status for key in keys)
    lines += ["| | Old | New |", "|---|---:|---:|"]
    for status in sorted(old_counts.keys() | new_counts.keys()):
        lines.append(row(f"`{status}`", number(old_counts[status]), number(new_counts[status])))
    lines.append("")

    if crashes:
        crash_rows = [(code(document), f"`{result.status}`", result.detail) for document, result in crashes]
        lines += table("Crashes and timeouts", ("Document", "Status", "Detail"), crash_rows)
    if preparations:
        preparation_rows = [(code(document), f"`{before}`", f"`{after}`") for document, before, after in preparations]
        lines += table("Preparation changes", ("Document", "Old", "New"), preparation_rows)
    if resolved:
        resolved_rows = [(code(document), f"`{result.status}`", result.detail) for document, result in resolved]
        lines += table("No longer crash or time out", ("Document", "Was", "Detail"), resolved_rows)
    if status_changes:
        transitions = Counter((old[key].status, new[key].status) for key in status_changes)
        intro = (", ".join(f"`{a}` -> `{b}`: {count}" for (a, b), count in transitions.most_common()), "")
        status_rows = [(target(key), f"`{old[key].status}`", f"`{new[key].status}`") for key in status_changes]
        lines += table("Status changes", ("Schema", "Old", "New"), status_rows, intro)

    changed = status_changes + output_changes
    if changed:
        # Diffs fill whatever room the other sections leave.
        room = budget - len("\n".join(lines)) - SECTION_OVERHEAD_CHARS
        lines += changed_section(changed, diffs, max_diffs, room)
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    fetch_parser = commands.add_parser("fetch")
    fetch_parser.add_argument("--corpus", type=Path, required=True)
    run_parser = commands.add_parser("run")
    run_parser.add_argument("--binary", type=Path, required=True)
    run_parser.add_argument("--corpus", type=Path, required=True)
    run_parser.add_argument("--output", type=Path, required=True)
    report_parser = commands.add_parser("report")
    report_parser.add_argument("--old", type=Path, required=True)
    report_parser.add_argument("--new", type=Path, required=True)
    report_parser.add_argument("--old-binary", type=Path, required=True)
    report_parser.add_argument("--new-binary", type=Path, required=True)
    report_parser.add_argument("--corpus", type=Path, required=True)
    report_parser.add_argument("--output", type=Path, required=True, help="Report sized for a PR comment")
    report_parser.add_argument("--full-output", type=Path, required=True, help="Report with more diffs")
    args = parser.parse_args()

    if args.command == "fetch":
        fetch(args.corpus)
        return 0
    if args.command == "run":
        run(args.binary.resolve(), args.corpus.resolve(), args.output)
        return 0
    old, new = load(args.old), load(args.new)
    status_changes, output_changes = changes(old, new)
    binaries = (args.old_binary.resolve(), args.new_binary.resolve(), args.corpus.resolve())
    diffs = render_diffs(status_changes + output_changes, *binaries, FULL_DIFFS)
    args.output.write_text(report(old, new, diffs, COMMENT_DIFFS, COMMENT_CHARS), encoding="utf-8")
    args.full_output.write_text(report(old, new, diffs, FULL_DIFFS, FULL_REPORT_CHARS), encoding="utf-8")
    return 1 if crash_transitions(old, new)[0] else 0


if __name__ == "__main__":
    sys.exit(main())
