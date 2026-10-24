use super::*;
use crate::{HashMap, daemon::updater::do_placeholder_replacements};

#[test]
pub fn placeholder_replacement() {

    let map = build_test_map();
    let mut content = "the $$dog$$ goes meow\nthe $$cat$$ goes moo".to_string();

    do_placeholder_replacements(map, &mut content).unwrap();

    assert_eq!(
        content,
        "the cat goes meow\nthe cow goes moo".to_string()
    )
}


fn build_test_map() -> HashMap<String,String> {
    let mut map = HashMap::new();

    map.insert("dog".to_string(), "cat".to_string());
    map.insert("cat".to_string(), "cow".to_string());

    map
}