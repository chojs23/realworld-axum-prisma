#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub exp_in_sec: i64,
}
