#[cfg(test)]
mod tests {
    use crate::theme::Theme;

    #[test]
    fn test_older_theme_format_deserialization() {
        // A theme JSON without the `cursor` field
        let old_json = r#"{
            "id": "old_theme",
            "name": "Old Theme",
            "is_custom": false,
            "colors": {
                "bg": [0,0,0],
                "menu_bg": [0,0,0],
                "card_bg": [0,0,0],
                "border": [0,0,0],
                "covered": [0,0,0],
                "revealed": [0,0,0],
                "text_primary": [0,0,0],
                "text_muted": [0,0,0],
                "accent": [0,0,0],
                "danger": [0,0,0],
                "success": [0,0,0],
                "button_bg": [0,0,0],
                "hover_highlight": [0,0,0,0],
                "number_colors": []
            },
            "geometry": {
                "cell_spacing": 1.0,
                "square_radius": 1.0,
                "shadow_depth": 1.0,
                "animation_speed": 1.0,
                "border_thickness": 1.0
            }
        }"#;

        let theme: Theme = serde_json::from_str(old_json).expect("Failed to deserialize old theme format");
        
        // Assert that the missing cursor field was safely defaulted
        assert_eq!(theme.cursor.enabled, true);
        assert_eq!(theme.cursor.size, 1.0);
        assert_eq!(theme.cursor.trail, false);
        assert_eq!(theme.cursor.reduced_motion, false);
    }
}
