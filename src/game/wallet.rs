use anyhow::Result;

use crate::storage::DbConnection;

pub trait Wallet {
    #[cfg(feature = "dev")]
    fn balance(&self) -> Result<u64>;

    fn earn(&mut self, points: u64) -> Result<()>;

    #[expect(dead_code)]
    fn spend(&mut self, points: u64) -> Result<()>;
}

pub struct UserWallet<'a> {
    conn: &'a DbConnection,
}

impl<'a> UserWallet<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }
}

impl<'a> Wallet for UserWallet<'a> {
    #[cfg(feature = "dev")]
    fn balance(&self) -> Result<u64> {
        let points: i64 =
            self.conn
                .query_one("SELECT party_points FROM player WHERE id = 1", (), |row| {
                    row.get(0)
                })?;
        Ok(points as u64)
    }

    fn earn(&mut self, points: u64) -> Result<()> {
        self.conn.execute(
            "UPDATE player SET party_points = party_points + ?1 WHERE id = 1",
            (points as i64,),
        )?;
        Ok(())
    }

    fn spend(&mut self, points: u64) -> Result<()> {
        self.conn.execute(
            "UPDATE player SET party_points = MAX(party_points - ?1, 0) WHERE id = 1",
            (points as i64,),
        )?;
        Ok(())
    }
}

#[cfg(feature = "dev")]
pub struct MemoryWallet {
    balance: u64,
}

#[cfg(feature = "dev")]
impl MemoryWallet {
    pub fn new(balance: u64) -> Self {
        Self { balance }
    }
}

#[cfg(feature = "dev")]
impl Wallet for MemoryWallet {
    fn balance(&self) -> Result<u64> {
        Ok(self.balance)
    }

    fn earn(&mut self, points: u64) -> Result<()> {
        self.balance += points;
        Ok(())
    }

    fn spend(&mut self, points: u64) -> Result<()> {
        self.balance = self.balance.saturating_sub(points);
        Ok(())
    }
}
