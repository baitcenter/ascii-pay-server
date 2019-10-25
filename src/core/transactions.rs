use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::core::schema::transaction;
use crate::core::{generate_uuid, Account, DbConnection, Error, Money};

#[derive(Debug, Queryable, Insertable, Identifiable, AsChangeset)]
#[table_name = "transaction"]
pub struct Transaction {
    pub id: String,
    pub account: String,
    pub cashier: Option<String>,
    pub total: Money,
    pub date: NaiveDateTime,
}

pub fn execute(
    conn: &DbConnection,
    account: &mut Account,
    cashier: Option<&Account>,
    total: Money,
) -> Result<(), Error> {
    use crate::core::schema::transaction::dsl;
    
    let new_credit = account.credit + total;

    let result = conn.exclusive_transaction(|| {
        let mut account = Account::get(conn, &account.id)?;

        if new_credit < account.limit && new_credit < account.credit {
            return Err(Error::InternalServerError);
        }

        let a = Transaction {
            id: generate_uuid(),
            account: account.id.clone(),
            cashier: cashier.map(|c| c.id.clone()),
            total,
            date: chrono::Local::now().naive_local(),
        };
        account.credit = new_credit;

        diesel::insert_into(dsl::transaction)
            .values(&a)
            .execute(conn)?;

        account.update(conn)?;

        Ok(())
    });

    if result.is_ok() {
        account.credit = new_credit;
    }

    result
}

pub fn get_by_user(
    conn: &DbConnection,
    account: &Account,
    from: &NaiveDateTime,
    to: &NaiveDateTime,
) -> Result<Vec<Transaction>, Error> {
    use crate::core::schema::transaction::dsl;

    let results = dsl::transaction
        .filter(
            dsl::account
                .eq(account.id.to_string())
                .and(dsl::date.between(from, to)),
        )
        .load::<Transaction>(conn)?;

    Ok(results)
}
