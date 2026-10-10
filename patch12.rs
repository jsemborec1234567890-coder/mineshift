use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    let old = "                    let _ = db.set_profile_cosmetics(prof.id, &prof.title, &prof.badge_id);\n                    // Also need to save avatar_id somehow... wait, does set_profile_avatar exist?";
    let new = "                    let _ = db.set_profile_cosmetics(prof.id, &prof.title, &prof.badge_id);\n                    let _ = db.set_profile_avatar(prof.id, &prof.avatar_id);";
    text = text.replace(old, new);
    fs::write("src/main.rs", text).unwrap();
}
