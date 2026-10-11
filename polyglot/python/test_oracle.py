"""
Test runner for Python Solver Oracle against Polyglot contract fixtures.
"""

import json
import os
import sys
from solver_oracle import ConstraintSolver


def test_fixture(fixture_path: str):
    print(f"[TEST] Testing fixture: {os.path.basename(fixture_path)}")
    with open(fixture_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    solver = ConstraintSolver(
        topology=data["topology"],
        width=data.get("width", 0),
        height=data.get("height", 0),
        radius=data.get("radius", 0),
    )

    res = solver.solve(data.get("visible_revealed", []), data.get("visible_flags", []))
    exp = data["expected_deductions"]

    exp_mines = [tuple(m) for m in exp.get("forced_mines", [])]
    exp_safes = [tuple(s) for s in exp.get("forced_safes", [])]

    # Verify that every expected mine and safe is correctly deduced
    for m in exp_mines:
        assert m in res["forced_mines"], f"Expected forced mine {m} missing from deductions {res['forced_mines']}"
    for s in exp_safes:
        assert s in res["forced_safes"], f"Expected forced safe {s} missing from deductions {res['forced_safes']}"

    print(f"  -> SUCCESS! Deduced mines: {res['forced_mines']}, safes: {res['forced_safes']}")


def main():
    fixtures_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "fixtures"))
    fixtures = [
        "fixture_01_square_classic_121.json",
        "fixture_02_square_ring2.json",
        "fixture_03_hex_radius2.json",
    ]

    passed = 0
    for fix in fixtures:
        fpath = os.path.join(fixtures_dir, fix)
        if os.path.exists(fpath):
            test_fixture(fpath)
            passed += 1

    print(f"\n[PYTHON ORACLE] All {passed} fixtures verified successfully against contract v1!")


if __name__ == "__main__":
    main()
