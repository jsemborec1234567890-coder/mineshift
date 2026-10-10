import json
import sys

def solve(board_json):
    # This is a very simple Python implementation matching the Rust logic solver.
    board = json.loads(board_json)
    width = board['width']
    height = board['height']
    geometry = board['geometry']
    cells = board['cells'] # list of dicts: {"x": x, "y": y, "state": "Revealed", "is_mine": False, "adjacent_mines": 0}

    # Map for easy lookup
    cell_map = {(c['x'], c['y']): c for c in cells}
    
    def neighbors(x, y):
        if geometry == "Square":
            deltas = [
                (-1, -1), (0, -1), (1, -1),
                (-1,  0),          (1,  0),
                (-1,  1), (0,  1), (1,  1),
            ]
        else: # Hex
            deltas = [
                (1, 0), (1, -1), (0, -1),
                (-1, 0), (-1, 1), (0, 1),
            ]
        res = []
        for dx, dy in deltas:
            nx, ny = x + dx, y + dy
            if 0 <= nx < width and 0 <= ny < height:
                if (nx, ny) in cell_map:
                    res.append((nx, ny))
        return res

    constraints = []
    
    for c in cells:
        if c['state'] == "Revealed" and not c['is_mine']:
            hidden = set()
            flagged = 0
            for nx, ny in neighbors(c['x'], c['y']):
                n_c = cell_map[(nx, ny)]
                if n_c['state'] == "Covered":
                    hidden.add((nx, ny))
                elif n_c['state'] == "Flagged":
                    flagged += 1
            if hidden:
                rem_mines = max(0, c['adjacent_mines'] - flagged)
                constraints.append({"cells": hidden, "mines": rem_mines})
                
    deductions = []
    
    # 1. Direct deductions
    for cons in constraints:
        if cons['mines'] == 0:
            for cell in cons['cells']:
                deductions.append({"type": "Safe", "x": cell[0], "y": cell[1]})
        elif len(cons['cells']) == cons['mines']:
            for cell in cons['cells']:
                deductions.append({"type": "Mine", "x": cell[0], "y": cell[1]})
                
    if deductions:
        # unique
        unique_deductions = []
        seen = set()
        for d in deductions:
            k = (d['type'], d['x'], d['y'])
            if k not in seen:
                seen.add(k)
                unique_deductions.append(d)
        return unique_deductions
        
    # 2. Subset deductions
    for i, c_a in enumerate(constraints):
        for j, c_b in enumerate(constraints):
            if i != j and c_a['cells'].issubset(c_b['cells']) and len(c_a['cells']) < len(c_b['cells']):
                diff = c_b['cells'] - c_a['cells']
                diff_mines = max(0, c_b['mines'] - c_a['mines'])
                if diff_mines == 0:
                    for cell in diff:
                        deductions.append({"type": "Safe", "x": cell[0], "y": cell[1]})
                elif len(diff) == diff_mines:
                    for cell in diff:
                        deductions.append({"type": "Mine", "x": cell[0], "y": cell[1]})

    unique_deductions = []
    seen = set()
    for d in deductions:
        k = (d['type'], d['x'], d['y'])
        if k not in seen:
            seen.add(k)
            unique_deductions.append(d)
    return unique_deductions

if __name__ == "__main__":
    if len(sys.argv) > 1:
        with open(sys.argv[1], 'r') as f:
            print(json.dumps(solve(f.read())))
    else:
        print(json.dumps(solve(sys.stdin.read())))
