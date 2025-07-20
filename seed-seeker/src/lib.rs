use clap::Parser;
use colored_json::ToColoredJson;
use rand::{rng, RngCore};
use std::io::BufRead;
use subprocess::{PopenConfig, Redirection};

fn default_fdbserver_path() -> String {
    String::from("/usr/sbin/fdbserver")
}

#[derive(clap::Parser, Debug)]
struct Cli {
    #[clap(long, default_value_t = default_fdbserver_path())]
    fdbserver_path: String,
    #[clap(long, short = 'f')]
    test_file: String,
    max_iterations: Option<u64>,
    #[clap(long)]
    seed: Option<u32>,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = PopenConfig {
        stdout: Redirection::Pipe,
        stderr: Redirection::Pipe,
        ..Default::default()
    };

    let mut rng = rng();

    let seed = cli.seed.unwrap_or_else(|| rng.next_u32());
    let data_dir = tempfile::tempdir()?;

    let simfdb_data_dir = data_dir.path().join("simfdb");
    let logs_dir = data_dir.path().join("logs");

    std::fs::create_dir_all(&logs_dir)?;

    let mut process = subprocess::Popen::create(
        &[
            cli.fdbserver_path.as_str(),
            "-r",
            "simulation",
            "-b",
            "on",
            "--trace-format",
            "json",
            "-f",
            cli.test_file.as_str(),
            "-d",
            simfdb_data_dir
                .to_str()
                .expect("failed to get simfdb data dir path"),
            "-L",
            logs_dir.to_str().expect("failed to get logs dir path"),
            "-s",
            &seed.to_string(),
        ],
        config,
    )?;

    let (out, err) = process.communicate(None)?;

    let Some(exit_status) = process.poll() else {
        process.terminate()?;
        return Err("Failed to terminate process".into());
    };

    println!("{:?}", exit_status);

    println!("seed: {seed}");

    //println!("{out:?}");

    let mut compiled = jq_rs::compile(r#"select(.Layer=="Rust") | select(.Severity=="40")"#)?;

    for file in walkdir::WalkDir::new(logs_dir) {
        let file = file?;
        if file.path().extension().unwrap_or_default() == "json" {
            let file = std::fs::File::open(file.path())?;
            let reader = std::io::BufReader::new(file);

            for line in reader.lines() {
                let logs = compiled.run(&line?)?;
                if logs.is_empty() {
                    continue;
                }
                let pretty = jsonxf::pretty_print(&logs)?.to_colored_json_auto()?;
                println!("{pretty}");
            }
        }
    }

    Ok(())
}
