use std::net::IpAddr;

use anyhow::{anyhow, Result};
pub use local_ip_address::local_ip as get_local_ip;
use reqwest::blocking::get;

pub fn get_public_ip() -> Result<IpAddr> {
    let urls = [
        "https://api.ipify.org",
        "https://ifconfig.me",
        "https://ipinfo.io/ip",
        "https://iprs.fly.dev",
    ];

    for url in urls.into_iter() {
        if let Ok(response) = get(url).and_then(|r| r.text()) {
            if let Ok(ip) = response.parse::<IpAddr>() {
                return Ok(ip);
            }
        }
    }
    Err(anyhow!("Unable to obtain public IP Address."))
}
