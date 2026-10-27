mod classifier;
mod db;

use clap::{Parser, Subcommand};
use db::Room;
use serde::Deserialize;
use std::{error::Error, io};

#[derive(Parser)]
#[command(
    name = "search-lab",
    version,
    about = "Offline search for cybersecurity labs"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the number of saved rooms
    Stats,

    /// Search rooms by name, description, or category
    Search {
        query: String,

        /// Filter by Free, Premium, or Unknown
        #[arg(long)]
        access: Option<String>,
    },

    /// List saved rooms, optionally filtered by access type
    List {
        #[arg(long)]
        access: Option<String>,
    },

    /// Add or update a room manually
    Add {
        #[arg(long)]
        id: String,

        #[arg(long)]
        name: String,

        #[arg(long)]
        url: String,

        #[arg(long, default_value = "Unknown")]
        difficulty: String,

        #[arg(long, default_value = "Unknown")]
        access: String,

        #[arg(long, default_value = "")]
        description: String,

        /// Extra categories separated by commas
        #[arg(long, value_delimiter = ',')]
        category: Vec<String>,
    },

    /// Import room metadata from a local CSV file
    Import {
        #[arg(long)]
        file: String,
    },
}

fn normalize_access(value: &str) -> Result<String, Box<dyn Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "free" => Ok("Free".into()),
        "premium" => Ok("Premium".into()),
        "unknown" | "" => Ok("Unknown".into()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Access must be Free, Premium, or Unknown",
        )
        .into()),
    }
}

fn add_categories(target: &mut Vec<String>, incoming: Vec<String>) {
    for category in incoming {
        let category = category.trim().to_string();

        if !category.is_empty()
            && !target
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&category))
        {
            target.push(category);
        }
    }
}

fn print_rooms(rooms: &[Room]) {
    if rooms.is_empty() {
        println!("No rooms found.");
        return;
    }

    for room in rooms {
        println!("----------------------------------------");
        println!("Name:        {}", room.name);
        println!("ID:          {}", room.id);
        println!("Difficulty:  {}", room.difficulty);
        println!("Access:      {}", room.access_type);
        println!(
            "Categories:  {}",
            if room.categories.is_empty() {
                "Unclassified".to_string()
            } else {
                room.categories.join(", ")
            }
        );

        if !room.description.is_empty() {
            println!("Description: {}", room.description);
        }

        println!("URL:         {}", room.url);
    }

    println!("----------------------------------------");
    println!("Found {} room(s).", rooms.len());
}

#[derive(Debug, Deserialize)]
struct CsvRoom {
    id: String,
    name: String,
    url: String,

    #[serde(default)]
    difficulty: String,

    #[serde(default, alias = "access_type")]
    access: String,

    #[serde(default)]
    description: String,

    #[serde(default)]
    categories: String,
}

fn import_csv(conn: &rusqlite::Connection, path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;

    let mut imported = 0usize;

    for (index, result) in reader.deserialize::<CsvRoom>().enumerate() {
        let record = result.map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("CSV data row {}: {error}", index + 2),
            )
        })?;

        if record.id.trim().is_empty()
            || record.name.trim().is_empty()
            || record.url.trim().is_empty()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("CSV data row {} needs id, name, and url", index + 2),
            )
            .into());
        }

        let access_type = normalize_access(&record.access)?;
        let classification =
            classifier::classify(&format!("{} {}", record.name, record.description));

        eprintln!(
            "Classifier confidence: {:.0}% | evidence: {}",
            classification.confidence * 100.0,
            if classification.evidence.is_empty() {
                "none".to_string()
            } else {
                classification.evidence.join(", ")
            }
        );
        let mut categories = classification.categories;

        // In the CSV, separate multiple categories with | or ;.
        add_categories(
            &mut categories,
            record
                .categories
                .split(|c| c == '|' || c == ';')
                .map(str::to_string)
                .collect(),
        );

        let room = Room {
            id: record.id,
            name: record.name,
            url: record.url,
            difficulty: if record.difficulty.trim().is_empty() {
                "Unknown".into()
            } else {
                record.difficulty
            },
            access_type,
            description: record.description,
            categories,
        };

        db::save(conn, &room)?;
        imported += 1;
    }

    println!("CSV rows imported or updated: {imported}");
    println!("Total rooms in database: {}", db::count(conn)?);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let conn = db::open()?;

    match cli.command {
        Commands::Stats => {
            println!("Saved rooms: {}", db::count(&conn)?);
        }

        Commands::Search { query, access } => {
            let access = access.map(|value| normalize_access(&value)).transpose()?;
            let rooms = db::search(&conn, &query, access.as_deref())?;
            print_rooms(&rooms);
        }

        Commands::List { access } => {
            let access = access.map(|value| normalize_access(&value)).transpose()?;
            let rooms = db::search(&conn, "", access.as_deref())?;
            print_rooms(&rooms);
        }

        Commands::Add {
            id,
            name,
            url,
            difficulty,
            access,
            description,
            category,
        } => {
            let access_type = normalize_access(&access)?;
            let classification = classifier::classify(&format!("{name} {description}"));
            eprintln!(
                "Classifier confidence: {:.0}% | evidence: {}",
                classification.confidence * 100.0,
                if classification.evidence.is_empty() {
                    "none".to_string()
                } else {
                    classification.evidence.join(", ")
                }
            );
            let mut categories = classification.categories;
            add_categories(&mut categories, category);

            let room = Room {
                id,
                name,
                url,
                difficulty,
                access_type,
                description,
                categories,
            };

            db::save(&conn, &room)?;
            println!("Room saved: {}", room.name);
            println!("Access: {}", room.access_type);
            println!("Categories: {}", room.categories.join(", "));
        }

        Commands::Import { file } => {
            import_csv(&conn, &file)?;
        }
    }

    Ok(())
}
