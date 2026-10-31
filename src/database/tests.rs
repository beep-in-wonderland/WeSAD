use super::*;

#[test]
pub fn read_webhook_id() {

    let session = Session::_init_from_path(
        "/home/beep/Dev/wesad/dev/persist/wesad.db").expect("DB path not found");
    let test_webhook_name: &String = &format!("wesad-status");

    let result: usize = read::webhook_by_name(&session, test_webhook_name).unwrap_or(0);
    println!("{}", result);
    assert_eq!(result, 1)
}

