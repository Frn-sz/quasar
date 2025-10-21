#[macro_export]
macro_rules! measure_block {
    ($metric:expr, $unit:ident, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let elapsed = start.elapsed();

        let duration_value = match stringify!($unit) {
            "s" => elapsed.as_secs_f64(),
            "ms" => elapsed.as_millis() as f64,
            "us" => elapsed.as_micros() as f64,
            _ => panic!("Invalid measurement unit for macro measure! Use 's', 'ms', or 'us'."),
        };

        $metric.observe(duration_value);
        result
    }};
}

#[macro_export]
macro_rules! measure {
    ($metric:expr, $code:block) => {
        $crate::measure_block!($metric, s, $code)
    };
}

#[macro_export]
macro_rules! measure_ms {
    ($metric:expr, $code:block) => {
        $crate::measure_block!($metric, ms, $code)
    };
}

#[macro_export]
macro_rules! measure_us {
    ($metric:expr, $code:block) => {
        $crate::measure_block!($metric, us, $code)
    };
}
