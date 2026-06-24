pub fn calculate_new_srs_state(review_level: i32, ease_factor: f32) -> Result<(i32, f32), String> {
    if ease_factor.is_nan() || ease_factor.is_infinite() {
        return Err("Invalid initial ease factor".to_string());
    }

    // Applying the SM-2 inspired update
    let new_ease = ease_factor + 0.1;

    if new_ease.is_infinite() || new_ease.is_nan() {
        return Err("Ease factor calculation resulted in invalid float".to_string());
    }

    // Clamp the ease factor to a reasonable upper bound to prevent downstream panic
    let clamped_ease = new_ease.clamp(1.3, 10.0);

    let new_level = review_level.saturating_add(1);

    Ok((new_level, clamped_ease))
}
