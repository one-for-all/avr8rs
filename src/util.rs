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

#[macro_export]
macro_rules! flog {
    ($($t:tt)*) => {
        // #[cfg(debug_assertions)]
        {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!($($t)*).into());
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!($($t)*);
            }
        }
    };
}
