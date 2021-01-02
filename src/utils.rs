use crate::web::{check_answer, check_answer_with_user, get_test, healthy, sing_in, sing_up};
use anyhow::Context;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use std::{env, path::Path};

pub fn services_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(sing_up)
        .service(sing_in)
        .service(get_test)
        .service(check_answer_with_user)
        .service(check_answer)
        .service(healthy);
}

pub fn tls_builder() -> anyhow::Result<SslAcceptorBuilder> {
    let (key, cert) = get_certs_paths()?;

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file(key.as_str(), SslFiletype::PEM)?;
    builder.set_certificate_chain_file(cert.as_str())?;
    Ok(builder)
}

fn get_certs_paths() -> anyhow::Result<(String, String)> {
    match env::var_os("CERT_DIR") {
        // Try to get path to cert files from environment variable
        Some(path_to_cert_dir) => {
            let path = Path::new(&path_to_cert_dir);
            Ok((
                path.join("key.key")
                    .to_str()
                    .with_context(|| "failed to convert path to str of cert's key file")?
                    .to_owned(),
                path.join("cert.crt")
                    .to_str()
                    .with_context(|| "failed to convert path to str of cert file")?
                    .to_owned(),
            ))
        }
        None => {
            // If `CERT_DIR` environment variable isn't set, it will try get cert files from current project directory in `/cert` folder
            let current_dir = env::current_dir().with_context(|| "failed to get current_dir path of cert files")?;
            Ok((
                current_dir
                    .join("cert/key.key")
                    .to_str()
                    .with_context(|| "failed to convert path to str of cert's key")?
                    .to_owned(),
                current_dir
                    .join("cert/cert.crt")
                    .to_str()
                    .with_context(|| "failed to convert path to str of cert file")?
                    .to_owned(),
            ))
        }
    }
}

// keys for encrypt and decrypt users passwords
pub fn get_keys_paths() -> anyhow::Result<(String, String)> {
    let keys = match env::var_os("KEYS_DIR") {
        // Try to get paths of keys files from environment variable
        Some(path_to_keys_dir) => {
            let path = Path::new(&path_to_keys_dir);
            (
                path.join("public-key.pem")
                    .to_str()
                    .with_context(|| "failed to convert path to str of public-key file")?
                    .to_owned(),
                path.join("private-key.pem")
                    .to_str()
                    .with_context(|| "failed to convert path to str of private-key file")?
                    .to_owned(),
            )
        }
        None => {
            // If `KEYS_DIR` environment variable isn't set, it will try get keys files from current project directory in `/rsa-keys` folder
            let current_dir = env::current_dir().with_context(|| "failed to get current_dir path of keys files")?;
            (
                current_dir
                    .join("rsa-keys/public-key.pem")
                    .to_str()
                    .with_context(|| "failed to convert path to str of public-key file")?
                    .to_owned(),
                current_dir
                    .join("rsa-keys/private-key.pem")
                    .to_str()
                    .with_context(|| "failed to convert path to str of private-key file")?
                    .to_owned(),
            )
        }
    };
    Ok(keys)
}
