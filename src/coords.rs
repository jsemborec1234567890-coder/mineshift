use eframe::egui;
use minecore::board::Geometry;

/// Calculates the exact screen coordinate center for a board cell under both Square and Hexagon geometries.
///
/// Under Hexagon geometry, cells are centered using axial coordinates with:
/// `q = x - radius`, `r = y - radius`
/// `cx = cell_size * sqrt(3) * (q + r / 2.0)`
/// `cy = cell_size * 1.5 * r`
/// with origin at `board_origin + vec2(board_w / 2.0, board_h / 2.0)`.
pub fn compute_cell_screen_center(
    geometry: Geometry,
    x: usize,
    y: usize,
    radius: usize,
    cell_size: f32,
    spacing: f32,
    board_origin: egui::Pos2,
    board_w: f32,
    board_h: f32,
    shake_offset: egui::Vec2,
) -> egui::Pos2 {
    match geometry {
        Geometry::Square => {
            board_origin
                + egui::vec2(
                    x as f32 * (cell_size + spacing) + cell_size / 2.0,
                    y as f32 * (cell_size + spacing) + cell_size / 2.0,
                )
                + shake_offset
        }
        Geometry::Hex => {
            let q = x as f32 - radius as f32;
            let r = y as f32 - radius as f32;
            let cx = cell_size * 3.0_f32.sqrt() * (q + r / 2.0);
            let cy = cell_size * 1.5 * r;
            board_origin + egui::vec2(board_w / 2.0 + cx, board_h / 2.0 + cy) + shake_offset
        }
    }
}

/// Generates the 6 vertices for a regular hexagon centered at `center`.
pub fn hex_vertices(center: egui::Pos2, radius: f32) -> Vec<egui::Pos2> {
    (0..6)
        .map(|i| {
            let angle = (i as f32 * 60.0 - 30.0).to_radians();
            center + egui::vec2(angle.cos() * radius, angle.sin() * radius)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_screen_center() {
        let origin = egui::pos2(100.0, 50.0);
        let cell_size = 30.0;
        let spacing = 4.0;
        let center_0_0 = compute_cell_screen_center(
            Geometry::Square,
            0,
            0,
            0,
            cell_size,
            spacing,
            origin,
            400.0,
            400.0,
            egui::Vec2::ZERO,
        );
        assert!((center_0_0.x - (100.0 + 15.0)).abs() < 1e-4);
        assert!((center_0_0.y - (50.0 + 15.0)).abs() < 1e-4);

        let center_2_3 = compute_cell_screen_center(
            Geometry::Square,
            2,
            3,
            0,
            cell_size,
            spacing,
            origin,
            400.0,
            400.0,
            egui::Vec2::ZERO,
        );
        assert!((center_2_3.x - (100.0 + 2.0 * 34.0 + 15.0)).abs() < 1e-4);
        assert!((center_2_3.y - (50.0 + 3.0 * 34.0 + 15.0)).abs() < 1e-4);
    }

    #[test]
    fn test_hex_screen_center_origin_and_symmetry() {
        let origin = egui::pos2(200.0, 100.0);
        let board_w = 600.0;
        let board_h = 600.0;
        let radius = 4;
        let cell_size = 25.0;

        // The center cell in hexagonal axial space is at x = radius, y = radius (q=0, r=0)
        let center_mid = compute_cell_screen_center(
            Geometry::Hex,
            radius,
            radius,
            radius,
            cell_size,
            0.0,
            origin,
            board_w,
            board_h,
            egui::Vec2::ZERO,
        );

        // Center must be exactly at origin + board_w/2, board_h/2
        assert!((center_mid.x - (origin.x + board_w / 2.0)).abs() < 1e-4);
        assert!((center_mid.y - (origin.y + board_h / 2.0)).abs() < 1e-4);

        // Opposite points should be symmetric around center
        let p_plus = compute_cell_screen_center(
            Geometry::Hex,
            radius + 1,
            radius,
            radius,
            cell_size,
            0.0,
            origin,
            board_w,
            board_h,
            egui::Vec2::ZERO,
        );
        let p_minus = compute_cell_screen_center(
            Geometry::Hex,
            radius - 1,
            radius,
            radius,
            cell_size,
            0.0,
            origin,
            board_w,
            board_h,
            egui::Vec2::ZERO,
        );
        assert!(((p_plus.x - center_mid.x) + (p_minus.x - center_mid.x)).abs() < 1e-4);
        assert!(((p_plus.y - center_mid.y) + (p_minus.y - center_mid.y)).abs() < 1e-4);
    }

    #[test]
    fn test_hex_vertices_count_and_radius() {
        let center = egui::pos2(150.0, 150.0);
        let r = 20.0;
        let verts = hex_vertices(center, r);
        assert_eq!(verts.len(), 6);
        for v in verts {
            let dist = (v - center).length();
            assert!((dist - r).abs() < 1e-4);
        }
    }
}
