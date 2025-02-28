use argon2::{Algorithm, Argon2, Params, Version};

pub fn get_argon2_ctx<'a>() -> Argon2<'a> {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default())
}
