use base64::Engine;
use base64::engine::general_purpose;
use hyper_tls::native_tls;
use thiserror::Error;
use prost::Message;
use crate::youtube::channel_continuation::Token;
use crate::youtube::ChannelContinuation;
use chrono::NaiveDate;
use std::net::{IpAddr, Ipv6Addr};
use rand::Rng;
use chrono::{DateTime, Duration, TimeZone, Utc};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref TIME_REGEX: Regex = Regex::new(
        r"^(\d+)\s+(second|minute|hour|day|week|month|year)s?\s+ago$"
    ).unwrap();
}


#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
    #[error("TLS error: {0}")]
    TlsError(#[from] native_tls::Error),
}

pub fn get_rand_ipv6(subnet: &str, range_id: u16) -> Result<IpAddr, Box<dyn std::error::Error>> {
    // Split the subnet string into address and prefix length
    let parts: Vec<&str> = subnet.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid subnet format".into());
    }

    // Parse the IPv6 address
    let ipv6: u128 = parts[0].parse::<Ipv6Addr>()?.into();

    // Parse the prefix length
    let prefix_len: u8 = parts[1].parse()?;
    if prefix_len != 48 {
        return Err("Only /48 subnets are supported".into());
    }

    // Clear the lower 80 bits (128 - 48) of the network address
    let net_part = (ipv6 >> 80) << 80;
    
    // Shift the range_id into position (place it in bits 64-79)
    let range_part = (range_id as u128) << 64;
    
    // Generate random number for host portion (lower 64 bits)
    let rand: u128 = rand::thread_rng().gen();
    let host_part = rand & ((1u128 << 64) - 1);  // Only keep lower 64 bits
    
    // Combine all parts
    let result = net_part | range_part | host_part;

    Ok(IpAddr::V6(result.into()))
}

pub fn parse_numeric_string(numeric_str: &str) -> i64 {
    let numeric_str = numeric_str.trim();
    let parts: Vec<&str> = numeric_str.split_whitespace().collect();

    if parts.len() >= 1 {
        let value_str = parts[0].replace(",", "");
        value_str.parse().unwrap_or(0)
    } else {
        0
    }
}

pub fn parse_multiplied_string(multiplied_str: &str) -> i64 {
    let multiplied_str = multiplied_str.trim();
    let multiplied_str = multiplied_str.split_whitespace().next().unwrap_or("0");

    let multiplier: i64;
    let value: f64;

    if multiplied_str.ends_with('K') {
        multiplier = 1000;
        value = multiplied_str[..multiplied_str.len() - 1].parse().unwrap_or(0.0);
    } else if multiplied_str.ends_with('M') {
        multiplier = 1_000_000;
        value = multiplied_str[..multiplied_str.len() - 1].parse().unwrap_or(0.0);
    } else if multiplied_str.ends_with('B') {
        multiplier = 1_000_000_000;
        value = multiplied_str[..multiplied_str.len() - 1].parse().unwrap_or(0.0);
    } else {
        return multiplied_str.parse().unwrap_or(0);
    }

    (value * multiplier as f64) as i64
}

pub fn parse_creation_date(joined_date_str: &str) -> i32 {
    let parts: Vec<&str> = joined_date_str.split_whitespace().collect();
    if parts.len() == 0 {
        return 0
    }

    if parts.len() == 3 {
        let month = match parts[0] {
            "Jan" => 1, "Feb" => 2, "Mar" => 3, "Apr" => 4, "May" => 5, "Jun" => 6,
            "Jul" => 7, "Aug" => 8, "Sep" => 9, "Oct" => 10, "Nov" => 11, "Dec" => 12,
            _ => 0,
        };

        let day: u32 = parts[1].trim_end_matches(',').parse().unwrap_or_default();
        let year: i32 = parts[2].parse().unwrap_or_default();
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_opt(0, 0, 0))
            .map(|date_time| date_time.and_utc().timestamp() as i32)
            .unwrap_or_default()
    } else {
        0
    }
}

pub fn generate_continuation_token(channel_id: String, request_token: String) -> String {
    let token = Token {
        channel_id,
        request_token,
    };

    let continuation = ChannelContinuation {
        token: Some(token),
    };

    // Encode the protobuf message directly to bytes
    let proto_bytes = continuation.encode_to_vec();
    
    // Encode to base64
    general_purpose::STANDARD.encode(proto_bytes)
}

pub fn relative_time_to_timestamp(input: &str) -> Result<i64, String> {
    let now = Utc::now();
    let input = input.trim().to_lowercase();
    
    if let Some(captures) = TIME_REGEX.captures(&input) {
        let amount: i64 = captures[1]
            .parse()
            .map_err(|_| "Failed to parse number")?;
            
        let unit = &captures[2];
        
        let result = match unit {
            "second" => now - Duration::seconds(amount),
            "minute" => now - Duration::minutes(amount),
            "hour" => now - Duration::hours(amount),
            "day" => now - Duration::days(amount),
            "week" => now - Duration::weeks(amount),
            "month" => now - Duration::days(amount * 30), // approximation
            "year" => now - Duration::days(amount * 365), // approximation
            _ => return Err("Invalid time unit".to_string())
        };
        
        Ok(result.timestamp())
    } else {
        Err("Invalid input format".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_relative_time_parsing() {
        let now = Utc::now();
        
        // Test cases
        let test_cases = vec![
            ("33 seconds ago", Duration::seconds(33)),
            ("4 minutes ago", Duration::minutes(4)),
            ("15 minutes ago", Duration::minutes(15)),
            ("10 hours ago", Duration::hours(10)),
            ("2 days ago", Duration::days(2)),
            ("3 weeks ago", Duration::weeks(3)),
            ("1 month ago", Duration::days(30)),
            ("9 years ago", Duration::days(9 * 365)),
        ];
        
        for (input, expected_duration) in test_cases {
            let result = relative_time_to_timestamp(input).unwrap();
            let expected = (now - expected_duration).timestamp();
            // Allow for 1 second difference due to test execution time
            assert!((result - expected).abs() <= 1, 
                "Failed for input: {}", input);
        }
    }
}