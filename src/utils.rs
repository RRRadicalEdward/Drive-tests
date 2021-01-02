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
    let (key, cert) = match env::var_os("CERT_DIR") {
        // Try to get path cert files for environment variables
        Some(path_to_cert) => {
            let path = Path::new(&path_to_cert);
            (
                path.join("key.key")
                    .to_str()
                    .with_context(|| "failed to convert path to str for cert key file")?
                    .to_owned(),
                path.join("cert.crt")
                    .to_str()
                    .with_context(|| "failed to convert path to str for cert file")?
                    .to_owned(),
            )
        }
        None => {
            // If `CERT_DIR` environment variable isn't set, it will try get cert files from current project directory in `/cert` folder
            let current_dir = env::current_dir().with_context(|| "failed to get current_dir path for cert files")?;
            (
                current_dir
                    .join("cert/key.pem")
                    .to_str()
                    .with_context(|| "failed to convert path to str for cert key")?
                    .to_owned(),
                current_dir
                    .join("cert/cert.pem")
                    .to_str()
                    .with_context(|| "failed to convert path to str for cert file")?
                    .to_owned(),
            )
        }
    };

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file(key.as_str(), SslFiletype::PEM)?;
    builder.set_certificate_chain_file(cert.as_str())?;
    Ok(builder)
}
