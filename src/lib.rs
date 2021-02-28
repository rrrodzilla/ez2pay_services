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
}

fn get_db(salt: &str) -> PickleDb {
    let db_name = format!("{}.db", salt);
    //first load the db if it exists, else create a new one
    let mut db = match PickleDb::load_json(&db_name, PickleDbDumpPolicy::AutoDump) {
        Ok(v) => v,
        Err(_e) => PickleDb::new_json(format!("{}.db", salt), PickleDbDumpPolicy::AutoDump),
    };
    db
}

pub struct Customer {}
pub struct Order {}
pub struct Product {}

pub fn test() {
    println!("Test");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account() {
        let p = String::from("+12063832022");
        let a = Account::get(p.clone());
        assert_eq!(p, a.phone_number);
        assert_eq!(false, a.subscriber);
    }
}
