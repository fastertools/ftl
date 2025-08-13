use clap::{Arg, ArgAction, Command, value_parser};
use clap_mangen::Man;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Build the CLI structure (simplified version matching the actual CLI)
    let cmd = build_cli();

    // Generate man page
    let man = Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    // Write to OUT_DIR for embedding in binary
    fs::write(out_dir.join("ftl-resolve.1"), &buffer)?;

    // Also write to a predictable location in target directory for distribution
    // This makes it easy to find for packaging
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let target_dir = PathBuf::from(manifest_dir)
            .parent()  // crates
            .and_then(|p| p.parent())  // project root
            .map(|p| p.join("target"))
            .unwrap_or_else(|| PathBuf::from("target"));

        let man_dir = target_dir.join("man");
        fs::create_dir_all(&man_dir).ok();
        fs::write(man_dir.join("ftl-resolve.1"), &buffer).ok();
    }

    Ok(())
}

fn build_cli() -> Command {
    Command::new("ftl-resolve")
        .version(env!("CARGO_PKG_VERSION"))
        .author("FTL Platform Team")
        .about("Component resolver and transpiler for FTL configuration files")
        .long_about(
            "Resolves registry components, validates syntax, generates JSON schemas, \
             and transpiles FTL configuration to Spin TOML format. Supports TOML and JSON inputs.\n\n\
             Uses wkg to download registry components by default, with option to preserve Spin's native registry format."
        )
        .after_help("For more information, see: https://github.com/fastertools/ftl-cli")
        .subcommand(
            Command::new("spin")
                .about("Generate Spin TOML, resolving registry components with wkg by default")
                .arg(Arg::new("input")
                    .value_name("FILE")
                    .help("Input file (ftl.toml or ftl.json, use '-' for stdin)")
                    .required(true)
                    .index(1)
                    .value_parser(value_parser!(PathBuf)))
                .arg(Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("Output spin.toml file (writes to stdout if not provided)")
                    .value_parser(value_parser!(PathBuf)))
                .arg(Arg::new("format")
                    .short('f')
                    .long("format")
                    .value_name("FORMAT")
                    .help("Format of the input (toml or json)")
                    .default_value("toml")
                    .value_parser(["toml", "json"]))
                .arg(Arg::new("project-dir")
                    .short('d')
                    .long("project-dir")
                    .value_name("DIR")
                    .help("Project directory for resolving relative paths")
                    .default_value(".")
                    .value_parser(value_parser!(PathBuf)))
                .arg(Arg::new("spin-resolve")
                    .long("spin-resolve")
                    .help("Use Spin's native registry resolution instead of wkg")
                    .action(ArgAction::SetTrue))
                .arg(Arg::new("no-cache")
                    .long("no-cache")
                    .help("Force fresh downloads, ignoring cached components")
                    .action(ArgAction::SetTrue))
        )
        .subcommand(
            Command::new("schema")
                .about("Generate JSON schema for FTL configuration validation")
                .arg(Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("Output file for the schema (writes to stdout if not provided)")
                    .value_parser(value_parser!(PathBuf)))
                .arg(Arg::new("mini")
                    .short('m')
                    .long("mini")
                    .help("Output minified JSON instead of pretty-printed")
                    .action(ArgAction::SetTrue))
        )
        .subcommand(
            Command::new("validate")
                .about("Validate FTL configuration file syntax and structure")
                .arg(Arg::new("input")
                    .value_name("FILE")
                    .help("Input file to validate (use '-' for stdin)")
                    .required(true)
                    .index(1)
                    .value_parser(value_parser!(PathBuf)))
                .arg(Arg::new("format")
                    .short('f')
                    .long("format")
                    .value_name("FORMAT")
                    .help("Format of the input (toml or json)")
                    .default_value("toml")
                    .value_parser(["toml", "json"]))
        )
}
