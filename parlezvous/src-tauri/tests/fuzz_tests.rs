use parlezvous_lib::services::srs::calculate_new_srs_state;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_srs_math_does_not_panic(
        review_level in i32::MIN..i32::MAX,
        ease_factor in proptest::num::f32::ANY
    ) {
        // We only care that the function doesn't panic.
        // It's allowed to return an Err if the inputs are invalid floats.
        let _ = calculate_new_srs_state(review_level, ease_factor);
    }

    #[test]
    fn test_srs_math_valid_ranges(
        review_level in 0..1000i32,
        ease_factor in 1.3f32..10.0f32
    ) {
        // For standard inputs, it should always succeed
        let result = calculate_new_srs_state(review_level, ease_factor);
        prop_assert!(result.is_ok());

        let (new_level, new_ease) = result.unwrap();

        // Ensure new_level increases or saturates
        prop_assert!(new_level >= review_level);

        // Ensure ease is clamped
        prop_assert!(new_ease >= 1.3 && new_ease <= 10.0);
    }
}
