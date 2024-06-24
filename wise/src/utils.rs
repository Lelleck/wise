use tracing::level_filters::LevelFilter;

pub const fn get_levelfilter(num: i32) -> LevelFilter {
    match num {
        -2 => LevelFilter::ERROR,
        -1 => LevelFilter::WARN,
        1 => LevelFilter::DEBUG,
        2 => LevelFilter::TRACE,
        0 | _ => LevelFilter::INFO,
    }
}
