use std::env;
use self_update::{self, InstallOpts};
use rustup::Result;
use clap::{App, Arg};
use common;

pub fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let arg1 = args.get(1).map(|a| &**a);

    // Secret command used during self-update. Not for users.
    if arg1 == Some("--self-replace") {
        return self_update::self_replace();
    }

    let cli = App::new("multirust-setup")
        .version(common::version())
        .about("The installer for multirust")
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Enable verbose output"))
        .arg(Arg::with_name("no-prompt")
             .short("y")
             .help("Disable confirmation prompt."))
        .arg(Arg::with_name("default-toolchain")
             .long("default-toolchain")
             .takes_value(true)
             .possible_values(&["stable", "beta", "nightly"])
             .help("Choose a default toolchain to install"))
        .arg(Arg::with_name("no-modify-path")
             .long("no-modify-path")
             .help("Don't configure the PATH environment variable"));

    let matches = cli.get_matches();
    let no_prompt = matches.is_present("no-prompt");
    let verbose = matches.is_present("verbose");
    let default_toolchain = matches.value_of("default-toolchain").unwrap_or("stable");
    let no_modify_path = matches.is_present("no-modify-path");

    let opts = InstallOpts {
        default_toolchain: default_toolchain.to_owned(),
        no_modify_path: no_modify_path,
    };

    try!(self_update::install(no_prompt, verbose, opts));

    Ok(())
}
