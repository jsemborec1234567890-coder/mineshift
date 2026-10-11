/**
 * MINE//SHIFT: BEYOND — Java Puzzle Verifier & Topology Oracle
 * Oracle JDK 17 implementation
 */

import java.util.*;

public class PuzzleVerifier {
    public static int getSquareRing2Count(int x, int y, int w, int h) {
        int count = 0;
        for (int dy = -2; dy <= 2; dy++) {
            for (int dx = -2; dx <= 2; dx++) {
                int dist = Math.max(Math.abs(dx), Math.abs(dy));
                if (dist == 2) {
                    int nx = x + dx;
                    int ny = y + dy;
                    if (nx >= 0 && nx < w && ny >= 0 && ny < h) {
                        count++;
                    }
                }
            }
        }
        return count;
    }

    public static int getHexNeighborCount(int x, int y, int radius) {
        int span = radius * 2 + 1;
        int[][] deltas = {{1, 0}, {1, -1}, {0, -1}, {-1, 0}, {-1, 1}, {0, 1}};
        int count = 0;
        for (int[] d : deltas) {
            int nx = x + d[0];
            int ny = y + d[1];
            if (nx >= 0 && nx < span && ny >= 0 && ny < span) {
                int q = nx - radius;
                int r = ny - radius;
                if (Math.abs(q) <= radius && Math.abs(r) <= radius && Math.abs(q + r) <= radius) {
                    count++;
                }
            }
        }
        return count;
    }

    public static void main(String[] args) {
        System.out.println("[JAVA VERIFIER] Running Java Topology & Constraint Verification...");

        // 1. Ring-2 Chebyshev distance count for interior cell in 7x7
        int r2_count = getSquareRing2Count(3, 3, 7, 7);
        if (r2_count != 16) {
            throw new RuntimeException("Square ring-2 count != 16, got: " + r2_count);
        }
        System.out.println("  -> Square 7x7 center Ring-2 cell count = 16 (PASS)");

        // 2. Hex center cell neighbor count
        int hex_count = getHexNeighborCount(2, 2, 2);
        if (hex_count != 6) {
            throw new RuntimeException("Hex center count != 6, got: " + hex_count);
        }
        System.out.println("  -> Hex radius 2 center neighbor count = 6 (PASS)");

        // 3. 1-2-1 Pattern Model Check
        // Set A: {(0,1), (1,1)} with 1 mine
        // Set B: {(0,1), (1,1), (2,1)} with 2 mines
        Set<String> setA = new HashSet<>(Arrays.asList("0,1", "1,1"));
        Set<String> setB = new HashSet<>(Arrays.asList("0,1", "1,1", "2,1"));

        Set<String> diff = new HashSet<>(setB);
        diff.removeAll(setA);

        if (diff.size() != 1 || !diff.contains("2,1")) {
            throw new RuntimeException("Set difference deduction failed");
        }
        System.out.println("  -> Java Set Difference: B \\ A = {2,1} (Mines = 2-1 = 1) -> Forced Mine (PASS)");

        System.out.println("[JAVA VERIFIER] SUCCESS: 100% verified.");
    }
}
