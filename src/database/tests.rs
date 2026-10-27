use super::*;

#[test]
pub fn read_webhook_id() {

    let session = Session::_init_from_path(
        "/home/beep/Dev/bbnd/dev/persist/bbnd.db").expect("DB path not found");
    let test_webhook_name: &String = &format!("bbnd-status");

    let result: usize = read::webhook_by_name(&session, test_webhook_name).unwrap_or(0);
    println!("{}", result);
    assert_eq!(result, 1)
}

