# hive-state

The hive's state for this project ([Bee-ruche/hive](https://github.com/Bee-ruche/hive), `docs/hosting.md`):

- `ledger.jsonl`: the append-only record of what the hive dealt and what the judge answered, one JSON object per line.
- `feed.json`: what workers read: the tasks, their status, and the task each member holds.
- `accounts.json`, `ignored.json`, `stats.json`: what the hive keeps between runs.

Only the hive's own run writes this branch. Nothing here is edited by hand.
