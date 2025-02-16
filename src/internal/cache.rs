use redis::{AsyncCommands, Client};
use rocket::{
    http::{ContentType, Status},
    response::Responder,
    Request, Response,
};
use serde::de::DeserializeOwned;
use std::error::Error as StdError;
use std::{fmt, io::Cursor};

#[derive(Debug)]
pub struct CacheService {
    pub client: Client,
}

impl CacheService {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }
    pub async fn set_value(
        &self,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        con.set(key, value).await?;
        Ok(())
    }

    pub async fn get_value<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<T, Box<dyn StdError + Send + Sync>> {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let value: String = con.get(key).await?;
        let deserialized: T = serde_json::from_str(&value)?;
        Ok(deserialized)
    }
}

#[derive(Debug)]
pub enum CacheError {
    RedisError(redis::RedisError),
    SerializationError(serde_json::Error),
    BoxedError(Box<dyn StdError + Send + Sync>),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::RedisError(e) => write!(f, "Redis error: {}", e),
            CacheError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            CacheError::BoxedError(e) => write!(f, "Boxed error: {}", e),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        CacheError::RedisError(err)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::SerializationError(err)
    }
}
impl From<Box<dyn StdError + Send + Sync>> for CacheError {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        CacheError::BoxedError(err)
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for CacheError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'o> {
        let (status, error_message) = match self {
            CacheError::RedisError(e) => {
                (Status::InternalServerError, format!("Redis error: {}", e))
            }
            CacheError::SerializationError(e) => {
                (Status::BadRequest, format!("Serialization error: {}", e))
            }
            CacheError::BoxedError(e) => {
                (Status::InternalServerError, format!("Boxed error: {}", e))
            }
        };

        Response::build()
            .status(status)
            .header(ContentType::Plain)
            .sized_body(error_message.len(), Cursor::new(error_message))
            .ok()
    }
}
