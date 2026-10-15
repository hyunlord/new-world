from __future__ import annotations

import hashlib
import re
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from evaluate_perf_pass17 import evaluate as evaluate_performance
from evaluate_pass17_data import InputError, Verdict
from evaluate_pass17_read import JsonValue, array, integer, mapping, number, read_json, required


@dataclass(frozen=True, slots=True)
class Evidence:
    verdict: Verdict
    source: str | None
    sha256: str | None
    payload: Mapping[str, JsonValue] | None
    error: str | None = None


def digest(path: Path) -> str:
    try:
        with path.open("rb") as stream:
            return hashlib.file_digest(stream, "sha256").hexdigest()
    except OSError as error:
        raise InputError(str(path), str(error)) from error


def boolean(value: JsonValue, location: str) -> bool:
    if type(value) is bool:
        return value
    raise InputError(location, "expected a boolean")


def text_value(value: JsonValue, location: str) -> str:
    if isinstance(value, str) and value:
        return value
    raise InputError(location, "expected a nonempty string")


def verify_header(payload: Mapping[str, JsonValue]) -> Verdict:
    if integer(required(payload, "schema_version"), "schema_version") != 1 or required(payload, "stage") != "a":
        raise InputError("evidence", "requires schema_version 1 and stage a")
    verdict = required(payload, "verdict")
    errors = array(required(payload, "errors"), "errors")
    if verdict == "ERROR":
        if not errors or required(payload, "passed") is not None:
            raise InputError("evidence", "ERROR requires errors and passed=null")
        return "ERROR"
    if verdict not in ("PASS", "FAIL") or errors:
        raise InputError("evidence", "invalid verdict or errors accompanying a measured result")
    if boolean(required(payload, "passed"), "passed") != (verdict == "PASS"):
        raise InputError("evidence", "verdict disagrees with passed")
    return "PASS" if verdict == "PASS" else "FAIL"


def verify_identity(payload: Mapping[str, JsonValue], source: Path) -> None:
    checks = mapping(required(payload, "checks"), "checks")
    names = {"I1", "I2", "I3", "I4", "v19_pins", "v20_pins"}
    if set(checks) != names:
        raise InputError("identity checks", "requires I1-I4 and v19_pins/v20_pins exactly")
    outcomes = []
    for name, value in checks.items():
        check = mapping(value, name)
        outcomes.append(boolean(required(check, "passed"), name))
        artifacts = array(required(check, "evidence"), name)
        if not artifacts:
            raise InputError(name, "no hash-bound evidence")
        for value in artifacts:
            artifact = mapping(value, name)
            path = Path(text_value(required(artifact, "path"), name))
            if not path.is_absolute():
                path = source.parent / path
            expected = text_value(required(artifact, "sha256"), name)
            if not re.fullmatch(r"[0-9a-f]{64}", expected) or digest(path) != expected:
                raise InputError(name, "evidence SHA256 mismatch")
    if all(outcomes) != required(payload, "passed"):
        raise InputError("identity", "aggregate verdict disagrees with individual checks")


def verify_performance(payload: Mapping[str, JsonValue]) -> None:
    checks = mapping(required(payload, "checks"), "checks")
    limits = {"mean_relative": (0.15, "<="), "mean_absolute": (4.0, "<="),
        "p999_relative": (0.35, "<="), "slope": (1.7, "<")}
    if set(checks) != set(limits):
        raise InputError("performance", "missing or unexpected timing checks")
    outcomes = []
    for name, (limit, operator) in limits.items():
        check = mapping(checks[name], name)
        value = number(required(check, "value"), name)
        if number(required(check, "limit"), name) != limit or required(check, "operator") != operator:
            raise InputError(name, "performance threshold differs from pre-registration")
        passed = value < limit if operator == "<" else value <= limit
        if boolean(required(check, "passed"), name) != passed:
            raise InputError(name, "numeric timing value disagrees with verdict")
        outcomes.append(passed)
    if all(outcomes) != required(payload, "passed"):
        raise InputError("performance", "aggregate verdict disagrees with timing checks")
    for field in ("medians", "slopes", "quick", "provenance"):
        if not mapping(required(payload, field), field):
            raise InputError(field, "missing performance provenance or measurements")
    repeats = mapping(required(payload, "repeats"), "repeats")
    if any(len(array(required(repeats, side), side)) != 3 for side in ("old", "new")):
        raise InputError("repeats", "requires three old and three new measurements")
    provenance = mapping(required(payload, "provenance"), "provenance")
    directory = Path(text_value(required(provenance, "input_directory"), "input_directory"))
    verified = evaluate_performance(directory, "a")
    if verified["verdict"] == "ERROR":
        raise InputError("performance", f"raw evidence verification failed: {verified['errors']}")
    for field in ("checks", "repeats", "medians", "slopes", "quick"):
        if required(payload, field) != required(verified, field):
            raise InputError(field, "stored performance result differs from verified raw evidence")
    if required(provenance, "manifest_sha256") != required(mapping(verified["provenance"], "provenance"), "manifest_sha256"):
        raise InputError("provenance", "performance manifest changed")


def load_evidence(path: Path | None, kind: Literal["identity", "performance"], tuning: bool) -> Evidence:
    if path is None:
        return Evidence("PENDING" if tuning else "ERROR", None, None, None, f"{kind} evidence was not supplied")
    try:
        payload = mapping(read_json(path), kind)
        sha = digest(path)
        verdict = verify_header(payload)
        if verdict == "ERROR":
            return Evidence(verdict, str(path), sha, payload, "producer reported ERROR")
        if kind == "identity":
            verify_identity(payload, path)
        else:
            verify_performance(payload)
        return Evidence(verdict, str(path), sha, payload)
    except InputError as error:
        return Evidence("ERROR", str(path), None, None, str(error))
