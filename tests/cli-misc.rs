//! Test cases of the multirust command that do not depend on the
//! dist server, mostly derived from multirust/test-v2.sh

extern crate rustup_dist;
extern crate rustup_utils;
extern crate rustup_mock;

use rustup_mock::clitools::{self, Config, Scenario,
                               expect_stdout_ok, expect_stderr_ok,
                               expect_ok, expect_err, run,
                               this_host_triple};

macro_rules! for_host { ($s: expr) => (&format!($s, this_host_triple())) }

pub fn setup(f: &Fn(&Config)) {
    clitools::setup(Scenario::SimpleV2, f);
}

#[test]
fn smoke_test() {
    setup(&|config| {
        expect_ok(config, &["rustup", "--version"]);
    });
}

#[test]
fn no_colors_in_piped_error_output() {
    setup(&|config| {
        let out = run(config, "rustc", &[], &[]);
        assert!(!out.ok);
        assert!(!out.stderr.contains("\u{1b}"));
    });
}

#[test]
fn rustc_with_bad_multirust_toolchain_env_var() {
    setup(&|config| {
        let out = run(config, "rustc", &[], &[("RUSTUP_TOOLCHAIN", "bogus")]);
        assert!(!out.ok);
        assert!(out.stderr.contains("toolchain 'bogus' is not installed"));
    });
}

#[test]
fn custom_invalid_names() {
    setup(&|config| {
        expect_err(config, &["rustup", "toolchain", "link", "nightly",
                             "foo"],
                   for_host!("invalid custom toolchain name: 'nightly-{0}'"));
        expect_err(config, &["rustup", "toolchain", "link", "beta",
                             "foo"],
                   for_host!("invalid custom toolchain name: 'beta-{0}'"));
        expect_err(config, &["rustup", "toolchain", "link", "stable",
                             "foo"],
                   for_host!("invalid custom toolchain name: 'stable-{0}'"));
    });
}

#[test]
fn custom_invalid_names_with_archive_dates() {
    setup(&|config| {
        expect_err(config, &["rustup", "toolchain", "link", "nightly-2015-01-01",
                             "foo"],
                   for_host!("invalid custom toolchain name: 'nightly-2015-01-01-{0}'"));
        expect_err(config, &["rustup", "toolchain", "link", "beta-2015-01-01",
                             "foo"],
                   for_host!("invalid custom toolchain name: 'beta-2015-01-01-{0}'"));
        expect_err(config, &["rustup", "toolchain", "link", "stable-2015-01-01",
                             "foo"],
                   for_host!("invalid custom toolchain name: 'stable-2015-01-01-{0}'"));
    });
}

#[test]
fn running_with_v2_metadata() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        // Replace the metadata version
        rustup_utils::raw::write_file(&config.rustupdir.join("version"),
                               "2").unwrap();
        expect_err(config, &["rustup", "default", "nightly"],
                   "rustup's metadata is out of date. run `rustup self upgrade-data`");
        expect_err(config, &["rustc", "--version"],
                   "rustup's metadata is out of date. run `rustup self upgrade-data`");
    });
}

// The thing that changed in the version bump from 2 -> 12 was the
// toolchain format. Check that on the upgrade all the toolchains.
// are deleted.
#[test]
fn upgrade_v2_metadata_to_v12() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        // Replace the metadata version
        rustup_utils::raw::write_file(&config.rustupdir.join("version"),
                               "2").unwrap();
        expect_stderr_ok(config, &["rustup", "self", "upgrade-data"],
                         "warning: this upgrade will remove all existing toolchains. you will need to reinstall them");
        expect_err(config, &["rustc", "--version"],
                   for_host!("toolchain 'nightly-{0}' is not installed"));
        expect_ok(config, &["rustup", "update", "nightly"]);
        expect_stdout_ok(config, &["rustc", "--version"],
                         "hash-n-2");
    });
}

