#[macro_export]
macro_rules! ternary {
    ($condition:expr, $if_true:expr, $if_false:expr) => {
        if $condition != 0 { $if_true } else { $if_false }
    };
}

#[macro_export]
macro_rules! assert_close {
    ($left:expr, $right:expr, $tolerance:expr) => {
        let left = $left;
        let right = $right;
        let tol = $tolerance;
        let diff = (left - right).abs();
        assert!(
            diff < tol,
            "assertion failed: {} ~= {} \
                (tolerance: {}, difference: {})",
            left,
            right,
            tol,
            diff
        );
    };
}
