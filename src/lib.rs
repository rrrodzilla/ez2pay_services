//our account consists of a phone number, it's always tied to
//the phone used to contact the service
pub struct Account {
    phone_number: String,
    stripe_id: String,
    subscriber: bool,
}
impl Account {
    fn new(id: String) -> Self {
        Account {
            phone_number: id,
            stripe_id: String::from(""),
            subscriber: false,
        }
    }
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
        let a = Account::new(p.clone());
        assert_eq!(p, a.phone_number);
        assert_eq!(false, a.subscriber);
    }
}
