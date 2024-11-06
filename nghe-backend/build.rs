fn main() {
    println!("cargo::rustc-check-cfg=cfg(hearing_test)");
    if std::env::var("NGHE_HEARING_TEST_INPUT").is_ok_and(|s| !s.is_empty())
        && std::env::var("NGHE_HEARING_TEST_OUTPUT").is_ok_and(|s| !s.is_empty())
    {
        println!("cargo::rustc-cfg=hearing_test");
    }
}
