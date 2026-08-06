use std::net::SocketAddr;

use clap::Parser;
use colored::Colorize;
use integration_tests_sv2::start_tracing;
use interoperability_tests_sv2::{
    app_type::AppType,
    config::{Config, Target},
    endpoint::Endpoint,
    runner::run_suite,
    scenarios::ScenarioReport,
};

#[derive(Parser)]
#[command(
    name = "sv2-compliance",
    about = "Sv2 interoperability compliance tester"
)]
struct Cli {
    /// Sv2 endpoint address (host:port)
    #[arg(long, required_unless_present = "config", conflicts_with = "config")]
    endpoint: Option<SocketAddr>,

    /// Type of Sv2 application: solo-pool, pool, jds, tp
    #[arg(long, default_value = "solo-pool")]
    app_type: AppTypeArg,

    /// `user_identity` used when opening mining channels
    #[arg(long, default_value = "interoperability_tests_sv2")]
    user_identity: String,

    /// Path to a TOML config file listing sites to test
    #[arg(long)]
    config: Option<String>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, clap::ValueEnum)]
enum AppTypeArg {
    SoloPool,
    Pool,
    Jds,
    Tp,
}

impl From<AppTypeArg> for AppType {
    fn from(arg: AppTypeArg) -> Self {
        match arg {
            AppTypeArg::SoloPool => AppType::SoloPool,
            AppTypeArg::Pool => AppType::Pool,
            AppTypeArg::Jds => AppType::JobDeclaratorServer,
            AppTypeArg::Tp => AppType::TemplateProvider,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.verbose {
        start_tracing();
    }

    let targets: Vec<Target> = if let Some(config_path) = &cli.config {
        match Config::from_path(config_path) {
            Ok(config) => config.targets(),
            Err(e) => {
                eprintln!("{} {e}", "error:".red().bold());
                std::process::exit(1);
            }
        }
    } else {
        let addr = cli.endpoint.expect("endpoint required without --config");
        vec![Target {
            site_name: addr.to_string(),
            endpoint: Endpoint {
                addr,
                app_type: cli.app_type.into(),
                user_identity: cli.user_identity,
            },
        }]
    };

    if targets.is_empty() {
        eprintln!("{} no endpoints configured to test", "error:".red().bold());
        std::process::exit(1);
    }

    let mut passed = 0u32;
    let mut failed = 0u32;
    for target in &targets {
        println!(
            "── {} ──",
            format!("{:?} at {}", target.endpoint.app_type, target.endpoint.addr)
                .blue()
                .bold()
        );
        let reports = run_suite(&target.endpoint).await;
        for report in &reports {
            print_report(report);
            match &report.result {
                Ok(_) => passed += 1,
                Err(_) => failed += 1,
            }
        }
        println!();
    }

    let total = passed + failed;
    if failed == 0 {
        println!(
            "{} {}/{} scenarios passed across {} target(s)",
            "✓".green().bold(),
            passed,
            total,
            targets.len()
        );
    } else {
        println!(
            "{} {}/{} scenarios passed, {} failed across {} target(s)",
            "✗".red().bold(),
            passed,
            total,
            failed,
            targets.len()
        );
        std::process::exit(1);
    }
}

fn print_report(report: &ScenarioReport) {
    match &report.result {
        Ok(detail) => {
            let detail = detail
                .as_deref()
                .map(|d| format!(" ({})", d.dimmed()))
                .unwrap_or_default();
            println!("  {} {}{detail}", "✓".green(), report.id.blue());
        }
        Err(e) => println!(
            "  {} {}: {}",
            "✗".red(),
            report.id.blue(),
            e.to_string().red()
        ),
    }
}
