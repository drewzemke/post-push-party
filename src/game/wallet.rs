use anyhow::Result;

use crate::storage::DbConnection;

pub trait Wallet {
    #[cfg(feature = "dev")]
    fn balance(&self) -> Result<u64>;

    fn earn(&self, points: u32) -> Result<()>;

    #[expect(dead_code)]
    fn spend(&self, points: u32) -> Result<()>;
}

pub struct GameWallet<'a> {
    conn: &'a DbConnection,
}

impl<'a> GameWallet<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }
}

impl<'a> Wallet for GameWallet<'a> {
    fn balance(&self) -> Result<u64> {
        let points: i64 =
            self.conn
                .query_one("SELECT party_points FROM player WHERE id = 1", (), |row| {
                    row.get(0)
                })?;
        Ok(points as u64)
    }

    fn earn(&self, points: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE player SET party_points = party_points + ?1 WHERE id = 1",
            (points,),
        )?;
        Ok(())
    }

    fn spend(&self, points: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE player SET party_points = MAX(party_points - ?1, 0) WHERE id = 1",
            (points,),
        )?;
        Ok(())
    }
}
