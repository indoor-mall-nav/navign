use lazy_static::lazy_static;

lazy_static! {
    /// Get the Secret Key, which is stored in `./secret_key.txt` or in the environment variable `CERTIFICATION_SECRET_KEY`.
    /// If unset, generate a `default_secret_key`.
    static ref SECRET_KEY: String = {
        uuid::Uuid::new_v4().to_string()
    };
}
