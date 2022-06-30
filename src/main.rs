fn main() {
    let cmd = clap::Command::new("nacos-perf-utils")
        .bin_name("nacos-perf-utils")
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("instance")
                .about("instance register")
                .arg(clap::arg!(<nacos> "The remote nacos"))
                .arg(clap::Arg::new("port")
                    .short('p')
                    .long("start-port")
                    .help("nacos mock client start port number, default 10000")
                    .exclusive(true)
                    .takes_value(true)
                    .default_missing_value("10000"))
                .arg(clap::Arg::new("number")
                    .short('n')
                    .long("instance-number")
                    .help("nacos mock client number")
                    .exclusive(true)
                    .takes_value(true)
                    .default_missing_value("1"))
                .arg_required_else_help(true),
        );
    let matches = cmd.get_matches();
    let matches = match matches.subcommand() {
        Some(("instance", matches)) => matches,
        _ => unreachable!("clap should ensure we don't get here"),
    };
    let nacos = matches.get_one::<&str>("nacos");
    println!("{:?}", nacos);
}
