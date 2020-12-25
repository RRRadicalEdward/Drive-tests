use crate::web::{check_answer, check_answer_with_user, get_test, healthy, sing_in, sing_up};

use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

pub fn services_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(sing_up)
        .service(sing_in)
        .service(get_test)
        .service(check_answer_with_user)
        .service(check_answer)
        .service(healthy);
}

pub fn tls_builder() -> anyhow::Result<SslAcceptorBuilder> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file("cert/key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("cert/cert.pem")?;
    Ok(builder)
}
