use redis::{FromRedisValue, Value};
use redis_macros::JsonVec;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
enum Address {
    Street(String),
    Road(String),
}

#[derive(Debug, PartialEq, Deserialize)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

impl User {
    fn three_default_users() -> Vec<Self> {
        vec![User::default(), User::default(), User::default()]
    }
}

impl Default for User {
    fn default() -> Self {
        User {
            id: 1,
            name: "Ziggy".to_string(),
            addresses: vec![
                Address::Street("Downing".to_string()),
                Address::Road("Abbey".to_string()),
            ],
        }
    }
}

fn default_user_bulk_string() -> Value {
    Value::BulkString(
            b"[{\"id\":1,\"name\":\"Ziggy\",\"addresses\":[{\"Street\":\"Downing\"},{\"Road\":\"Abbey\"}]}]".to_vec()
        )
}

fn default_user_array() -> Value {
    let bulk = default_user_bulk_string();
    three_value_array(bulk)
}

fn three_value_array(value: Value) -> Value {
    Value::Array(vec![
        value.clone(), value.clone(), value,
    ])
}

#[test]
pub fn it_should_deserialize_json_results() {
    let users = User::three_default_users();
    let val = default_user_array();

    match JsonVec::<User>::from_redis_value(&val) {
        Ok(JsonVec(parsed_users)) => assert_eq!(parsed_users, users),
        Err(e) => panic!("{e}"),
    }
}

#[test]
pub fn it_should_also_deserialize_json_wrappable_arguments() {
    let addresses = vec![
        Address::Street("Downing".to_string()),
        Address::Road("Abbey".to_string()),
    ];

    let val = Value::Array(vec![
        Value::BulkString(b"[{\"Street\":\"Downing\"}]".to_vec()),
        Value::BulkString(b"[{\"Road\":\"Abbey\"}]".to_vec()),
    ]);

    match JsonVec::<Address>::from_redis_value(&val) {
        Ok(JsonVec(parsed_addresses)) => assert_eq!(parsed_addresses, addresses),
        Err(e) => panic!("{e}"),
    }
}

#[test]
pub fn it_should_fail_if_the_result_is_not_redis_json() {
    // RedisJSON responses should have wrapping brackets (i.e. [{...}])
    let string = r#"{"id":1,"name":"Ziggy","addresses":[{"Street":"Downing"},{"Road":"Abbey"}]}"#;
    let val = Value::BulkString(string.as_bytes().into());
    let values = three_value_array(val);

    match JsonVec::<User>::from_redis_value(&values) {
        Ok(val) => panic!("RedisJSON unwrapping should fail: {val:?}"),
        Err(err) => assert_eq!(
            err.to_string(),
            format!("Response was of incompatible type - TypeError: Response type was not JSON type. (response was bulk-string('{string:?}'))")),
    }
}

#[test]
pub fn it_should_fail_if_input_is_not_compatible_with_type() {
    let string = "[{}]";
    let val = Value::BulkString(string.as_bytes().into());
    let values = three_value_array(val);

    match JsonVec::<User>::from_redis_value(&values) {
        Ok(val) => panic!("Deserialization should fail: {val:?}"),
        Err(err) => assert_eq!(
            err.to_string(),
            format!("Response was of incompatible type - TypeError: Response type in JSON was not deserializable. (response was bulk-string('{string:?}'))"),
        ),
    }
}

#[test]
pub fn it_should_fail_if_input_is_not_valid_utf8() {
    let bytes = [0, 159, 146, 150];
    let val = Value::BulkString(bytes.to_vec()); // Some invalid utf8
    let values = three_value_array(val);

    match JsonVec::<User>::from_redis_value(&values) {
        Ok(val) => panic!("UTF-8 parsing should fail: {val:?}"),
        Err(err) => assert_eq!(
            err.to_string(),
            format!("Response was of incompatible type - TypeError: Response was not valid UTF-8 string. (response was binary-data({bytes:?}))")
        ),
    }
}

#[test]
pub fn it_should_fail_if_input_is_missing() {
    match JsonVec::<User>::from_redis_value(&Value::Nil) {
        Ok(val) => panic!("Value Nil should fail: {val:?}"),
        Err(err) => assert_eq!(
            err.to_string(),
            "Response was of incompatible type - TypeError: Response type not a RedisJSON deserializable Array. (response was nil)"
        ),
    }
}
