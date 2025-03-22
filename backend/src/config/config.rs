use std::env;

#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub solana_url: String,
    pub port: u16,
    pub token_expiration: i64,
    pub server_host: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            solana_url: env::var("SOLANA_URL").expect("SOLANA_URL must be set"),
            port: env::var("PORT")
                .expect("PORT must be set")
                .parse()
                .expect("PORT must be a number"),
            token_expiration: env::var("TOKEN_EXPIRATION")
                .expect("TOKEN_EXPIRATION must be set")
                .parse()
                .expect("TOKEN_EXPIRATION must be a number"),
            server_host: env::var("SERVER_HOST").expect("SERVER_HOST must be set"),
        }
    }
}