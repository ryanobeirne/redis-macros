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

    con.json_set::<'_, &str, &str, User, ()>("optuser_key", "$", &user).await?;

    // Wrap the data in `JsonOpt(..)` when reading from from Redis
    let JsonOpt(stored_user): JsonOpt<User> = con.json_get("optuser_key", "$").await?;
    assert_eq!(Some(user), stored_user);

    // Return ok even when result is Nil
    let JsonOpt(noexist): JsonOpt<User> = con.json_get("some_key_that_should_not_exist", "$").await?;
    assert_eq!(None, noexist);

    Ok(())
}

#[test]
fn test_jsonopt_wrapper_basic() {
    assert_eq!(main(), Ok(()));
}
