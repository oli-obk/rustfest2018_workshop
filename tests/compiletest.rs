use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest_rs::Config::default();

    config.mode = mode.parse().expect("Invalid mode");
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path
    config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464
    config.rustc_path = PathBuf::from("target/debug/rustfest2018_workshop");

    std::env::set_var("LINTER_TESTMODE", "1");
    compiletest_rs::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("ui");
}
