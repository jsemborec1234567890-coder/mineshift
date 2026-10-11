use eframe::egui;
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub enum ParticleKind {
    Debris,
    Sparkle,
    Confetti,
    Shockwave,
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub pos: egui::Pos2,
    pub vel: egui::Vec2,
    pub color: egui::Color32,
    pub size: f32,
    pub age: f32,
    pub lifetime: f32,
    pub kind: ParticleKind,
    pub rotation: f32,
    pub rot_speed: f32,
}

#[derive(Default)]
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub enabled: bool,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(512),
            enabled: true,
        }
    }

    pub fn spawn_explosion(&mut self, center: egui::Pos2, danger_color: egui::Color32, reduced_motion: bool) {
        if !self.enabled {
            return;
        }

        let mut rng = rand::thread_rng();
        let count = if reduced_motion { 16 } else { 64 };

        // Central expanding shockwave
        self.particles.push(Particle {
            pos: center,
            vel: egui::Vec2::ZERO,
            color: danger_color,
            size: 4.0,
            age: 0.0,
            lifetime: 0.45,
            kind: ParticleKind::Shockwave,
            rotation: 0.0,
            rot_speed: 0.0,
        });

        for _ in 0..count {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = if reduced_motion {
                rng.gen_range(60.0..180.0)
            } else {
                rng.gen_range(120.0..550.0)
            };
            let vel = egui::vec2(angle.cos() * speed, angle.sin() * speed);
            let lifetime = rng.gen_range(0.35..0.85);
            let size = rng.gen_range(3.0..8.0);

            // Blend danger color with flame orange and fiery yellow
            let color = match rng.gen_range(0..3) {
                0 => danger_color,
                1 => egui::Color32::from_rgb(255, 140, 20),
                _ => egui::Color32::from_rgb(255, 220, 60),
            };

            self.particles.push(Particle {
                pos: center,
                vel,
                color,
                size,
                age: 0.0,
                lifetime,
                kind: ParticleKind::Debris,
                rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                rot_speed: rng.gen_range(-12.0..12.0),
            });
        }
    }

    pub fn spawn_tile_pop(&mut self, center: egui::Pos2, accent_color: egui::Color32, reduced_motion: bool) {
        if !self.enabled {
            return;
        }

        let mut rng = rand::thread_rng();
        let count = if reduced_motion { 4 } else { 12 };

        // Subtle shockwave ring
        self.particles.push(Particle {
            pos: center,
            vel: egui::Vec2::ZERO,
            color: accent_color,
            size: 2.0,
            age: 0.0,
            lifetime: 0.25,
            kind: ParticleKind::Shockwave,
            rotation: 0.0,
            rot_speed: 0.0,
        });

        for _ in 0..count {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(40.0..180.0);
            let vel = egui::vec2(angle.cos() * speed, angle.sin() * speed);

            self.particles.push(Particle {
                pos: center,
                vel,
                color: accent_color,
                size: rng.gen_range(2.0..4.5),
                age: 0.0,
                lifetime: rng.gen_range(0.2..0.4),
                kind: ParticleKind::Sparkle,
                rotation: 0.0,
                rot_speed: 0.0,
            });
        }
    }

    pub fn spawn_victory(&mut self, rect: egui::Rect, count: usize, reduced_motion: bool) {
        if !self.enabled {
            return;
        }

        let mut rng = rand::thread_rng();
        let actual_count = if reduced_motion { count / 3 } else { count };

        let colors = [
            egui::Color32::from_rgb(121, 101, 190), // Circuit Purple
            egui::Color32::from_rgb(60, 200, 140),  // Green
            egui::Color32::from_rgb(255, 190, 40),  // Gold
            egui::Color32::from_rgb(60, 180, 255),  // Azure
            egui::Color32::from_rgb(255, 80, 120),  // Magenta
        ];

        for _ in 0..actual_count {
            let x = rng.gen_range(rect.left()..rect.right());
            let y = rng.gen_range(rect.top()..rect.top() + rect.height() * 0.4);
            let vel = egui::vec2(rng.gen_range(-120.0..120.0), rng.gen_range(80.0..320.0));
            let color = colors[rng.gen_range(0..colors.len())];

            self.particles.push(Particle {
                pos: egui::pos2(x, y),
                vel,
                color,
                size: rng.gen_range(5.0..10.0),
                age: 0.0,
                lifetime: rng.gen_range(1.5..3.0),
                kind: ParticleKind::Confetti,
                rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                rot_speed: rng.gen_range(-6.0..6.0),
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.1);
        let gravity = 450.0;
        let friction = 0.96;

        self.particles.retain_mut(|p| {
            p.age += dt;
            if p.age >= p.lifetime {
                return false;
            }

            match p.kind {
                ParticleKind::Debris => {
                    p.vel.y += gravity * dt;
                    p.vel *= friction;
                    p.pos += p.vel * dt;
                    p.rotation += p.rot_speed * dt;
                }
                ParticleKind::Sparkle => {
                    p.vel *= 0.92;
                    p.pos += p.vel * dt;
                }
                ParticleKind::Confetti => {
                    // Fluttering sway
                    let flutter = (p.age * 8.0 + p.pos.x * 0.05).sin() * 50.0;
                    p.pos += egui::vec2((p.vel.x + flutter) * dt, p.vel.y * dt);
                    p.rotation += p.rot_speed * dt;
                }
                ParticleKind::Shockwave => {
                    // Expands in place
                }
            }

            true
        });
    }

    pub fn render(&self, painter: &egui::Painter) {
        if !self.enabled || self.particles.is_empty() {
            return;
        }

        for p in &self.particles {
            let progress = (p.age / p.lifetime).clamp(0.0, 1.0);
            let alpha = ((1.0 - progress) * 255.0) as u8;

            let draw_color = egui::Color32::from_rgba_unmultiplied(
                p.color.r(),
                p.color.g(),
                p.color.b(),
                alpha,
            );

            match p.kind {
                ParticleKind::Shockwave => {
                    let radius = p.size + progress * 70.0;
                    let stroke_width = (3.0 * (1.0 - progress)).max(0.5);
                    painter.circle_stroke(
                        p.pos,
                        radius,
                        egui::Stroke::new(stroke_width, draw_color),
                    );
                }
                ParticleKind::Debris => {
                    let current_size = (p.size * (1.0 - progress * 0.6)).max(1.0);
                    let half = current_size / 2.0;
                    let cos_r = p.rotation.cos();
                    let sin_r = p.rotation.sin();

                    let corners = [
                        egui::vec2(-half, -half),
                        egui::vec2(half, -half),
                        egui::vec2(half, half),
                        egui::vec2(-half, half),
                    ];

                    let rotated_pts: Vec<egui::Pos2> = corners
                        .iter()
                        .map(|c| {
                            let rx = c.x * cos_r - c.y * sin_r;
                            let ry = c.x * sin_r + c.y * cos_r;
                            p.pos + egui::vec2(rx, ry)
                        })
                        .collect();

                    painter.add(egui::Shape::convex_polygon(
                        rotated_pts,
                        draw_color,
                        egui::Stroke::NONE,
                    ));
                }
                ParticleKind::Sparkle => {
                    let current_size = (p.size * (1.0 - progress)).max(0.8);
                    painter.circle_filled(p.pos, current_size, draw_color);
                }
                ParticleKind::Confetti => {
                    let w = p.size;
                    let h = p.size * 0.55;
                    let cos_r = p.rotation.cos();
                    let sin_r = p.rotation.sin();

                    let corners = [
                        egui::vec2(-w / 2.0, -h / 2.0),
                        egui::vec2(w / 2.0, -h / 2.0),
                        egui::vec2(w / 2.0, h / 2.0),
                        egui::vec2(-w / 2.0, h / 2.0),
                    ];

                    let rotated_pts: Vec<egui::Pos2> = corners
                        .iter()
                        .map(|c| {
                            let rx = c.x * cos_r - c.y * sin_r;
                            let ry = c.x * sin_r + c.y * cos_r;
                            p.pos + egui::vec2(rx, ry)
                        })
                        .collect();

                    painter.add(egui::Shape::convex_polygon(
                        rotated_pts,
                        draw_color,
                        egui::Stroke::NONE,
                    ));
                }
            }
        }
    }
}
