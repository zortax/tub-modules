# TU Berlin Module Search

A web application for searching and exploring TU Berlin module descriptions (because Moses sucks ass).

> **Disclaimer**: This project was vibe-coded. No correctness guarantees
> are provided. Use at your own risk.

## Features

- Search and filter module catalog
- View detailed module information
- Save favorite modules (stored locally)
- Responsive design with mobile support

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- NodeJS/npm
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) (`cargo install cargo-leptos`)
- [sqlx-cli](https://crates.io/crates/sqlx-cli) (`cargo install sqlx-cli`)

## Setup

### 1. Start the Database

```bash 
docker-compose up -d
```

This starts a PostgreSQL database on `localhost:5432`.

### 2. Run Migrations

```bash 
cd db 
cargo sqlx migrate run 
cd ..
```

### 3. Scrape Module Data

First, export the module list from
[MOSES](https://moseskonto.tu-berlin.de/moses/modultransfersystem/bolognamodule/suchen.html)
as `Modul_export.csv` and place it in the project root.

Then run the scraper:

```bash 
cd moses-scraper 
cargo run --release -- -j 1
cd ..
```

This will fetch all module details from MOSES and populate the database. It may
take a while depending on the number of modules.

### 4. Start the Web Application

```bash
cd app
npm i
cargo leptos watch
```

The app will be available at [http://localhost:3000](http://localhost:3000).

## Development

The project consists of three main crates:

- **`db`**: Database models and migrations
- **`moses-scraper`**: Web scraper for fetching module data from MOSES
- **`app`**: Leptos web application (frontend + backend)

## Tech Stack

- **Frontend**: [Leptos](https://leptos.dev/) (Rust web framework)
- **Backend**: [Actix Web](https://actix.rs/)
- **Database**: PostgreSQL with [sqlx](https://github.com/launchbadge/sqlx)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) +
[DaisyUI](https://daisyui.com/)

## License

This project is provided as-is with no warranty or guarantees.
