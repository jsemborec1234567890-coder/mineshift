"""
MINE//SHIFT: BEYOND — Independent Python Solver Oracle
Polyglot Lab / Independent Verification System
"""

import json
from typing import Dict, List, Set, Tuple, Optional


class BoardTopology:
    @staticmethod
    def square_neighbors(x: int, y: int, width: int, height: int) -> List[Tuple[int, int]]:
        deltas = [
            (-1, -1), (0, -1), (1, -1),
            (-1,  0),          (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ]
        res = []
        for dx, dy in deltas:
            nx, ny = x + dx, y + dy
            if 0 <= nx < width and 0 <= ny < height:
                res.append((nx, ny))
        return sorted(res)

    @staticmethod
    def hex_neighbors(x: int, y: int, radius: int) -> List[Tuple[int, int]]:
        """Axial hexagonal coordinates with radius bounds check."""
        span = radius * 2 + 1
        deltas = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)]
        res = []
        for dx, dy in deltas:
            nx, ny = x + dx, y + dy
            if 0 <= nx < span and 0 <= ny < span:
                # axial radius check: |dx|, |dy|, |dx+dy| <= radius
                ax_q = nx - radius
                ax_r = ny - radius
                if abs(ax_q) <= radius and abs(ax_r) <= radius and abs(ax_q + ax_r) <= radius:
                    res.append((nx, ny))
        return sorted(res)

    @staticmethod
    def square_ring2(x: int, y: int, width: int, height: int) -> List[Tuple[int, int]]:
        """Ring-2 neighbors: distance 2 in Chebyshev metric."""
        d1 = set(BoardTopology.square_neighbors(x, y, width, height))
        d1.add((x, y))
        res = []
        for dx in range(-2, 3):
            for dy in range(-2, 3):
                nx, ny = x + dx, y + dy
                if 0 <= nx < width and 0 <= ny < height:
                    if (nx, ny) not in d1:
                        res.append((nx, ny))
        return sorted(res)


class ConstraintSolver:
    def __init__(self, topology: str, width: int, height: int, radius: int = 0):
        self.topology = topology
        self.width = width
        self.height = height
        self.radius = radius

    def get_neighbors(self, x: int, y: int) -> List[Tuple[int, int]]:
        if self.topology == "square":
            return BoardTopology.square_neighbors(x, y, self.width, self.height)
        elif self.topology == "hex":
            return BoardTopology.hex_neighbors(x, y, self.radius)
        raise ValueError(f"Unknown topology: {self.topology}")

    def solve(self, revealed: List[Dict], flags: List[Tuple[int, int]]) -> Dict[str, List[Tuple[int, int]]]:
        """
        Independent constraint deduction algorithm:
        1. Direct constraint resolution (free safes / forced mines)
        2. Set difference / subset reduction (e.g. 1-2-1 patterns)
        """
        flag_set = set(tuple(f) for f in flags)
        revealed_map = {(r["x"], r["y"]): r for r in revealed}

        forced_mines: Set[Tuple[int, int]] = set()
        forced_safes: Set[Tuple[int, int]] = set()

        # Build equations: Set of covered neighbors -> remaining mine count
        equations: List[Tuple[Set[Tuple[int, int]], int]] = []

        for (rx, ry), rdata in revealed_map.items():
            clue = rdata.get("clue", 0)
            clue_type = rdata.get("clue_type", "standard")

            if clue_type == "ring2":
                r2_clue = rdata.get("ring2_clue", 0)
                # For ring2 clue 0, all ring2 neighbors must be safe!
                if r2_clue == 0:
                    r2_cells = BoardTopology.square_ring2(rx, ry, self.width, self.height)
                    for c in r2_cells:
                        if c not in revealed_map and c not in flag_set:
                            forced_safes.add(c)
                # Standard adjacent clue 0 means all adjacent cells safe
                if clue == 0:
                    for c in self.get_neighbors(rx, ry):
                        if c not in revealed_map and c not in flag_set:
                            forced_safes.add(c)
                continue

            nbrs = self.get_neighbors(rx, ry)
            covered_unknown: Set[Tuple[int, int]] = set()
            mines_found = 0

            for nx, ny in nbrs:
                if (nx, ny) in flag_set:
                    mines_found += 1
                elif (nx, ny) not in revealed_map:
                    covered_unknown.add((nx, ny))

            rem_clue = clue - mines_found
            if rem_clue == 0:
                for c in covered_unknown:
                    forced_safes.add(c)
            elif rem_clue == len(covered_unknown):
                for c in covered_unknown:
                    forced_mines.add(c)
            elif covered_unknown:
                equations.append((covered_unknown, rem_clue))

        # Iterative propagation and subset reduction
        changed = True
        while changed:
            changed = False

            # 1. Substitute discovered mines and safes into equations
            new_eqs = []
            for eq_set, eq_rem in equations:
                # Remove known mines
                known_m = eq_set.intersection(forced_mines)
                rem_set = eq_set - forced_mines - forced_safes
                rem_val = eq_rem - len(known_m)

                if rem_val == 0:
                    for c in rem_set:
                        if c not in forced_safes:
                            forced_safes.add(c)
                            changed = True
                elif rem_val == len(rem_set) and len(rem_set) > 0:
                    for c in rem_set:
                        if c not in forced_mines:
                            forced_mines.add(c)
                            changed = True
                elif len(rem_set) > 0 and rem_val > 0:
                    new_eqs.append((rem_set, rem_val))

            equations = new_eqs

            # 2. Subset reductions between pairs of equations
            for i in range(len(equations)):
                set_a, rem_a = equations[i]
                for j in range(len(equations)):
                    if i == j:
                        continue
                    set_b, rem_b = equations[j]
                    if set_a.issubset(set_b) and len(set_a) < len(set_b):
                        diff_set = set_b - set_a
                        diff_rem = rem_b - rem_a
                        if diff_rem == 0:
                            for c in diff_set:
                                if c not in forced_safes:
                                    forced_safes.add(c)
                                    changed = True
                        elif diff_rem == len(diff_set):
                            for c in diff_set:
                                if c not in forced_mines:
                                    forced_mines.add(c)
                                    changed = True

        return {
            "forced_mines": sorted(list(forced_mines)),
            "forced_safes": sorted(list(forced_safes)),
        }
