use clap::crate_version;

#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("README.md")
        .insert_var("[VERSION]", crate_version!())
        .unwrap();
}
