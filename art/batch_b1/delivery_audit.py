# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy", "typer"]
# ///
# How to run: python3 -m art.batch_b1.delivery audit COMMIT --evidence /absolute/B1
"""Run the unchanged pinned importer against committed candidate blobs."""

import subprocess
import sys
from pathlib import Path
from typing import Final

# Keep the historical importer's untyped API in its own process. No source edits,
# network access, game writes, or substitutes for GitReader are involved.
RUNNER: Final = r"""
import hashlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

root, evidence, commit = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
def blob(revision, path):
    return subprocess.check_output(["git", "-C", str(root), "show", f"{revision}:{path}"])
commit = subprocess.check_output(["git", "-C", str(root), "rev-parse", "--verify", f"{commit}^{{commit}}"], text=True).strip()
catalog = json.loads(blob(commit, "art/batch_b1/CATALOG.json"))
reference = catalog["reference_game_commit"]
script = evidence / "records/reference_art_import.py"
config_path = evidence / "records/reference_art_import.json"
assert script.read_bytes() == blob(reference, "tools/v8/art_import.py"), "pinned importer changed"
assert config_path.read_bytes() == blob(reference, "tools/v8/art_import.json"), "pinned config changed"
spec = importlib.util.spec_from_file_location("pinned_b1_importer", script)
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)
module.ROOT = root
config = json.loads(config_path.read_bytes())
fragment = json.loads(blob(commit, "art/batch_b1/drafts/people.fragment.json"))
assert set(fragment) == {"people", "look_grid"}
assert fragment["look_grid"] == catalog["proposed_look_grid"]
assert fragment["look_grid"]["fallbacks"] == config["look_grid"]["fallbacks"]
original = json.loads(config_path.read_bytes())
config.update(fragment)
config["source_commit"] = commit
assert all(config[k] == v for k, v in original.items() if k not in ("people", "look_grid", "source_commit"))
assert all(config["people"][k] == v for k, v in original["people"].items() if k not in ("looks", "frame_source", "portrait_source"))
assert config["people"]["frame_source"] == "art/character/batch_b1/{look}/{facing}/{pose}.png"
assert config["people"]["portrait_source"] == "art/portrait/batch_b1/{look}/{variant}.png"
git = module.GitReader(commit)
try:
    rows, extra = module.plan(config, git)
finally:
    git.close()
logical = [f"{a}_{s}_{t}_{h}" for a in config["look_grid"]["ages"] for s in config["look_grid"]["sexes"] for t in config["look_grid"]["tops"] for h in config["look_grid"]["hairs"]]
actual = [look for look in logical if look not in config["look_grid"]["fallbacks"]]
assert len(logical) == 120 and len(actual) == 118
assert config["people"]["looks"] == actual
assert [row["id"] for row in extra["looks"]] == logical
assert [row["index"] for row in extra["looks"]] == list(range(120))
for row in extra["looks"]:
    assert row["frames"] == config["look_grid"]["fallbacks"].get(row["id"], row["id"])
inventory = json.loads((evidence / "records/delivery_inventory.json").read_bytes())
expected = {entry["source"]: entry for entry in inventory["files"]}
people = [entry["row"] for entry in rows if entry["row"]["role"] in ("people_frame", "portrait")]
assert len(people) == len(expected) == 1888
assert sum(row["role"] == "people_frame" for row in people) == 1180
assert sum(row["role"] == "portrait" for row in people) == 708
assert {row["source"] for row in people} == set(expected)
for row in people:
    assert row["sha256"] == expected[row["source"]]["sha256"], row["source"]
    assert row["role"] == expected[row["source"]]["role"]
    suffix = row["source"].split("/batch_b1/", 1)[1]
    prefix = "people/frames/" if row["role"] == "people_frame" else "people/portraits/"
    assert row["path"] == prefix + suffix
for entry in inventory["baseline_copies"]:
    source = Path(entry["source"]).relative_to(root).as_posix()
    dest = Path(entry["destination"]).relative_to(root).as_posix()
    old = blob(catalog["baseline_commit"], source)
    assert hashlib.sha256(old).hexdigest() == entry["sha256"]
    assert blob(commit, source) == old == blob(commit, dest), source
assert len(inventory["baseline_copies"]) == 544
paths = [entry["dest"] for entry in rows]
assert len(paths) == len(set(paths))
assert all(not Path(path).is_absolute() and ".." not in Path(path).parts for path in paths)
total = sum(len(entry["data"]) for entry in rows)
assert total <= config["budget_bytes"], (total, config["budget_bytes"])
report = {"status": "PASS_unapplied_plan_only", "commit": commit,
          "reference_game_commit": reference, "importer_sha256": hashlib.sha256(script.read_bytes()).hexdigest(),
          "full_config_sections_preserved": True, "logical_looks": 120, "actual_looks": 118,
          "people_frames": 1180, "portraits": 708, "baseline_pngs_unchanged": 544,
          "all_import_rows": len(rows), "total_bytes": total, "budget_bytes": config["budget_bytes"],
          "pending": inventory["pending"]}
(evidence / "records/delivery_audit.json").write_text(json.dumps(report, indent=2) + "\n")
(evidence / "records/proposed_full_art_import.json").write_text(json.dumps(config, indent=2) + "\n")
print(json.dumps(report))
"""


def audit(commit: str, root: Path, evidence: Path) -> None:
    """Fail on missing committed sources, hash drift, path/index drift or budget."""
    _ = subprocess.run(
        [sys.executable, "-c", RUNNER, str(root), str(evidence), commit], check=True
    )
