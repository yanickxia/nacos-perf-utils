use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod nacos;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let cmd = clap::Command::new("nacos-perf-utils")
        .bin_name("nacos-perf-utils")
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("instance")
                .about("instance register")
                .arg(clap::arg!(<nacos> "The remote nacos"))
                .arg(
                    clap::Arg::new("port")
                        .short('p')
                        .long("start-port")
                        .help("nacos mock client start port number, default 10000")
                        .default_missing_value("10000"),
                )
                .arg(
                    clap::Arg::new("number")
                        .short('n')
                        .long("instance-number")
                        .help("nacos mock client number")
                        .default_missing_value("1"),
                )
                .arg_required_else_help(true),
        );
    let matches = cmd.get_matches();
    let matches = match matches.subcommand() {
        Some(("instance", matches)) => matches,
        _ => unreachable!("clap should ensure we don't get here"),
    };
    let nacos = matches.get_one::<String>("nacos").unwrap();
    let port = matches
        .get_one::<String>("port")
        .unwrap()
        .to_string()
        .parse::<u32>()
        .unwrap();
    let num = matches
        .get_one::<String>("number")
        .unwrap()
        .to_string()
        .parse::<u32>()
        .unwrap();

    let mut runtime = nacos::nacos::Runtime::default();
    runtime.run(nacos.clone(), port, num).await?;

    println!("Waiting for Ctrl-C...");
    while running.load(Ordering::SeqCst) {}
    println!("Got it! Exiting...");
    Ok(())
}
