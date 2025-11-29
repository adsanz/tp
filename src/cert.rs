use anyhow::Result;
use rcgen::generate_simple_self_signed;

pub struct Certs {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
}

pub fn generate_self_signed_certs(subject_alt_names: Vec<String>) -> Result<Certs> {
    let cert = generate_simple_self_signed(subject_alt_names)?;
    Ok(Certs {
        cert: cert.cert.der().to_vec(),
        key: cert.key_pair.serialize_der(),
    })
}
