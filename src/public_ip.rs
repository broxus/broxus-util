use std::net::Ipv4Addr;

pub async fn resolve_public_ip(ip: Option<Ipv4Addr>) -> Result<Ipv4Addr, IpResolutionError> {
    // TODO: Add tunnel punching

    match ip {
        Some(address) => Ok(address),
        None => public_ip::addr_v4()
            .await
            .ok_or(IpResolutionError::PublicIpNotFound),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IpResolutionError {
    #[error("public ip not found")]
    PublicIpNotFound,
}
