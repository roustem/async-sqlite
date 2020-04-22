use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone)]
struct Account {
    id: i64,
    name: String,
}

fn main() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE accounts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL
    )",
        params![],
    )?;

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO accounts (name) VALUES (?)",
        params!["Account 1"],
    )?;
    tx.execute(
        "INSERT INTO accounts (name) VALUES (?)",
        params!["Account 2"],
    )?;
    tx.execute(
        "INSERT INTO accounts (name) VALUES (?)",
        params!["Account 4"],
    )?;
    tx.commit()?;

    let mut stmt = conn.prepare("SELECT id, name FROM accounts")?;
    let account_iter = stmt.query_map(params![], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    for a in account_iter {
        if let Ok(account) = a {
            println!("{:03} - {}", &account.id, &account.name);
        }
    }

    Ok(())
}
