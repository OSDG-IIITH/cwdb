#!/usr/bin/env python3
import json
import re
from pathlib import Path

SKIP = {"the", "and", "a", "an", "of", "in", "to", "for", "with", "on", "at", "by", "from", "or", "its"}

def make_alias(name):
    words = name.split()
    letters = []
    for word in words:
        clean = re.sub(r"[^a-zA-Z0-9]", "", word)
        if clean and clean.lower() not in SKIP:
            letters.append(clean[0].upper())
    return "".join(letters)

def ask(name, suggestion):
    print(f"\n  {name}")
    print(f"  suggested: {suggestion}")
    while True:
        resp = input("  [enter]=confirm  [r]=reject  [type]=alternate(s, space-separated): ").strip()
        if resp == "":
            return [suggestion]
        elif resp.lower() == "r":
            return []
        else:
            return [a.strip() for a in resp.split() if a.strip()]

def process(path):
    data = json.loads(Path(path).read_text())
    changed = 0
    for course in data:
        if not course.get("aliases"):
            aliases = ask(course["name"], make_alias(course["name"]))
            if aliases:
                course["aliases"] = aliases
                changed += 1
    Path(path).write_text(json.dumps(data, indent=2) + "\n")
    print(f"\n{changed} aliases added to {path}\n")

for f in ["monsoon.json", "spring.json"]:
    path = Path(__file__).parent / f
    print(f"\n=== {f} ===")
    process(path)
