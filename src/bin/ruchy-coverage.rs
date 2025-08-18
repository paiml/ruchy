//! Ruchy coverage reporting tool
//!
//! This binary runs test coverage analysis and generates reports

use clap::{Arg, Command};
use ruchy::quality::{CoverageCollector, CoverageTool, HtmlReportGenerator, QualityGates};
use std::process;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    let matches = Command::new("ruchy-coverage")
        .version("0.3.2")
        .author("Ruchy Contributors")
        .about("Ruchy test coverage reporting tool")
        .arg(
            Arg::new("tool")
                .long("tool")
                .help("Coverage tool to use")
                .value_name("TOOL")
                .value_parser(["tarpaulin", "grcov", "llvm"])
                .default_value("tarpaulin"),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .help("Output directory for reports")
                .value_name("DIR")
                .default_value("target/coverage"),
        )
        .arg(
            Arg::new("quality-gates")
                .long("quality-gates")
                .help("Run quality gates check")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let tool_name = matches.get_one::<String>("tool").unwrap();
    let output_dir = matches.get_one::<String>("output").unwrap();
    let run_quality_gates = matches.get_flag("quality-gates");

    let coverage_tool = match tool_name.as_str() {
        "tarpaulin" => CoverageTool::Tarpaulin,
        "grcov" => CoverageTool::Grcov,
        "llvm" => CoverageTool::Llvm,
        _ => {
            error!("Unknown coverage tool: {}", tool_name);
            process::exit(1);
        }
    };

    info!("Running coverage analysis with {}", tool_name);

    // Collect coverage
    let collector = CoverageCollector::new(coverage_tool);

    if !collector.is_available() {
        error!("Coverage tool '{}' is not available", tool_name);
        println!("Please install the tool first:");
        match tool_name.as_str() {
            "tarpaulin" => println!("  cargo install cargo-tarpaulin"),
            "grcov" => println!("  cargo install grcov"),
            "llvm" => println!("  Install LLVM tools for your platform"),
            _ => {}
        }
        process::exit(1);
    }

    let coverage_report = match collector.collect() {
        Ok(report) => report,
        Err(err) => {
            error!("Failed to collect coverage: {}", err);
            process::exit(1);
        }
    };

    // Generate HTML report
    let html_generator = HtmlReportGenerator::new(output_dir);
    if let Err(err) = html_generator.generate(&coverage_report) {
        error!("Failed to generate HTML report: {}", err);
        process::exit(1);
    }

    // Print summary
    println!("Coverage Analysis Complete!");
    println!("==========================");
    println!(
        "  Lines: {:.1}% ({}/{})",
        coverage_report.line_coverage_percentage(),
        coverage_report.covered_lines,
        coverage_report.total_lines
    );
    println!(
        "  Functions: {:.1}% ({}/{})",
        coverage_report.function_coverage_percentage(),
        coverage_report.covered_functions,
        coverage_report.total_functions
    );
    println!("  HTML Report: {}/coverage.html", output_dir);

    // Run quality gates if requested
    if run_quality_gates {
        println!("\nRunning Quality Gates...");
        let mut quality_gates = QualityGates::new();

        match quality_gates.collect_metrics() {
            Ok(_) => match quality_gates.check() {
                Ok(_) => println!("✅ All quality gates passed!"),
                Err(report) => {
                    println!("❌ Quality gate failures: {:?}", report);
                    process::exit(1);
                }
            },
            Err(err) => {
                error!("Failed to collect quality metrics: {}", err);
                process::exit(1);
            }
        }
    }
}
