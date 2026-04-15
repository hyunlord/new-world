#!/usr/bin/env python3
"""Interactive controller for WorldSim harness visual verification.

Connects to Godot's TCP command server and executes test scenarios.
Protocol: newline-delimited JSON over TCP (port 9223).
No external dependencies — uses only Python stdlib.
"""

import argparse
import json
import os
import re
import socket
import sys
import time


def send_command(sock: socket.socket, cmd: dict) -> dict:
    """Send a JSON command and receive the response."""
    msg = json.dumps(cmd) + "\n"
    sock.sendall(msg.encode("utf-8"))
    buf = b""
    while b"\n" not in buf:
        chunk = sock.recv(4096)
        if not chunk:
            raise ConnectionError("Server disconnected")
        buf += chunk
    line = buf.split(b"\n")[0]
    return json.loads(line.decode("utf-8"))


def parse_scenarios(text: str) -> list:
    """Parse interactive scenarios from markdown format."""
    scenarios = []
    current = None

    for line in text.split("\n"):
        if line.startswith("### Scenario"):
            if current:
                scenarios.append(current)
            name = line.replace("### ", "").strip()
            current = {"name": name, "steps": [], "expected": ""}
        elif current is not None and line.startswith("Expected:"):
            current["expected"] = line.replace("Expected:", "").strip()
        elif current is not None and re.match(r"^\d+\.", line.strip()):
            current["steps"].append(line.strip())

    if current:
        scenarios.append(current)

    return scenarios


def execute_scenario(sock: socket.socket, scenario: dict, evidence_dir: str) -> dict:
    """Execute a single test scenario and return results."""
    name = scenario["name"]
    safe_name = re.sub(r"[^a-zA-Z0-9]", "_", name)[:30]
    result = {"name": name, "steps_log": [], "result": "PASS", "detail": ""}
    screenshot_idx = 0

    for step in scenario["steps"]:
        step_lower = step.lower()

        # Zoom commands
        if "zoom" in step_lower and ("level" in step_lower or re.search(r"zoom\s+\d", step_lower)):
            match = re.search(r"(\d+\.?\d*)", step)
            if match:
                level = float(match.group(1))
                resp = send_command(sock, {"action": "zoom", "level": level})
                result["steps_log"].append(f"zoom {level}: {resp}")

        # Click commands
        elif any(kw in step_lower for kw in ["클릭", "click"]):
            # Check for explicit coordinates
            coord_match = re.search(r"\((\d+),\s*(\d+)\)", step)
            if coord_match:
                x, y = float(coord_match.group(1)), float(coord_match.group(2))
            else:
                # Default: click center of viewport
                state = send_command(sock, {"action": "get_state"})
                vp = state.get("viewport_size", [1152, 648])
                x, y = vp[0] / 2, vp[1] / 2
            resp = send_command(sock, {"action": "click", "x": x, "y": y})
            result["steps_log"].append(f"click ({x}, {y}): {resp}")

        # Screenshot/capture commands
        elif any(kw in step_lower for kw in ["스크린샷", "screenshot", "캡처"]):
            screenshot_idx += 1
            label = f"interactive_{safe_name}_{screenshot_idx}"
            resp = send_command(sock, {"action": "screenshot", "label": label})
            result["steps_log"].append(f"screenshot {label}: {resp}")

        # Wait commands
        elif any(kw in step_lower for kw in ["대기", "wait", "프레임"]):
            match = re.search(r"(\d+)", step)
            count = int(match.group(1)) if match else 5
            if "tick" in step_lower:
                resp = send_command(sock, {"action": "wait_ticks", "count": count})
            else:
                resp = send_command(sock, {"action": "wait_frames", "count": count})
            result["steps_log"].append(f"wait {count}: {resp}")

        # Verify/check commands — take a verification screenshot
        elif any(kw in step_lower for kw in ["확인", "verify", "check", "검증"]):
            screenshot_idx += 1
            label = f"verify_{safe_name}_{screenshot_idx}"
            resp = send_command(sock, {"action": "screenshot", "label": label})
            result["steps_log"].append(f"verify screenshot {label}: {resp}")

        else:
            result["steps_log"].append(f"skipped (unrecognized): {step}")

    return result


def main():
    parser = argparse.ArgumentParser(description="WorldSim interactive test controller")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=9223)
    parser.add_argument("--evidence-dir", required=True)
    parser.add_argument("--scenarios", required=True, help="Path to scenarios markdown file")
    args = parser.parse_args()

    # Read and parse scenarios
    with open(args.scenarios) as f:
        scenario_text = f.read()

    scenarios = parse_scenarios(scenario_text)
    if not scenarios:
        print("No scenarios found in input file")
        return

    print(f"Parsed {len(scenarios)} scenario(s)")

    # Connect to Godot command server
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(10.0)
    try:
        sock.connect((args.host, args.port))
    except ConnectionRefusedError:
        print(f"ERROR: Could not connect to {args.host}:{args.port}")
        sys.exit(1)
    print(f"Connected to {args.host}:{args.port}")

    # Get initial state
    state = send_command(sock, {"action": "get_state"})
    print(f"Initial state: {state}")

    # Execute scenarios
    all_results = []
    for scenario in scenarios:
        print(f"\n--- Executing: {scenario['name']} ---")
        try:
            result = execute_scenario(sock, scenario, args.evidence_dir)
        except Exception as e:
            result = {
                "name": scenario["name"],
                "steps_log": [f"ERROR: {e}"],
                "result": "FAIL",
                "detail": str(e),
            }
        all_results.append(result)
        print(f"Result: {result['result']}")
        for log_line in result["steps_log"]:
            print(f"  {log_line}")

    # Write results
    output_path = os.path.join(args.evidence_dir, "interactive_results.txt")
    with open(output_path, "w") as f:
        for r in all_results:
            f.write(f"SCENARIO: {r['name']}\n")
            f.write(f"STEPS: {'; '.join(r['steps_log'])}\n")
            f.write(f"RESULT: {r['result']}\n")
            f.write(f"DETAIL: {r.get('detail', '')}\n\n")

    print(f"\nResults written to: {output_path}")

    # Tell Godot to quit
    try:
        send_command(sock, {"action": "quit"})
    except Exception:
        pass
    sock.close()


if __name__ == "__main__":
    main()
