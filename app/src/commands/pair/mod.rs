use std::{
    fs::File,
    io::{BufRead, BufReader},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::Path,
};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use local_ip_address::local_ip as get_local_ip;
use rand::{rngs::StdRng, seq::SliceRandom, thread_rng, Rng, SeedableRng};
use reqwest::blocking::get;

#[allow(clippy::module_inception)]
pub mod pair;
use crate::utils::get_data_path;
pub use pair::*;

const NUM_ADJ_VERBS: u64 = 3248;

fn get_public_ip() -> Result<IpAddr> {
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

fn read_lines(filename: &str) -> Result<Vec<String>> {
    let path = get_data_path(&Path::new("words").join(format!("{}.txt", filename)))?;
    let reader = BufReader::new(File::open(path)?).lines();
    Ok(reader.into_iter().map_while(Result::ok).collect())
}

fn ipv4_to_words(ip: Ipv4Addr, words: &[String]) -> (u32, String) {
    let mut value = ip.octets().iter().enumerate().fold(0u32, |acc, (i, &octet)| {
        acc | (octet as u32) << (24 - i * 8)
    });

    let ret: &mut Vec<&str> = &mut vec![];
    while value > 0 {
        ret.push(&words[value as usize % 335]);
        value /= 335;
    }
    (ret.len() as u32, ret.iter().rev().join(""))
}

fn ipv6_to_words(ip: Ipv6Addr, words: &[String]) -> (u32, String) {
    let mut value = ip.octets().iter().enumerate().fold(0u128, |acc, (i, &octet)| {
        acc | (octet as u128) << (24 - i * 8)
    });

    let ret: &mut Vec<&str> = &mut vec![];
    while value > 0 {
        ret.push(&words[value as usize % 335]);
        value /= 335;
    }
    (ret.len() as u32, ret.iter().rev().join(""))
}

fn create_invite_code() -> Result<String> {
    let seed = thread_rng().gen_range(0..NUM_ADJ_VERBS);
    let mut rng = StdRng::seed_from_u64(seed);
    let adj_verbs = &mut read_lines("adj_verbs")?;
    let (Ok(local_ip), Ok(public_ip)) = (get_local_ip(), get_public_ip()) else {
        return Err(anyhow!("Unable to retrieve public IP Address"));
    };
    let seed_word: String;

    {
        let Some(word) = adj_verbs.choose(&mut rng) else {
            return Err(anyhow!(""));
        };
        seed_word = word.to_string();
    }

    adj_verbs.shuffle(&mut rng);

    let (local_words_count, local_words) = match local_ip {
        IpAddr::V4(ip) => ipv4_to_words(ip, adj_verbs),
        IpAddr::V6(ip) => ipv6_to_words(ip, adj_verbs),
    };

    let (public_words_count, public_words) = match public_ip {
        IpAddr::V4(ip) => ipv4_to_words(ip, adj_verbs),
        IpAddr::V6(ip) => ipv6_to_words(ip, adj_verbs),
    };

    Ok(format!(
        "{seed_word} {local_words} {public_words} {} {}",
        adj_verbs[local_words_count as usize],
        read_lines("nouns")?[public_words_count as usize],
    ))
}

#[derive(Debug)]
struct Code {
    local_ip: IpAddr,
    public_ip: IpAddr,
}

fn get_index(needle: &str, haystack: &[String]) -> Option<usize> {
    for (i, straw) in haystack.iter().enumerate() {
        if straw.to_owned().eq(needle) {
            return Some(i);
        }
    }
    None
}

fn words_to_ipv4(octet_words: &[&str], word_listing: &[String]) -> Ipv4Addr {
    let bits = (0..4).rev().fold(
        octet_words
            .iter()
            .filter_map(|word| get_index(word, word_listing))
            .fold(0u32, |acc, idx| acc * 335 + idx as u32),
        |acc, i| ((acc >> (i * 8)) & 0xFF),
    );

    Ipv4Addr::from_bits(bits)
}

fn words_to_ipv6(octet_words: &[&str], word_listing: &[String]) -> Ipv6Addr {
    let bits = (0..8).rev().fold(
        octet_words
            .iter()
            .filter_map(|word| get_index(word, word_listing))
            .fold(0u128, |acc, idx| acc * 335 + idx as u128),
        |acc, i| ((acc >> (i * 8)) & 0xFFFF),
    );

    Ipv6Addr::from_bits(bits)
}

fn words_to_ip(octet_words: &[&str], word_listing: &[String]) -> Result<IpAddr> {
    match octet_words.len() {
        _len @ 3..4 => Ok(IpAddr::from(words_to_ipv4(octet_words, word_listing))),
        _len @ 11..16 => Ok(IpAddr::from(words_to_ipv4(octet_words, word_listing))),
        _ => Err(anyhow!("Unable to decypher code")),
    }
}

fn decrypt_code(code: &str) -> Result<Code> {
    let words = code.split(" ").collect::<Vec<&str>>();
    if 8 > words.len() {
        return Err(anyhow!("Incorrect Code"));
    }

    let adj_verbs = &mut read_lines("adj_verbs")?;
    let seed_word = words[0];
    let mut seed: Option<u64> = None;

    let seed_adj_verbs = adj_verbs.to_owned();
    for (i, adverb) in seed_adj_verbs.iter().enumerate() {
        if seed_word == adverb {
            seed = Some(i as u64);
            break;
        }
    }

    if seed.is_none() {
        return Err(anyhow!("Incorrect Code"));
    }

    adj_verbs.shuffle(&mut StdRng::seed_from_u64(seed.unwrap()));

    let local_len = get_index(words.iter().rev().nth(1).unwrap(), adj_verbs).unwrap();
    let local_words = &words[1..local_len + 1];
    let public_words = &words[local_len + 1..words.len() - 2];

    Ok(Code {
        local_ip: words_to_ip(local_words, adj_verbs)?,
        public_ip: words_to_ip(public_words, adj_verbs)?,
    })
}
