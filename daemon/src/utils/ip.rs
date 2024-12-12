use std::net::IpAddr;

use anyhow::{anyhow, Result};
pub use local_ip_address::local_ip as get_local_ip;
use minreq::get;

pub fn get_public_ip() -> Result<IpAddr> {
    let urls = [
        "https://ifconfig.me",
        "https://api.ipify.org",
        "https://ipinfo.io/ip",
        "https://iprs.fly.dev",
    ];

    for url in urls.into_iter() {
        if let Ok(response) = get(url).send() {
            if let Ok(body) = response.as_str() {
                if let Ok(ip) = body.parse::<IpAddr>() {
                    return Ok(ip);
                }
            }
        }
    }
    Err(anyhow!("Unable to obtain public IP Address."))
}
