use redis::{FromRedisValue, Value};
use redis_macros::JsonOpt;
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

#[test]
pub fn it_should_deserialize_json_results() {
    let user = User::default();
    let val = default_user_bulk_string();

    match JsonOpt::<User>::from_redis_value(&val) {
        Ok(JsonOpt(Some(parsed_users))) => assert_eq!(parsed_users, user),
        Ok(JsonOpt(None)) => panic!("result is None"),
        Err(e) => panic!("{e}"),
    }
}

#[test]
pub fn it_should_fail_if_the_result_is_not_redis_json() {
    // RedisJSON responses should have wrapping brackets (i.e. [{...}])
    let string = r#"{"id":1,"name":"Ziggy","addresses":[{"Street":"Downing"},{"Road":"Abbey"}]}"#;
    let val = Value::BulkString(string.as_bytes().into());

    match JsonOpt::<User>::from_redis_value(&val) {
        Ok(JsonOpt(Some(val))) => panic!("RedisJSON unwrapping should fail: {val:?}"),
        Ok(JsonOpt(None)) => panic!("RedisJSON unwrapping should fail, but returned `None`"),
        Err(err) => assert_eq!(
            err.to_string(),
            format!("Response was of incompatible type - TypeError: Response type was not JSON type. (response was bulk-string('{string:?}'))")),
    }
}

#[test]
pub fn it_should_fail_if_input_is_not_compatible_with_type() {
    let string = "[{}]";
    let val = Value::BulkString(string.as_bytes().into());

    match JsonOpt::<User>::from_redis_value(&val) {
        Ok(JsonOpt(Some(val))) => panic!("Deserialization should fail: {val:?}"),
        Ok(JsonOpt(None)) => panic!("Deserialization should fail, but returned `None`"),
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

    match JsonOpt::<User>::from_redis_value(&val) {
        Ok(JsonOpt(Some(val))) => panic!("UTF-8 parsing should fail: {val:?}"),
        Ok(JsonOpt(None)) => panic!("UTF-8 parsing should fail, but returned `None`"),
        Err(err) => assert_eq!(
            err.to_string(),
            format!("Response was of incompatible type - TypeError: Response was not valid UTF-8 string. (response was binary-data({bytes:?}))")
        ),
    }
}

#[test]
pub fn it_should_not_fail_if_input_is_missing() {
    match JsonOpt::<User>::from_redis_value(&Value::Nil) {
        Ok(JsonOpt(Some(user))) => panic!("result is `{user:?}`"),
        Ok(JsonOpt(None)) => (),
        Err(e) => panic!("{e}"),
    }
}
