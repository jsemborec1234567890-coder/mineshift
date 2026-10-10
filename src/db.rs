use rusqlite::{Connection, Result, params, OptionalExtension};
use std::path::PathBuf;
use directories::ProjectDirs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Profile {
    pub id: i64,
    pub display_name: String,
    pub created_at: i64,
    pub last_played_at: i64,
    pub avatar_id: String,
    pub current_level: i64,
    pub total_xp: i64,
}

#[derive(Clone, Debug)]
pub struct GameSession {
    pub profile_id: i64,
    pub mode: String,
    pub geometry: String,
    pub board_width: i64,
    pub board_height: i64,
    pub hex_radius: i64,
    pub mine_count: i64,
    pub started_at: i64,
    pub duration_ms: i64,
    pub result: String, // "Won", "Lost"
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::db_path();
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let mut conn = Connection::open(&db_path)?;
        Self::apply_migrations(&mut conn)?;
        
        Ok(Self { conn })
    }

    fn db_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "Antigravity", "MineShift") {
            proj_dirs.data_dir().join("mineshift.db")
        } else {
            PathBuf::from("mineshift.db")
        }
    }
    
    fn apply_migrations(conn: &mut Connection) -> Result<()> {
        let tx = conn.transaction()?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                display_name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_played_at INTEGER NOT NULL,
                avatar_id TEXT NOT NULL,
                selected_theme_id TEXT NOT NULL,
                current_level INTEGER NOT NULL DEFAULT 1,
                total_xp INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        
        tx.execute(
            "CREATE TABLE IF NOT EXISTS game_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                mode TEXT NOT NULL,
                geometry TEXT NOT NULL,
                board_width INTEGER NOT NULL,
                board_height INTEGER NOT NULL,
                hex_radius INTEGER NOT NULL,
                mine_count INTEGER NOT NULL,
                started_at INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                result TEXT NOT NULL,
                FOREIGN KEY(profile_id) REFERENCES profiles(id)
            )",
            [],
        )?;
        
        tx.execute(
            "CREATE TABLE IF NOT EXISTS profile_achievements (
                profile_id INTEGER NOT NULL,
                achievement_id TEXT NOT NULL,
                unlocked_at INTEGER NOT NULL,
                PRIMARY KEY (profile_id, achievement_id),
                FOREIGN KEY(profile_id) REFERENCES profiles(id)
            )",
            [],
        )?;
        
        tx.execute(
            "CREATE TABLE IF NOT EXISTS game_statistics (
                profile_id INTEGER NOT NULL,
                statistic_key TEXT NOT NULL,
                statistic_value INTEGER NOT NULL,
                PRIMARY KEY (profile_id, statistic_key),
                FOREIGN KEY(profile_id) REFERENCES profiles(id)
            )",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_profiles(&self) -> Result<Vec<Profile>> {
        let mut stmt = self.conn.prepare("SELECT id, display_name, created_at, last_played_at, avatar_id, current_level, total_xp FROM profiles")?;
        let profile_iter = stmt.query_map([], |row| {
            Ok(Profile {
                id: row.get(0)?,
                display_name: row.get(1)?,
                created_at: row.get(2)?,
                last_played_at: row.get(3)?,
                avatar_id: row.get(4)?,
                current_level: row.get(5)?,
                total_xp: row.get(6)?,
            })
        })?;
        let mut profiles = Vec::new();
        for p in profile_iter {
            profiles.push(p?);
        }
        Ok(profiles)
    }

    pub fn create_profile(&self, display_name: &str, avatar_id: &str) -> Result<i64> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.conn.execute(
            "INSERT INTO profiles (display_name, created_at, last_played_at, avatar_id, selected_theme_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![display_name, now, now, avatar_id, "soft_circuit"],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_profile_last_played(&self, profile_id: i64) -> Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.conn.execute("UPDATE profiles SET last_played_at = ?1 WHERE id = ?2", params![now, profile_id])?;
        Ok(())
    }

    pub fn add_xp(&self, profile_id: i64, xp: i64) -> Result<()> {
        self.conn.execute("UPDATE profiles SET total_xp = total_xp + ?1 WHERE id = ?2", params![xp, profile_id])?;
        self.update_level(profile_id)?;
        Ok(())
    }

    fn update_level(&self, profile_id: i64) -> Result<()> {
        // level threshold: level = sqrt(xp / 100) + 1 roughly
        let xp: i64 = self.conn.query_row("SELECT total_xp FROM profiles WHERE id = ?1", params![profile_id], |r| r.get(0))?;
        let mut level = 1;
        let mut required = 100;
        while xp >= required {
            level += 1;
            required += level * 100;
        }
        self.conn.execute("UPDATE profiles SET current_level = ?1 WHERE id = ?2", params![level, profile_id])?;
        let _ = self.evaluate_achievements(profile_id);
        Ok(())
    }

    pub fn save_session(&self, session: &GameSession) -> Result<()> {
        self.conn.execute(
            "INSERT INTO game_sessions (profile_id, mode, geometry, board_width, board_height, hex_radius, mine_count, started_at, duration_ms, result)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                session.profile_id, session.mode, session.geometry, session.board_width, session.board_height,
                session.hex_radius, session.mine_count, session.started_at, session.duration_ms, session.result
            ]
        )?;
        
        // Update stats
        self.increment_stat(session.profile_id, "games_played")?;
        if session.result == "Won" {
            self.increment_stat(session.profile_id, "games_won")?;
            if session.geometry == "Hex" {
                self.increment_stat(session.profile_id, "hex_won")?;
            }
        }
        
        let _ = self.evaluate_achievements(session.profile_id);
        
        Ok(())
    }

    pub fn increment_stat(&self, profile_id: i64, stat_key: &str) -> Result<()> {
        let updated = self.conn.execute(
            "UPDATE game_statistics SET statistic_value = statistic_value + 1 WHERE profile_id = ?1 AND statistic_key = ?2",
            params![profile_id, stat_key]
        )?;
        if updated == 0 {
            self.conn.execute(
                "INSERT INTO game_statistics (profile_id, statistic_key, statistic_value) VALUES (?1, ?2, 1)",
                params![profile_id, stat_key]
            )?;
        }
        Ok(())
    }
    
    pub fn get_stat(&self, profile_id: i64, stat_key: &str) -> i64 {
        self.conn.query_row(
            "SELECT statistic_value FROM game_statistics WHERE profile_id = ?1 AND statistic_key = ?2",
            params![profile_id, stat_key],
            |row| row.get(0)
        ).unwrap_or(0)
    }

    pub fn unlock_achievement(&self, profile_id: i64, achievement_id: &str) -> Result<bool> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let res = self.conn.execute(
            "INSERT OR IGNORE INTO profile_achievements (profile_id, achievement_id, unlocked_at) VALUES (?1, ?2, ?3)",
            params![profile_id, achievement_id, now]
        )?;
        Ok(res > 0)
    }

    pub fn get_unlocked_achievements(&self, profile_id: i64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT achievement_id FROM profile_achievements WHERE profile_id = ?1")?;
        let iter = stmt.query_map(params![profile_id], |row| row.get(0))?;
        let mut res = Vec::new();
        for a in iter { res.push(a?); }
        Ok(res)
    }

    pub fn add_stat(&self, profile_id: i64, stat_key: &str, amount: i64) -> Result<()> {
        let updated = self.conn.execute(
            "UPDATE game_statistics SET statistic_value = statistic_value + ?3 WHERE profile_id = ?1 AND statistic_key = ?2",
            params![profile_id, stat_key, amount]
        )?;
        if updated == 0 {
            self.conn.execute(
                "INSERT INTO game_statistics (profile_id, statistic_key, statistic_value) VALUES (?1, ?2, ?3)",
                params![profile_id, stat_key, amount]
            )?;
        }
        Ok(())
    }

    pub fn set_highest_stat(&self, profile_id: i64, record_key: &str, val: i64) -> Result<()> {
        let current = self.get_stat(profile_id, record_key);
        if val > current {
            let updated = self.conn.execute(
                "UPDATE game_statistics SET statistic_value = ?3 WHERE profile_id = ?1 AND statistic_key = ?2",
                params![profile_id, record_key, val]
            )?;
            if updated == 0 {
                self.conn.execute(
                    "INSERT INTO game_statistics (profile_id, statistic_key, statistic_value) VALUES (?1, ?2, ?3)",
                    params![profile_id, record_key, val]
                )?;
            }
        }
        Ok(())
    }

    pub fn set_best_time(&self, profile_id: i64, record_key: &str, time_ms: i64) -> Result<()> {
        let current = self.get_stat(profile_id, record_key);
        if current == 0 || time_ms < current {
            let updated = self.conn.execute(
                "UPDATE game_statistics SET statistic_value = ?3 WHERE profile_id = ?1 AND statistic_key = ?2",
                params![profile_id, record_key, time_ms]
            )?;
            if updated == 0 {
                self.conn.execute(
                    "INSERT INTO game_statistics (profile_id, statistic_key, statistic_value) VALUES (?1, ?2, ?3)",
                    params![profile_id, record_key, time_ms]
                )?;
            }
        }
        Ok(())
    }

    pub fn delete_profile(&self, profile_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM profile_achievements WHERE profile_id = ?1", params![profile_id])?;
        self.conn.execute("DELETE FROM game_statistics WHERE profile_id = ?1", params![profile_id])?;
        self.conn.execute("DELETE FROM game_sessions WHERE profile_id = ?1", params![profile_id])?;
        self.conn.execute("DELETE FROM profiles WHERE id = ?1", params![profile_id])?;
        Ok(())
    }
    
    pub fn rename_profile(&self, profile_id: i64, new_name: &str) -> Result<()> {
        self.conn.execute("UPDATE profiles SET display_name = ?2 WHERE id = ?1", params![profile_id, new_name])?;
        Ok(())
    }
    pub fn evaluate_achievements(&self, profile_id: i64) -> Result<()> {
        let played = self.get_stat(profile_id, "games_played");
        let won = self.get_stat(profile_id, "games_won");
        let playtime = self.get_stat(profile_id, "total_playtime_s");
        let hex_won = self.get_stat(profile_id, "hex_won");
        let lost = played - won;
        
        let level: i64 = self.conn.query_row("SELECT current_level FROM profiles WHERE id = ?1", params![profile_id], |r| r.get(0)).unwrap_or(1);
        
        if played >= 10 { let _ = self.unlock_achievement(profile_id, "play_10"); }
        if played >= 100 { let _ = self.unlock_achievement(profile_id, "play_100"); }
        if played >= 1000 { let _ = self.unlock_achievement(profile_id, "play_1000"); }
        if won >= 10 { let _ = self.unlock_achievement(profile_id, "win_10"); }
        if won >= 50 { let _ = self.unlock_achievement(profile_id, "win_50"); }
        if won >= 100 { let _ = self.unlock_achievement(profile_id, "win_100"); }
        if won >= 500 { let _ = self.unlock_achievement(profile_id, "win_500"); }
        if lost >= 1 { let _ = self.unlock_achievement(profile_id, "lose_1"); }
        if lost >= 100 { let _ = self.unlock_achievement(profile_id, "lose_100"); }
        if hex_won >= 10 { let _ = self.unlock_achievement(profile_id, "hex_10"); }
        if hex_won >= 50 { let _ = self.unlock_achievement(profile_id, "hex_50"); }
        if playtime >= 3600 { let _ = self.unlock_achievement(profile_id, "playtime_1h"); }
        if playtime >= 36000 { let _ = self.unlock_achievement(profile_id, "playtime_10h"); }
        if playtime >= 360000 { let _ = self.unlock_achievement(profile_id, "playtime_100h"); }
        if level >= 5 { let _ = self.unlock_achievement(profile_id, "level_5"); }
        if level >= 10 { let _ = self.unlock_achievement(profile_id, "level_10"); }
        if level >= 50 { let _ = self.unlock_achievement(profile_id, "level_50"); }
        Ok(())
    }
}
