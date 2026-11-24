#!/usr/bin/env python3
"""Regenerate the README's feature grid and task table from roadmap.json.

Usage (repo root):
    python .github/scripts/readme_roadmap.py          # rewrite README.md
    python .github/scripts/readme_roadmap.py --check  # exit 1 if README is out of date

Only text between the `<!-- roadmap:NAME:start -->` / `<!-- roadmap:NAME:end -->`
markers is replaced, so the rest of the README stays hand-written.
Standard library only.
"""

import json
import re
import sys
from pathlib import Path
from urllib.parse import quote

ROOT = Path(__file__).resolve().parents[2]
README = ROOT / "README.md"
ROADMAP = ROOT / "roadmap.json"

EMOJI = {
    "organize": "🗂️",
    "optimize": "🚀",
    "convert-to": "📥",
    "convert-from": "📤",
    "edit": "✏️",
    "security": "🛡️",
    "intelligence": "🧠",
    "scan": "📷",
}
MARK = {"done": "✅ ", "in_progress": "🚧 ", "pending": ""}


def status(task):
    s = task.get("status", "pending")
    return s if s in MARK else "pending"


def features(modules):
    """Two-column HTML grid of modules with checkboxes."""
    out = ["<table>"]
    for i in range(0, len(modules), 2):
        out.append("<tr>")
        for m in modules[i : i + 2]:
            width = ' width="50%"' if i == 0 else ""
            items = "\n".join(
                f"- [{'x' if status(t) == 'done' else ' '}] {t['name']}"
                + (" 🚧" if status(t) == "in_progress" else "")
                for t in m["tasks"]
            )
            out.append(
                f'<td{width} valign="top">\n\n### {EMOJI.get(m["key"], "📄")} {m["name"]}\n{items}\n\n</td>'
            )
        out.append("</tr>")
    out.append("</table>")
    return "\n".join(out)


def tasks(modules):
    """Progress badge plus one table row per run of tasks with the same assignee."""
    every = [t for m in modules for t in m["tasks"]]
    done = sum(status(t) == "done" for t in every)
    badge = quote(f"{done} of {len(every)} done", safe="")
    lines = [
        f"![Progress](https://img.shields.io/badge/progress-{badge}-DE0036?style=flat-square)",
        "",
        "| Category | Tools | Assignee | Status |",
        "| :--- | :--- | :--- | :---: |",
    ]
    for m in modules:
        groups = []
        for t in m["tasks"]:
            who = t.get("assignee") or None
            if groups and groups[-1][0] == who:
                groups[-1][1].append(t)
            else:
                groups.append((who, [t]))
        for who, group in groups:
            names = ", ".join(MARK[status(t)] + t["name"] for t in group)
            finished = sum(status(t) == "done" for t in group)
            if finished == len(group):
                state = "✅ Done"
            elif finished or any(status(t) == "in_progress" for t in group):
                state = f"🚧 {finished}/{len(group)}"
            else:
                state = "⏳ Pending"
            lines.append(f"| {m['name']} | {names} | {who or '_Unassigned_'} | {state} |")
    return "\n".join(lines)


def replace(text, name, body):
    start, end = f"<!-- roadmap:{name}:start -->", f"<!-- roadmap:{name}:end -->"
    pattern = re.compile(re.escape(start) + ".*?" + re.escape(end), re.S)
    if not pattern.search(text):
        sys.exit(f"README.md is missing the roadmap:{name} markers")
    return pattern.sub(lambda _: f"{start}\n{body}\n{end}", text)


def main():
    modules = json.loads(ROADMAP.read_text(encoding="utf-8"))["modules"]
    old = README.read_text(encoding="utf-8")
    new = replace(old, "features", features(modules))
    new = replace(new, "tasks", tasks(modules))
    if "--check" in sys.argv:
        if new != old:
            sys.exit("README.md is out of date: run python .github/scripts/readme_roadmap.py")
        return
    if new != old:
        README.write_text(new, encoding="utf-8", newline="\n")
        print("README.md updated")
    else:
        print("README.md already up to date")


if __name__ == "__main__":
    main()
