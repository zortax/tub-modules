use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

use moses_scraper::{search, runner::{ScraperConfig, ScraperEvent, run_scraper}};

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
    match db::run_migrations(&pool).await {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => {
            let err_msg = e.to_string();
            // Allow "already applied but modified" errors since we may have edited migrations
            if err_msg.contains("previously applied") || err_msg.contains("has been modified") {
                println!("{} (skipped, already applied)", "✓".yellow());
            } else {
                return Err(e.into());
            }
        }
    }
    println!();

    // Create scraping run
    print!("Creating scraping run... ");
    let scraping_run_id = sqlx::query!(
        r#"
        INSERT INTO scraping_run (status, total_modules)
        VALUES ('in_progress', 0)
        RETURNING id
        "#
    )
    .fetch_one(&pool)
    .await?
    .id;
    println!("{} (run_id: {})", "✓".green(), scraping_run_id.to_string().bright_yellow());
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

    // Update scraping run with total module count
    sqlx::query!(
        "UPDATE scraping_run SET total_modules = $1 WHERE id = $2",
        modules.len() as i32,
        scraping_run_id
    )
    .execute(&pool)
    .await?;

    println!();

    // Determine number of parallel workers
    let num_workers = args.workers.unwrap_or_else(num_cpus::get);
    println!(
        "{} Using {} parallel workers",
        "→".bright_blue(),
        num_workers.to_string().bright_yellow()
    );
    println!();

    // Setup progress bar
    let progress = Arc::new(ProgressBar::new(modules.len() as u64));
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Setup scraper config
    let config = ScraperConfig {
        retries: args.retries,
        num_workers,
    };

    let pool = Arc::new(pool);
    let progress_clone = Arc::clone(&progress);

    // Run scraper with event handling
    let result = run_scraper(
        pool.clone(),
        modules,
        scraping_run_id,
        config,
        move |event| {
            match event {
                ScraperEvent::Started { .. } => {
                    // Already printed
                }
                ScraperEvent::Progress { current, .. } => {
                    progress_clone.set_position(current as u64);
                }
                ScraperEvent::ModuleSuccess { number, version, title } => {
                    progress_clone.println(format!(
                        "{} {} v{}: {}",
                        "✓".green(),
                        number.to_string().bright_yellow(),
                        version.to_string().bright_yellow(),
                        title.bright_white()
                    ));
                }
                ScraperEvent::ModuleSkipped { number, version, reason } => {
                    progress_clone.println(format!(
                        "{} {} v{} {}",
                        "⊘".yellow(),
                        number.to_string().bright_black(),
                        version.to_string().bright_black(),
                        format!("({})", reason).bright_black()
                    ));
                }
                ScraperEvent::ModuleFailed { number, version, error } => {
                    progress_clone.println(format!(
                        "{} {} v{}: {}",
                        "✗".red(),
                        number,
                        version,
                        error.red()
                    ));
                }
                ScraperEvent::Completed { .. } => {
                    // Will be handled below
                }
            }
        },
    )
    .await?;

    progress.finish_and_clear();

    let successful = result.successful;
    let failed = result.failed;
    let skipped = result.skipped;

    // Update scraping run with final statistics
    sqlx::query!(
        r#"
        UPDATE scraping_run
        SET completed_at = NOW(),
            status = 'completed',
            successful_modules = $1,
            failed_modules = $2,
            skipped_modules = $3
        WHERE id = $4
        "#,
        successful as i32,
        failed as i32,
        skipped as i32,
        scraping_run_id
    )
    .execute(&*pool)
    .await?;

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
