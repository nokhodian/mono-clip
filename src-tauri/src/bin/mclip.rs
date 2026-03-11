use clap::{Parser, Subcommand};
use rusqlite::{Connection, params};
use std::path::PathBuf;

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "mclip",
    about = "MonoClip CLI — manage your clipboard history from the terminal",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List clipboard items
    List {
        /// Show items in a specific folder (default: Inbox)
        #[arg(short, long)]
        folder: Option<String>,
        /// Filter by content
        #[arg(short, long)]
        search: Option<String>,
        /// Maximum number of items to show (default: 20)
        #[arg(short, long, default_value = "20")]
        limit: i64,
    },
    /// Add a new clipboard item
    Add {
        /// The text content to add
        content: String,
        /// Destination folder name (default: Inbox)
        #[arg(short, long)]
        folder: Option<String>,
    },
    /// Remove a clipboard item by ID
    Remove {
        /// Item ID (from `mclip list`)
        id: i64,
    },
    /// Pin a clipboard item
    Pin {
        id: i64,
    },
    /// Unpin a clipboard item
    Unpin {
        id: i64,
    },
    /// Print the raw content of an item (pipe-friendly)
    Get {
        id: i64,
    },
    /// Manage folders
    Folder {
        #[command(subcommand)]
        action: FolderCmd,
    },
}

#[derive(Subcommand)]
enum FolderCmd {
    /// List all folders
    List,
    /// Create a new folder
    Add {
        name: String,
        #[arg(short, long, default_value = "📋")]
        icon: String,
        #[arg(short, long, default_value = "#6366f1")]
        color: String,
    },
    /// Delete a folder (items are moved to Inbox)
    Remove {
        name: String,
    },
}

// ── Database helpers ──────────────────────────────────────────────────────────

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".monoclip").join("monoclip.db")
}

fn open_db() -> Connection {
    let path = db_path();
    if !path.exists() {
        eprintln!("error: MonoClip database not found at {}", path.display());
        eprintln!("       Launch MonoClip.app at least once to create it.");
        std::process::exit(1);
    }
    let conn = Connection::open(&path).unwrap_or_else(|e| {
        eprintln!("error: failed to open database: {}", e);
        std::process::exit(1);
    });
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;",
    )
    .ok();
    conn
}

fn resolve_folder_id(conn: &Connection, name: &str) -> i64 {
    if name.eq_ignore_ascii_case("inbox") {
        return 1;
    }
    let result = conn.query_row(
        "SELECT id FROM folders WHERE LOWER(name) = LOWER(?1) LIMIT 1",
        params![name],
        |r| r.get::<_, i64>(0),
    );
    match result {
        Ok(id) => id,
        Err(_) => {
            eprintln!("error: folder '{}' not found", name);
            std::process::exit(1);
        }
    }
}

fn detect_type(content: &str) -> &'static str {
    let t = content.trim();
    if t.starts_with("http://") || t.starts_with("https://") {
        return "url";
    }
    if t.contains('@') && t.contains('.') && !t.contains(' ') {
        return "email";
    }
    if (t.starts_with('#') && (t.len() == 4 || t.len() == 7))
        && t[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return "color";
    }
    let lines: Vec<&str> = t.lines().collect();
    if lines.len() > 1 {
        let code_chars = t.chars().filter(|c| matches!(c, '{' | '}' | ';' | '(' | ')')).count();
        if code_chars > 3 {
            return "code";
        }
    }
    "text"
}

fn make_preview(content: &str, max: usize) -> String {
    let t = content.trim();
    if t.len() <= max {
        t.replace('\n', " ")
    } else {
        format!("{}…", &t[..max].replace('\n', " "))
    }
}

// ── Formatting helpers ────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max - 1).collect::<String>())
    }
}

fn relative_time(updated_at: &str) -> String {
    // updated_at is stored as SQLite datetime: "2024-01-15 12:34:56"
    let now = chrono::Local::now().naive_local();
    let parsed = chrono::NaiveDateTime::parse_from_str(updated_at, "%Y-%m-%d %H:%M:%S")
        .unwrap_or(now);
    let diff = now.signed_duration_since(parsed);
    let secs = diff.num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", diff.num_minutes())
    } else if secs < 86400 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}

// ── Commands ──────────────────────────────────────────────────────────────────

fn cmd_list(folder: Option<String>, search: Option<String>, limit: i64) {
    let conn = open_db();
    let folder_id = folder.as_deref().map(|f| resolve_folder_id(&conn, f));

    let where_folder = match folder_id {
        Some(id) => format!("AND folder_id = {}", id),
        None => String::new(),
    };
    let where_search = if search.is_some() {
        "AND (content LIKE ?1 OR preview LIKE ?1)"
    } else {
        ""
    };

    let sql = format!(
        "SELECT id, content_type, is_pinned, updated_at, preview, content
         FROM clip_items
         WHERE is_deleted = 0 {where_folder} {where_search}
         ORDER BY is_pinned DESC, updated_at DESC
         LIMIT {limit}"
    );

    println!("{:>5}  {:<6}  {:<4}  {:<12}  {}", "ID", "TYPE", "PIN", "UPDATED", "PREVIEW");
    println!("{}", "─".repeat(70));

    type Row = (i64, String, bool, String, String, String);

    let rows: Vec<Row> = if let Some(q) = search.as_deref() {
        let pattern = format!("%{}%", q);
        let mut stmt = conn.prepare(&sql).unwrap();
        stmt.query_map(params![pattern], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)? != 0,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .unwrap()
        .flatten()
        .collect()
    } else {
        let mut stmt = conn.prepare(&sql).unwrap();
        stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)? != 0,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .unwrap()
        .flatten()
        .collect()
    };

    if rows.is_empty() {
        println!("  (no items)");
        return;
    }
    for (id, ctype, pinned, updated_at, preview, content) in rows {
        let display = if !preview.is_empty() { preview } else { content };
        println!(
            "{:>5}  {:<6}  {:<4}  {:<12}  {}",
            id,
            truncate(&ctype, 6),
            if pinned { "📌" } else { "  " },
            truncate(&relative_time(&updated_at), 12),
            truncate(&display.replace('\n', " "), 46),
        );
    }
}