// Regression test for newline placement
#[test]
fn update_all_no_update_whitespace() {
    setup(&|config| {
        expect_stdout_ok(config, &["rustup", "update", "nightly"],
for_host!(r"
  nightly-{} installed - 1.3.0 (hash-n-2)

"));
    });
}

// Issue #145
#[test]
fn update_works_without_term() {
    setup(&|config| {
        let mut cmd = clitools::cmd(config, "rustup", &["update", "nightly"]);
        clitools::env(config, &mut cmd);
        cmd.env_remove("TERM");

        let out = cmd.output().unwrap();
        assert!(out.status.success());
    });
}

// Issue #140
// Don't panic when `target`, `update` etc. are called without subcommands.
#[test]
fn subcommand_required_for_target() {
    setup(&|config| {
        let mut cmd = clitools::cmd(config, "rustup", &["target"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        assert!(!out.status.success());
        assert!(out.status.code().unwrap() != 101);
    });
}

// Issue #140
// Don't panic when `target`, `update` etc. are called without subcommands.
#[test]
fn subcommand_required_for_toolchain() {
    setup(&|config| {
        let mut cmd = clitools::cmd(config, "rustup", &["toolchain"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        assert!(!out.status.success());
        assert!(out.status.code().unwrap() != 101);
    });
}

// Issue #140
// Don't panic when `target`, `update` etc. are called without subcommands.
#[test]
fn subcommand_required_for_override() {
    setup(&|config| {
        let mut cmd = clitools::cmd(config, "rustup", &["override"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        assert!(!out.status.success());
        assert!(out.status.code().unwrap() != 101);
    });
}

// Issue #140
// Don't panic when `target`, `update` etc. are called without subcommands.
#[test]
fn subcommand_required_for_self() {
    setup(&|config| {
        let mut cmd = clitools::cmd(config, "rustup", &["self"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        assert!(!out.status.success());
        assert!(out.status.code().unwrap() != 101);
    });
}

#[test]
fn multi_host_smoke_test() {
    // FIXME: Unfortunately the list of supported hosts is hard-coded,
    // so we have to use the triple of a host we actually test on. That means
    // that when we're testing on that host we can't test 'multi-host'.
    let trip = this_host_triple();
    if trip == clitools::MULTI_ARCH1 {
        return;
    }

    clitools::setup(Scenario::MultiHost, &|config| {
        let ref toolchain = format!("nightly-{}", clitools::MULTI_ARCH1);
        expect_ok(config, &["rustup", "default", toolchain]);
        expect_stdout_ok(config, &["rustc", "--version"],
                         "xxxx-n-2"); // cross-host mocks have their own versions
    });
}

#[test]
fn custom_toolchain_cargo_fallback_proxy() {
    setup(&|config| {
        let path = config.customdir.join("custom-1");

        expect_ok(config, &["rustup", "toolchain", "link", "mytoolchain",
                            &path.to_string_lossy()]);
        expect_ok(config, &["rustup", "default", "mytoolchain"]);

        expect_ok(config, &["rustup", "update", "stable"]);
        expect_stdout_ok(config, &["cargo", "--version"],
                         "hash-s-2");

        expect_ok(config, &["rustup", "update", "beta"]);
        expect_stdout_ok(config, &["cargo", "--version"],
                         "hash-b-2");

        expect_ok(config, &["rustup", "update", "nightly"]);
        expect_stdout_ok(config, &["cargo", "--version"],
                         "hash-n-2");
    });
}

#[test]
fn custom_toolchain_cargo_fallback_run() {
    setup(&|config| {
        let path = config.customdir.join("custom-1");

        expect_ok(config, &["rustup", "toolchain", "link", "mytoolchain",
                            &path.to_string_lossy()]);
        expect_ok(config, &["rustup", "default", "mytoolchain"]);

        expect_ok(config, &["rustup", "update", "stable"]);
        expect_stdout_ok(config, &["rustup", "run", "mytoolchain",
                                   "cargo", "--version"],
                         "hash-s-2");

        expect_ok(config, &["rustup", "update", "beta"]);
        expect_stdout_ok(config, &["rustup", "run", "mytoolchain",
                                   "cargo", "--version"],
                         "hash-b-2");

        expect_ok(config, &["rustup", "update", "nightly"]);
        expect_stdout_ok(config, &["rustup", "run", "mytoolchain",
                                   "cargo", "--version"],
                         "hash-n-2");

    });
}

#[test]
fn multirust_env_compat() {
    setup(&|config| {
        let mut cmd = clitools::cmd(config, "rustup", &["update", "nightly"]);
        clitools::env(config, &mut cmd);
        cmd.env_remove("RUSTUP_HOME");
        cmd.env("MULTIRUST_HOME", &config.rustupdir);
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr.contains("environment variable MULTIRUST_HOME is deprecated. Use RUSTUP_HOME"));
    });
}

#[test]
fn toolchains_are_resolved_early() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);

        let full_toolchain = format!("nightly-{}", this_host_triple());
        expect_stderr_ok(config, &["rustup", "default", &full_toolchain],
                         &format!("info: using existing install for '{}'", full_toolchain));
    });
}

// #190
#[test]
fn proxies_pass_empty_args() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(config, &["rustup", "run", "nightly", "rustc", "--empty-arg-test", ""]);
    });
}
