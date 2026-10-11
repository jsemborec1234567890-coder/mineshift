//! MINE//SHIFT: BEYOND — Zig Bitset Topology & Deduction Solver
//! Native Zig 0.17 implementation

const std = @import("std");
const assert = std.debug.assert;

pub const Coord = struct {
    x: usize,
    y: usize,
};

pub fn countSquareNeighbors(x: usize, y: usize, w: usize, h: usize) usize {
    var count: usize = 0;
    const xi: isize = @intCast(x);
    const yi: isize = @intCast(y);
    const wi: isize = @intCast(w);
    const hi: isize = @intCast(h);

    var dy: isize = -1;
    while (dy <= 1) : (dy += 1) {
        var dx: isize = -1;
        while (dx <= 1) : (dx += 1) {
            if (dx == 0 and dy == 0) continue;
            const nx = xi + dx;
            const ny = yi + dy;
            if (nx >= 0 and nx < wi and ny >= 0 and ny < hi) {
                count += 1;
            }
        }
    }
    return count;
}

pub fn countHexNeighbors(x: usize, y: usize, radius: usize) usize {
    const span: isize = @intCast(radius * 2 + 1);
    const r: isize = @intCast(radius);
    const xi: isize = @intCast(x);
    const yi: isize = @intCast(y);

    const deltas = [_][2]isize{
        .{ 1, 0 },
        .{ 1, -1 },
        .{ 0, -1 },
        .{ -1, 0 },
        .{ -1, 1 },
        .{ 0, 1 },
    };

    var count: usize = 0;
    for (deltas) |d| {
        const nx = xi + d[0];
        const ny = yi + d[1];
        if (nx >= 0 and nx < span and ny >= 0 and ny < span) {
            const q = nx - r;
            const ax_r = ny - r;
            const abs_q = if (q < 0) -q else q;
            const abs_r = if (ax_r < 0) -ax_r else ax_r;
            const sum_qr = q + ax_r;
            const abs_sum = if (sum_qr < 0) -sum_qr else sum_qr;
            if (abs_q <= r and abs_r <= r and abs_sum <= r) {
                count += 1;
            }
        }
    }
    return count;
}

pub fn main() void {
    std.debug.print("[ZIG SOLVER] Running Zig Bitset Topology Verification...\n", .{});

    // 1. Topology counts
    const sq_n = countSquareNeighbors(1, 1, 3, 3);
    assert(sq_n == 8);
    const hex_n = countHexNeighbors(2, 2, 2);
    assert(hex_n == 6);
    std.debug.print("  -> Square 3x3 center = {d} neighbors (PASS)\n", .{sq_n});
    std.debug.print("  -> Hex radius 2 center = {d} neighbors (PASS)\n", .{hex_n});

    // 2. Bitset 1-2-1 SAT verification
    // 3 unknowns: bit 0=(0,1), bit 1=(1,1), bit 2=(2,1)
    var valid_models: u32 = 0;
    var last_model: u8 = 0;

    var m: u8 = 0;
    while (m < 8) : (m += 1) {
        const b0 = (m >> 0) & 1;
        const b1 = (m >> 1) & 1;
        const b2 = (m >> 2) & 1;

        const c0 = b0 + b1;
        const c1 = b0 + b1 + b2;
        const c2 = b1 + b2;

        if (c0 == 1 and c1 == 2 and c2 == 1) {
            valid_models += 1;
            last_model = m;
        }
    }

    assert(valid_models == 1);
    assert(last_model == 0b101); // mines at bit 0 and bit 2, safe at bit 1
    std.debug.print("  -> Bitset SAT Model: {b:0>3} (unique model) (PASS)\n", .{last_model});
    std.debug.print("[ZIG SOLVER] SUCCESS: 100% verified.\n", .{});
}