fn cmd_add(content: String, folder: Option<String>) {
    let conn = open_db();
    let folder_id = folder.as_deref().map(|f| resolve_folder_id(&conn, f)).unwrap_or(1);
    let ctype = detect_type(&content);
    let preview = make_preview(&content, 200);

    conn.execute(
        "INSERT INTO clip_items (content, content_type, preview, folder_id)
         VALUES (?1, ?2, ?3, ?4)",
        params![content, ctype, preview, folder_id],
    ).unwrap_or_else(|e| { eprintln!("error: {}", e); std::process::exit(1); });

    let id = conn.last_insert_rowid();
    let folder_name = folder.as_deref().unwrap_or("Inbox");
    println!("✓ Added clip #{} to {} ({})", id, folder_name, ctype);
}

fn cmd_remove(id: i64) {
    let conn = open_db();
    let affected = conn.execute(
        "UPDATE clip_items SET is_deleted = 1, updated_at = datetime('now') WHERE id = ?1 AND is_deleted = 0",
        params![id],
    ).unwrap_or(0);
    if affected == 0 {
        eprintln!("error: clip #{} not found", id);
        std::process::exit(1);
    }
    println!("✓ Removed clip #{}", id);
}

fn cmd_pin(id: i64, pin: bool) {
    let conn = open_db();
    let affected = conn.execute(
        "UPDATE clip_items SET is_pinned = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![pin as i64, id],
    ).unwrap_or(0);
    if affected == 0 {
        eprintln!("error: clip #{} not found", id);
        std::process::exit(1);
    }
    println!("✓ {} clip #{}", if pin { "Pinned" } else { "Unpinned" }, id);
}

fn cmd_get(id: i64) {
    let conn = open_db();
    let result = conn.query_row(
        "SELECT content FROM clip_items WHERE id = ?1 AND is_deleted = 0",
        params![id],
        |r| r.get::<_, String>(0),
    );
    match result {
        Ok(content) => print!("{}", content),
        Err(_) => {
            eprintln!("error: clip #{} not found", id);
            std::process::exit(1);
        }
    }
}

fn cmd_folder_list() {
    let conn = open_db();
    let mut stmt = conn.prepare(
        "SELECT f.id, f.icon, f.name, COUNT(c.id) as cnt
         FROM folders f
         LEFT JOIN clip_items c ON c.folder_id = f.id AND c.is_deleted = 0
         GROUP BY f.id ORDER BY f.position ASC, f.id ASC"
    ).unwrap();

    println!("{:>4}  {:<2}  {:<24}  {}", "ID", "", "NAME", "ITEMS");
    println!("{}", "─".repeat(40));
    for row in stmt.query_map([], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?))
    }).unwrap().flatten() {
        println!("{:>4}  {}  {:<24}  {}", row.0, row.1, truncate(&row.2, 24), row.3);
    }
}

fn cmd_folder_add(name: String, icon: String, color: String) {
    let conn = open_db();
    let max_pos: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), 0) FROM folders", [], |r| r.get(0)
    ).unwrap_or(0);
    conn.execute(
        "INSERT INTO folders (name, icon, color, position) VALUES (?1, ?2, ?3, ?4)",
        params![name, icon, color, max_pos + 1],
    ).unwrap_or_else(|e| { eprintln!("error: {}", e); std::process::exit(1); });
    let id = conn.last_insert_rowid();
    println!("✓ Created folder '{}' (id={})", name, id);
}

fn cmd_folder_remove(name: String) {
    let conn = open_db();
    let id = resolve_folder_id(&conn, &name);
    if id == 1 {
        eprintln!("error: cannot delete the Inbox folder");
        std::process::exit(1);
    }
    conn.execute(
        "UPDATE clip_items SET folder_id = 1, updated_at = datetime('now') WHERE folder_id = ?1",
        params![id],
    ).ok();
    conn.execute("DELETE FROM folders WHERE id = ?1", params![id]).ok();
    println!("✓ Deleted folder '{}', items moved to Inbox", name);
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Cmd::List { folder, search, limit } => cmd_list(folder, search, limit),
        Cmd::Add { content, folder } => cmd_add(content, folder),
        Cmd::Remove { id } => cmd_remove(id),
        Cmd::Pin { id } => cmd_pin(id, true),
        Cmd::Unpin { id } => cmd_pin(id, false),
        Cmd::Get { id } => cmd_get(id),
        Cmd::Folder { action } => match action {
            FolderCmd::List => cmd_folder_list(),
            FolderCmd::Add { name, icon, color } => cmd_folder_add(name, icon, color),
            FolderCmd::Remove { name } => cmd_folder_remove(name),
        },
    }
}
