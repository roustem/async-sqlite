use log;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Location {
    Memory,
    Persistent(PathBuf),
}

type AccountId = i64;

#[derive(Debug, Clone)]
pub struct Account {
    id: i64,
    name: String,
}

impl Account {
    pub fn new(name: &str) -> Self {
        Account {
            id: 0,
            name: name.to_string(),
        }
    }
}

struct Db {
    location: Location,
}

impl Db {
    pub fn new(location: Location) -> Self {
        Self { location }
    }

    pub fn init(&self) -> Result<()> {
        let conn = open_connection(&self.location)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL
    )",
            params![],
        )?;

        Ok(())
    }

    pub fn transaction(&self) -> Result<Transaction> {
        Ok(Transaction::new(&self.location)?)
    }
}

fn open_connection(location: &Location) -> Result<Connection> {
    Ok(match location {
        Location::Memory => Connection::open_in_memory(),
        Location::Persistent(path) => Connection::open(path),
    }?)
}

pub struct Transaction {
    conn: Connection,
}

impl Transaction {
    pub fn new(location: &Location) -> Result<Self> {
        let conn = open_connection(location)?;
        conn.execute_batch("BEGIN TRANSACTION")?;
        Ok(Self { conn })
    }

    pub fn add_account(&self, a: &Account) -> Result<AccountId> {
        self.conn
            .execute("INSERT INTO accounts (name) VALUES (?)", params![&a.name])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM accounts")?;
        let account_iter = stmt.query_map(params![], |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        Ok(account_iter.map(|a| a.unwrap()).collect())
    }

    pub fn commit(self) -> Result<()> {
        Ok(self.conn.execute_batch("COMMIT")?)
    }

    pub fn rollback(&self) -> Result<()> {
        Ok(self.conn.execute_batch("ROLLBACK")?)
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Err(e) = self.rollback() {
            log::error!("ROLLBACK failed: {}", e);
        }
    }
}

fn main() -> Result<()> {
    let db = Db::new(Location::Persistent(PathBuf::from("test.sqlite")));
    db.init()?;

    let tx = db.transaction()?;
    tx.add_account(&Account::new("Account A1"))?;
    tx.add_account(&Account::new("Account A2"))?;
    tx.add_account(&Account::new("Account A3"))?;

    let accounts = tx.get_accounts()?;
    tx.commit()?;

    for account in accounts {
        println!("{:03} - {}", &account.id, &account.name);
    }

    Ok(())
}
