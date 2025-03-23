use std::env;

#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub solana_url: String,
    pub port: u16,
    pub token_expiration: i64,
    pub server_host: String,
    pub admin_pubkey: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "your-very-long-secret-key-here".to_string()),
            solana_url: env::var("SOLANA_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            token_expiration: env::var("TOKEN_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            admin_pubkey: env::var("ADMIN_PUBKEY").expect("ADMIN_PUBKEY must be set"),
        }
    }
}