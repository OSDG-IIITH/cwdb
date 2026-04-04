#!/usr/bin/env python3

import json
import os
import re

ROMAN = {"viii": "8", "vii": "7", "vi": "6", "iv": "4", "v": "5", "iii": "3", "ii": "2", "i": "1"}


def normname(s):
    s = s.lower()
    s = s.replace("&", "and")
    s = re.sub(r"\(.*?\)", "", s)
    s = s.replace("-", " ").replace("_", " ")
    words = s.split()
    words = [ROMAN.get(w, w) for w in words]
    return " ".join(words).strip()


def load(path):
    with open(path) as f:
        return json.load(f)


def escape(s):
    return s.replace("'", "''")


def merge(monsoon, spring):
    courses = {}

    for course in monsoon:
        key = normname(course["name"])
        courses[key] = {
            "code": course.get("code", ""),
            "name": course["name"],
            "aliases": [a.lower() for a in course.get("aliases", [])],
            "instructors": course.get("instructors", []),
            "semester": "monsoon",
        }

    for course in spring:
        key = normname(course["name"])
        if key in courses:
            existing = courses[key]
            existing["semester"] = "both"
            if not existing["code"] and course.get("code"):
                existing["code"] = course["code"]
            existing_aliases = set(existing["aliases"])
            for a in course.get("aliases", []):
                if a.lower() not in existing_aliases:
                    existing["aliases"].append(a.lower())
                    existing_aliases.add(a.lower())
            existing_instructors = set(existing["instructors"])
            for inst in course.get("instructors", []):
                if inst not in existing_instructors:
                    existing["instructors"].append(inst)
                    existing_instructors.add(inst)
        else:
            courses[key] = {
                "code": course.get("code", ""),
                "name": course["name"],
                "aliases": [a.lower() for a in course.get("aliases", [])],
                "instructors": course.get("instructors", []),
                "semester": "spring",
            }

    return list(courses.values())


def to_pg_array(items):
    if not items:
        return "'{}'"
    escaped = ",".join(f'"{escape(i)}"' for i in items)
    return f"'{{{escaped}}}'"


def main():
    scriptdir = os.path.dirname(os.path.abspath(__file__))
    monsoon = load(os.path.join(scriptdir, "monsoon.json"))
    spring = load(os.path.join(scriptdir, "spring.json"))
    courses = merge(monsoon, spring)

    print(f"-- {len(courses)} courses")
    for c in courses:
        name = escape(c["name"].replace("&", "and"))
        code = escape(c["code"])
        sem = escape(c["semester"])
        aliases = to_pg_array(c["aliases"])
        instructors = to_pg_array(c["instructors"])
        print(
            f"INSERT INTO courses (code, name, aliases, instructors, semester) "
            f"VALUES ('{code}', '{name}', {aliases}, {instructors}, '{sem}') "
            f"ON CONFLICT (name) DO UPDATE SET "
            f"code = EXCLUDED.code, aliases = EXCLUDED.aliases, "
            f"instructors = EXCLUDED.instructors, semester = EXCLUDED.semester;"
        )


if __name__ == "__main__":
    main()
