use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::{Semaphore, Mutex};

mod search;
mod module;
mod mapper;
mod db_ops;
mod models;

#[derive(Parser, Debug)]
#[command(name = "moses-scraper")]
#[command(about = "Scrape TU Berlin MOSES module data and populate database", long_about = None)]
struct Args {
    /// Base URL for module search
    #[arg(
        long,
        default_value = "https://moseskonto.tu-berlin.de/moses/modultransfersystem/bolognamodule/suchen.html"
    )]
    url: String,

    /// Semester number to scrape
    #[arg(short, long, default_value = "75")]
    semester: u32,

    /// Limit number of modules to scrape (for testing)
    #[arg(short, long)]
    limit: Option<usize>,

    /// Number of retry attempts for failed requests
    #[arg(long, default_value = "3")]
    retries: u32,

    /// Number of parallel workers for concurrent processing
    #[arg(short = 'j', long)]
    workers: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load environment variables
    dotenvy::dotenv().ok();

    println!("{}", "=".repeat(80).bright_blue());
    println!("{}", "MOSES Module Scraper".bright_cyan().bold());
    println!("{}", "=".repeat(80).bright_blue());
    println!();

    // Connect to database
    print!("Connecting to database... ");
    let pool = db::create_pool_from_env().await?;
    println!("{}", "✓".green());

    // Run migrations
    print!("Running database migrations... ");
    db::run_migrations(&pool).await?;
    println!("{}", "✓".green());
    println!();

    // Load modules from CSV export file
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    spinner.set_message("Loading modules from CSV export...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let modules = search::fetch_all_modules("", args.limit).await?;

    spinner.finish_and_clear();
    println!(
        "{} Found {} modules",
        "✓".green(),
        modules.len().to_string().bright_yellow()
    );
    println!();

    // Determine number of parallel workers
    let num_workers = args.workers.unwrap_or_else(num_cpus::get);
    println!(
        "{} Using {} parallel workers",
        "→".bright_blue(),
        num_workers.to_string().bright_yellow()
    );
    println!();

    // Process modules in parallel
    let successful = Arc::new(Mutex::new(0_usize));
    let failed = Arc::new(Mutex::new(0_usize));
    let skipped = Arc::new(Mutex::new(0_usize));

    let progress = Arc::new(ProgressBar::new(modules.len() as u64));
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let pool = Arc::new(pool);
    let semaphore = Arc::new(Semaphore::new(num_workers));

    let mut tasks = Vec::new();

    for module_ref in modules {
        let pool = Arc::clone(&pool);
        let progress = Arc::clone(&progress);
        let successful = Arc::clone(&successful);
        let failed = Arc::clone(&failed);
        let skipped = Arc::clone(&skipped);
        let semaphore = Arc::clone(&semaphore);
        let retries = args.retries;

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            progress.set_message(format!(
                "Processing {} v{}",
                module_ref.number, module_ref.version
            ));

            match process_module(&pool, &module_ref, retries).await {
                Ok(true) => {
                    *successful.lock().await += 1;
                    progress.println(format!(
                        "{} {} v{}: {}",
                        "✓".green(),
                        module_ref.number.to_string().bright_yellow(),
                        module_ref.version.to_string().bright_yellow(),
                        module_ref.title.bright_white()
                    ));
                }
                Ok(false) => {
                    *skipped.lock().await += 1;
                    progress.println(format!(
                        "{} {} v{} {}",
                        "⊘".yellow(),
                        module_ref.number.to_string().bright_black(),
                        module_ref.version.to_string().bright_black(),
                        "(auth required)".bright_black()
                    ));
                }
                Err(e) => {
                    *failed.lock().await += 1;
                    progress.println(format!(
                        "{} {} v{}: {}",
                        "✗".red(),
                        module_ref.number,
                        module_ref.version,
                        e.to_string().red()
                    ));
                }
            }

            progress.inc(1);
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    futures::future::join_all(tasks).await;

    progress.finish_and_clear();

    let successful = *successful.lock().await;
    let failed = *failed.lock().await;
    let skipped = *skipped.lock().await;

    // Print summary
    println!();
    println!("{}", "=".repeat(80).bright_blue());
    println!("{}", "Summary".bright_cyan().bold());
    println!("{}", "=".repeat(80).bright_blue());
    println!("  {} modules processed successfully", successful.to_string().green().bold());
    println!("  {} modules skipped (auth required)", skipped.to_string().yellow().bold());
    println!("  {} modules failed", failed.to_string().red().bold());
    println!();

    Ok(())
}

async fn process_module(
    pool: &sqlx::PgPool,
    module_ref: &search::ModuleRef,
    retries: u32,
) -> Result<bool> {
    // Fetch module details
    let scraped_module = match module::fetch_module_details(&module_ref.detail_url, retries).await? {
        Some(m) => m,
        None => return Ok(false), // Authentication required
    };

    // Map to database models
    let mapped_data = mapper::map_module_data(pool, scraped_module).await?;

    // Insert into database
    db_ops::insert_module_data(pool, mapped_data).await?;

    Ok(true)
}
