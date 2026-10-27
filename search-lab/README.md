# Search Lab

A Rust command-line tool for searching cybersecurity lab metadata stored in
a local SQLite database.

## Features

- Offline, case-insensitive search
- Search by room name, description, or category
- Filter rooms by Free, Premium, or Unknown access
- Add or update rooms by ID
- Import room metadata from CSV
- Automatically suggest categories using keyword classification

## Requirements

Install Rust and Cargo from https://rustup.rs.

## Build and test

```bash
cargo fmt --all
cargo test
cargo check
cargo build --release
```

## Usage

Show available commands:

```bash
cargo run -- --help
```

Show the number of saved rooms:

```bash
cargo run -- stats
```

Search saved rooms:

```bash
cargo run -- search "Linux"
cargo run -- search "Kerberos"
cargo run -- search "Active Directory"
```

Filter by access type:

```bash
cargo run -- search "Linux" --access free
cargo run -- search "Kerberos" --access premium
cargo run -- list --access unknown
```

List saved rooms:

```bash
cargo run -- list
```

Add a room (replace these illustrative values with verified metadata):

```bash
cargo run -- add \
  --id "example-room" \
  --name "Example Linux Room" \
  --url "https://example.com/room/example-room" \
  --difficulty "Easy" \
  --access "Unknown" \
  --description "Example Linux enumeration lab" \
  --category "Linux,Networking & Enumeration"
```

Import a CSV file:

```bash
cargo run -- import --file data/rooms.csv
```

CSV header:

```csv
id,name,url,difficulty,access,description,categories
example-room,Example Room,https://example.com/room/example-room,Easy,Unknown,Example description,Linux|Networking
```

The sample row is illustrative only. Required columns are `id`, `name`, and
`url`. Optional columns are `difficulty`, `access` (or `access_type`),
`description`, and `categories`. Separate categories with `|` or `;`.

## Data storage and limitations

The local database is stored at `data/lets-search.db`. Search only returns
records already stored in that database. Add records manually or import
verified metadata from a source you are authorized to use.

Room details and access labels depend on the supplied data. Automatic
classification is keyword-based and may not always be accurate.

## Technologies

- Rust
- Clap
- SQLite / rusqlite
- Serde
- CSV

## License

No license has been selected yet. Add a license file if you intend to
distribute this project as open source.
