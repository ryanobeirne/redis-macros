use redis::{Client, ErrorKind, JsonAsyncCommands, RedisError, RedisResult};
use redis_macros::*;
use serde::{Deserialize, Serialize};

/// Define structs to hold the data
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
enum Address {
    Street(String),
    Road(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

/// Instead of deriving the data, use Json wrappers
/// This will make it compatible with any kind of data (for example Vec)
#[tokio::main]
async fn main() -> RedisResult<()> {
    // Open new connection to localhost
    let client = Client::open("redis://localhost:6379")?;
    let mut con = client.get_multiplexed_async_connection().await.map_err(|_| {
        RedisError::from((
            ErrorKind::InvalidClientConfig,
            "Cannot connect to localhost:6379. Try starting a redis-server process or container.",
        ))
    })?;

    // Define the data you want to store in Redis.
    let user = User {
        id: 1,
        name: "Ziggy".to_string(),
        addresses: vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
        ],
    };

    let keys = vec!["user_key_0", "user_key_1", "user_key_2"];
    let expected = vec![user.clone(), user.clone(), user.clone()];

    for (i, key) in keys.iter().enumerate() {
        con.json_set::<'_, &str, &str, User, ()>(key, "$", &expected[i]).await?;
    }

    // Wrap the data in `JsonVec(..)` when reading from from Redis
    let JsonVec(stored_user): JsonVec<User> = con.json_get(&keys, "$").await?;
    assert_eq!(expected, stored_user);

    // Passing an empty array as keys should return an error
    assert!(con.json_get::<&[&str], &str, JsonVec<User>>(&[], "$").await.is_err());

    // Failing to find a key should return an error, not an empty array
    let keys = vec!["some_key_that_should_not_exist", "some_key_that_should_not_exist"];
    match con.json_get::<Vec<&str>, &str, JsonVec<User>>(keys, "$").await {
        Ok(JsonVec(vec)) => panic!("this should return an error, but found this: {vec:?}"),
        Err(_) => (),
    }

    // Failing to find a key should return an error, not an empty array.
    let keys = vec!["some_key_that_should_not_exist"];
    match con.json_get::<Vec<&str>, &str, JsonVec<User>>(keys, "$").await {
        Ok(JsonVec(vec)) => panic!("this should return an error, but found this: {vec:?}"),
        Err(_) => (),
    }

    // Finding some keys and not others should return an error
    let keys = vec!["user_key_0", "some_key_that_should_not_exist"];
    match con.json_get::<Vec<&str>, &str, JsonVec<User>>(keys, "$").await {
        Ok(JsonVec(vec)) => panic!("this should return an error, but found this: {vec:?}"),
        Err(_) => (),
    }

    // Finding some keys and not others should return an error
    let keys = vec!["user_key_0", "user_key_1", "some_key_that_should_not_exist"];
    match con.json_get::<Vec<&str>, &str, JsonVec<User>>(keys, "$").await {
        Ok(JsonVec(vec)) => panic!("this should return an error, but found this: {vec:?}"),
        Err(_) => (),
    }

    Ok(())
}

#[test]
fn test_jsonvec_wrapper_basic() {
    assert_eq!(main(), Ok(()));
}
