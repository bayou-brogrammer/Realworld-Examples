pub fn hash_password(password: impl AsRef<[u8]>) -> crate::error::AppResult<String> {
    let salt = password_hash::SaltString::generate(&mut rand::thread_rng());

    let hash =
        password_hash::PasswordHash::generate(argon2::Argon2::default(), password.as_ref(), &salt)
            .map_err(|err| anyhow::anyhow!(err))?
            .to_string();
    Ok(hash)
}
