#!/usr/bin/env python3
"""Does the hive's own loader take the project's files?

  python3 bee/proofs/loads.py <the hive's tree> <the project's tree>

It imports the hive's `tools/hive/hive.py` from the tree it is given and asks it, without network and without
running anything of the project: the settings (hive/project.json), the tasks (hive/tasks.jsonl, empty), a task as
the project's cutter would write one, and the outline of the program's files (what the cutter reads of the code).

A hive that does not hold the project's kind of judge yet (`relay`) refuses the settings for that one word: the
script says so, then asks again with that word alone taken out, so that everything else is shown to load.
It ends with 0 when everything but, at most, the kind's name was taken.
"""

import json
import os
import sys
import tempfile


def main(arguments):
    if len(arguments) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    tree, project = arguments
    sys.path.insert(0, os.path.join(tree, "tools", "hive"))
    import hive

    path = os.path.join(project, "hive", "project.json")
    with open(path, encoding="utf-8") as f:
        raw = json.load(f)
    print("the hive's kinds of judge:", ", ".join(sorted(hive.JUDGES)))
    try:
        config = hive.load_config(path, raw["repo"])
        print(f"settings: loaded as they are (judge.kind {config['judge']['kind']})")
    except hive.HiveError as refused:
        print("settings as they are: REFUSED:", refused)
        if raw["judge"].get("kind") in hive.JUDGES:
            return 1
        less = dict(raw, judge={key: value for key, value in raw["judge"].items() if key != "kind"})
        with tempfile.TemporaryDirectory() as folder:
            other = os.path.join(folder, "project.json")
            with open(other, "w", encoding="utf-8") as f:
                json.dump(less, f)
            config = hive.load_config(other, raw["repo"])
        print("settings with judge.kind alone taken out: loaded (every other setting is one this hive takes)")
    print("  members:", config["members"]["only"], " backoff_hours:", config["backoff_hours"], " proofs taken:", config["judge"]["confirm"])
    print("  protected:", config["protected"])

    tasks = hive.load_tasks(os.path.join(project, "hive", "tasks.jsonl"))
    hive.check_goals(config, tasks)
    hive.check_links(tasks)
    print(f"tasks: loaded, {len(tasks)} task(s)")

    name = config["project"]
    goal = {"kind": "metric", "name": "gain_total_bytes", "min": 0.01, "and": [{"name": "sure_total_bytes", "min": 1}]}
    task = {"id": f"{name}-0001", "title": "a task as the cutter would write one", "files": ["src/mixer.rs"], "goal": goal}
    hive.check_goals(config, hive.tasks_of([json.dumps(task)], "a proposed task"))
    print("a task that asks gain_total_bytes >= 0.01 with its proof sure_total_bytes: taken")
    for what, wrong in (("a gain without its proof", {"kind": "metric", "name": "gain_total_bytes", "min": 0.01}),
                        ("a figure the project does not keep", {"kind": "metric", "name": "gain_speed", "min": 0.01, "and": goal["and"]})):
        try:
            hive.check_goals(config, {task["id"]: dict(task, goal=wrong)})
            print(f"{what}: TAKEN, which it must not be")
            return 1
        except hive.HiveError:
            print(f"{what}: refused, as it must be")

    print("the outline of the program (hive.outline_of: functions, first line, lines; no comment, no string):")
    for root, folders, names in os.walk(os.path.join(project, "src")):
        folders.sort()
        for file in sorted(names):
            relative = os.path.relpath(os.path.join(root, file), project).replace(os.sep, "/")
            with open(os.path.join(root, file), encoding="utf-8") as f:
                text = f.read()
            functions, closed = hive.outline_of(text, os.path.splitext(file)[1])
            print(f"  {relative}: {text.count(chr(10))} lines, closed={closed}, protected={hive.protected(relative, config['protected'])}, "
                  + ", ".join(f"{function[0]}@{function[1]}+{function[2]}" for function in functions))
            if not closed or not functions:
                return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
