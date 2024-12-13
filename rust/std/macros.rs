macro_rules! pages {
        ($v:expr) => {{
                use sys::getpagesize;
                let size = unsafe { getpagesize() };
                if size > 0 {
                        1 + ($v as u64 - 1 as u64) / size as u64
                } else {
                        0
                }
        }};
}

#[macro_export]
macro_rules! page_size {
        () => {{
                use sys::getpagesize;
                let v = unsafe { getpagesize() } as u64;
                v
        }};
}
