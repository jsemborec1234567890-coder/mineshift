/**
 * MINE//SHIFT: BEYOND — Portable Minimal C Reference Solver
 * Pure C11 zero-allocation reference implementation
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <assert.h>

typedef struct {
    int x;
    int y;
} Coord;

int c_square_neighbors(int x, int y, int w, int h, Coord out[8]) {
    int count = 0;
    for (int dy = -1; dy <= 1; ++dy) {
        for (int dx = -1; dx <= 1; ++dx) {
            if (dx == 0 && dy == 0) continue;
            int nx = x + dx;
            int ny = y + dy;
            if (nx >= 0 && nx < w && ny >= 0 && ny < h) {
                out[count++] = (Coord){nx, ny};
            }
        }
    }
    return count;
}

int c_hex_neighbors(int x, int y, int radius, Coord out[6]) {
    int span = radius * 2 + 1;
    int deltas[6][2] = {{1, 0}, {1, -1}, {0, -1}, {-1, 0}, {-1, 1}, {0, 1}};
    int count = 0;
    for (int i = 0; i < 6; ++i) {
        int nx = x + deltas[i][0];
        int ny = y + deltas[i][1];
        if (nx >= 0 && nx < span && ny >= 0 && ny < span) {
            int q = nx - radius;
            int r = ny - radius;
            if (abs(q) <= radius && abs(r) <= radius && abs(q + r) <= radius) {
                out[count++] = (Coord){nx, ny};
            }
        }
    }
    return count;
}

// Bitmask constraint solver for 3x2 board:
// Bit 0: (0, 1)
// Bit 1: (1, 1)
// Bit 2: (2, 1)
bool c_solve_121(int* forced_mines_mask, int* forced_safes_mask) {
    // Constraint 0: (0,0) with clue 1 over unknowns {(0,1), (1,1)} -> bits (1<<0) | (1<<1)
    // Constraint 1: (1,0) with clue 2 over unknowns {(0,1), (1,1), (2,1)} -> bits (1<<0) | (1<<1) | (1<<2)
    // Constraint 2: (2,0) with clue 1 over unknowns {(1,1), (2,1)} -> bits (1<<1) | (1<<2)
    
    // Test all 2^3 = 8 possible mine assignments (0 = safe, 1 = mine)
    int valid_count = 0;
    int valid_masks[8];

    for (int m = 0; m < 8; ++m) {
        int c0_mines = ((m >> 0) & 1) + ((m >> 1) & 1);
        int c1_mines = ((m >> 0) & 1) + ((m >> 1) & 1) + ((m >> 2) & 1);
        int c2_mines = ((m >> 1) & 1) + ((m >> 2) & 1);

        if (c0_mines == 1 && c1_mines == 2 && c2_mines == 1) {
            valid_masks[valid_count++] = m;
        }
    }

    // There should be exactly 1 valid assignment: m = 0b101 (mines at (0,1) and (2,1), safe at (1,1))
    if (valid_count != 1) return false;

    int mask = valid_masks[0];
    *forced_mines_mask = mask;
    *forced_safes_mask = (~mask) & 0x7;
    return true;
}

int main(void) {
    printf("[C REFERENCE] Running C11 Reference Solver Verification...\n");

    Coord sq[8];
    int sq_n = c_square_neighbors(1, 1, 3, 3, sq);
    assert(sq_n == 8);

    Coord hex[6];
    int hex_n = c_hex_neighbors(2, 2, 2, hex);
    assert(hex_n == 6);
    printf("  -> Topology check: Square = %d nbrs, Hex = %d nbrs (PASS)\n", sq_n, hex_n);

    int mines_mask = 0;
    int safes_mask = 0;
    bool ok = c_solve_121(&mines_mask, &safes_mask);
    assert(ok);
    assert((mines_mask & (1 << 0)) != 0); // (0,1) is mine
    assert((mines_mask & (1 << 2)) != 0); // (2,1) is mine
    assert((safes_mask & (1 << 1)) != 0); // (1,1) is safe

    printf("  -> Truth-table SAT enumeration: 1-2-1 unique model 0b%03x (PASS)\n", mines_mask);
    printf("[C REFERENCE] SUCCESS: 100%% verified.\n");
    return 0;
}
