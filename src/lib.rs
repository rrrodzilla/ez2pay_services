#![allow(dead_code)]
use pickledb::{PickleDb, PickleDbDumpPolicy};
use serde::{Deserialize, Serialize};

//our account consists of a phone number, it's always tied to
//the phone used to contact the service
#[derive(Serialize, Deserialize)]
pub struct Account {
    phone_number: String,
    stripe_id: String,
    subscriber: bool,
}

impl Account {
    //we're going to grab the existing account if it exists,
    //otherwise we're going to create one and return it
    fn get(id: String) -> Self {
        let mut db = get_db("accounts");
        //first try and get it
        let account = match db.get(&id) {
            Some(a) => a,
            None => {
                let acct = Account {
                    phone_number: id.clone(),
                    stripe_id: String::from(""),
                    subscriber: false,
                };
                db.set(&id, &acct).unwrap();
                acct
            }
        };

        account
    }

    //persist updates
    fn update(&self) {
        let mut db = get_db("accounts");
        db.set(&self.phone_number, &self).unwrap();
        ()
    }
}

fn get_db(salt: &str) -> PickleDb {
    let db_name = format!("{}.db", salt);
    //first load the db if it exists, else create a new one
    let db = match PickleDb::load_json(&db_name, PickleDbDumpPolicy::AutoDump) {
        Ok(v) => v,
        Err(_e) => PickleDb::new_json(format!("{}.db", salt), PickleDbDumpPolicy::AutoDump),
    };
    db
}

pub struct Customer {}
pub struct Order {}

#[derive(Serialize, Deserialize)]
pub struct Product {
    name: String,
    description: String,
    price: f32,
    tax: f32,
    status: ProductStatus,
    account: Account,
}

#[derive(Serialize, Deserialize)]
pub enum ProductStatus {
    New,
    Published,
    Unpublished,
}

pub fn test() {
    println!("Test");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account() {
        let p = String::from("+12063832022");
        let sid = String::from("stripe_id");
        let mut a = Account::get(p.clone());
        assert_eq!(p, a.phone_number);
        assert_eq!(false, a.subscriber);

        //test the update
        a.stripe_id = sid.clone();
        a.update();
        let updated = Account::get(a.phone_number);
        assert_eq!(sid, updated.stripe_id);
    }
}
