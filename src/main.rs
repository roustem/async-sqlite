use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqliteQueryAs;
use sqlx::{Result, SqliteConnection, SqlitePool};

type AccountId = i64;

#[derive(Debug, Clone, sqlx::FromRow)]
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
    pool: SqlitePool,
}

impl Db {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = SqlitePool::new(url).await?;
        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accounts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL
        )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn transaction(&self) -> Result<Tx> {
        let tx = self.pool.begin().await?;
        Ok(Tx { tx })
    }
}

pub struct Tx {
    tx: sqlx::Transaction<PoolConnection<SqliteConnection>>,
}

impl Tx {
    pub async fn add_account(&mut self, a: &Account) -> Result<AccountId> {
        let rowid: (AccountId,) = sqlx::query_as(
            r#"
        INSERT INTO accounts (name) VALUES ($1);
        SELECT last_insert_rowid();
        "#,
        )
        .bind(&a.name)
        .fetch_one(&mut self.tx)
        .await?;

        Ok(rowid.0)
    }

    pub async fn get_accounts(&mut self) -> Result<Vec<Account>> {
        let rows: Vec<Account> = sqlx::query_as("SELECT id, name FROM accounts")
            .fetch_all(&mut self.tx)
            .await?;

        Ok(rows)
    }

    pub async fn commit(self) -> Result<()> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<()> {
        self.tx.rollback().await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut path = std::env::current_dir()?;
    path.push("test.sqlite");

    let db = Db::new(path.to_str().unwrap()).await?;
    db.init().await?;

    let mut tx = db.transaction().await?;
    tx.add_account(&Account::new("Account A1")).await?;
    tx.add_account(&Account::new("Account A2")).await?;
    tx.add_account(&Account::new("Account A3")).await?;

    let accounts = tx.get_accounts().await?;
    tx.commit().await?;

    for account in accounts {
        println!("{:03} - {}", &account.id, &account.name);
    }

    Ok(())
}
