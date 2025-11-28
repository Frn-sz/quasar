#[macro_export]
macro_rules! measure_block {
    ($metric:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let elapsed = start.elapsed();

        $metric.observe(elapsed.as_secs_f64());
        result
    }};
}

#[macro_export]
macro_rules! measure {
    ($metric:expr, $code:block) => {
        $crate::measure_block!($metric, $code)
    };
}
