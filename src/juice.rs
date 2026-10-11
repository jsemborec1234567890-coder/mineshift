use eframe::egui;
use rand::Rng;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TraumaShake {
    pub trauma: f32,
    pub max_offset: f32,
    pub decay_rate: f32,
    pub intensity: f32, // Multiplier from user settings (0.0 .. 2.0)
}

impl Default for TraumaShake {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            max_offset: 24.0,
            decay_rate: 1.6,
            intensity: 1.0,
        }
    }
}

impl TraumaShake {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trauma(&mut self, amount: f32) {
        if self.intensity <= 0.01 {
            return;
        }
        self.trauma = (self.trauma + amount * self.intensity).clamp(0.0, 1.0);
    }

    pub fn update(&mut self, dt: f32) {
        if self.trauma > 0.0 {
            self.trauma = (self.trauma - self.decay_rate * dt).max(0.0);
        }
    }

    pub fn get_offset(&self, time: f64) -> egui::Vec2 {
        if self.trauma <= 0.001 || self.intensity <= 0.01 {
            return egui::Vec2::ZERO;
        }

        // Trauma squared gives non-linear, punchy feel
        let shake = self.trauma * self.trauma * self.max_offset;
        let mut rng = rand::thread_rng();

        // High frequency pseudo-noise based on time + random jitter
        let t = (time * 45.0) as f32;
        let angle = t.sin() * std::f32::consts::PI + rng.gen_range(-0.4..0.4);
        let dist = shake * (0.6 + 0.4 * (t * 1.3).cos().abs());

        egui::vec2(angle.cos() * dist, angle.sin() * dist)
    }
}

/// Tracks reveal timestamps per cell for fluid elastic scale bounce and reveal waves.
#[derive(Default)]
pub struct CellAnimationTracker {
    revealed_at: HashMap<(usize, usize), f64>,
}

impl CellAnimationTracker {
    pub fn new() -> Self {
        Self {
            revealed_at: HashMap::new(),
        }
    }

    pub fn on_reveal(&mut self, x: usize, y: usize, current_time: f64) {
        self.revealed_at.insert((x, y), current_time);
    }

    pub fn clear(&mut self) {
        self.revealed_at.clear();
    }

    /// Returns elastic bounce scale: 0.85 -> 1.08 -> 1.0 over ~160ms.
    pub fn get_bounce_scale(&self, x: usize, y: usize, current_time: f64, speed_mult: f32) -> f32 {
        if let Some(&start_time) = self.revealed_at.get(&(x, y)) {
            let elapsed = (current_time - start_time) as f32 * speed_mult.max(0.5);
            let duration = 0.18;

            if elapsed < duration {
                let t = elapsed / duration;
                // Elastic overshoot curve: f(t) = 1 + sin(t * pi) * (1 - t) * 0.18
                let overshoot = (t * std::f32::consts::PI).sin() * (1.0 - t) * 0.18;
                return 1.0 + overshoot;
            }
        }
        1.0
    }
}
