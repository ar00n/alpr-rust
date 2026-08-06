use reqwest::{Client, Method, Url};
use std::net::IpAddr;
use serde_json::Value;

use crate::error::AppError;

// 1. SSRF IP Validator
pub fn is_ip_allowed(ip: IpAddr) -> bool {
    match ip {
        // Block Localhost / Loopback
        IpAddr::V4(ipv4) if ipv4.is_loopback() => false,
        IpAddr::V6(ipv6) if ipv6.is_loopback() => false,
        // Block Cloud Metadata (AWS, GCP, etc.)
        IpAddr::V4(ipv4) if ipv4.is_link_local() => false,
        // Block Docker default bridge subnet (172.16.0.0 - 172.31.255.255)
        IpAddr::V4(ipv4) if ipv4.octets()[0] == 172 && (16..=31).contains(&ipv4.octets()[1]) => false,
        // Allow everything else (including 192.168.x.x LANs)
        _ => true,
    }
}

// 2. The Execution Function
pub async fn execute_action(
    action_url: &str, 
    method_str: &str, 
    template: Option<&str>,
    headers_val: Option<&Value>,
    auth_type: &str,
    auth_data: Option<&Value>,
    licence_plate: &str
) -> Result<(u16, String), AppError> {
    let url = Url::parse(action_url)
        .map_err(|_| AppError::bad_request("Failed to parse url"))?;

    // SSRF Check
    if let Some(host_str) = url.host_str() {
        if let Ok(ip) = host_str.parse::<IpAddr>() {
            if !is_ip_allowed(ip) {
                return Err(AppError::forbidden("Access to restricted network (SSRF protection)"));
            }
        }
    }

    let method = Method::from_bytes(method_str.to_uppercase().as_bytes())
        .map_err(|_| AppError::bad_request("Invalid method type."))?;
    
    let client = Client::new();
    let mut request = client.request(method, url);

    // Apply Headers
    if let Some(headers_obj) = headers_val.and_then(|v| v.as_object()) {
        for (key, value) in headers_obj {
            if let Some(val_str) = value.as_str() {
                request = request.header(key, val_str);
            }
        }
    }

    // Apply Authentication
    if let Some(a_data) = auth_data {
        match auth_type.to_lowercase().as_str() {
            "bearer" => {
                if let Some(token) = a_data.get("token").and_then(|v| v.as_str()) {
                    request = request.bearer_auth(token);
                }
            }
            "basic" => {
                let username = a_data.get("username").and_then(|v| v.as_str()).unwrap_or("");
                let password = a_data.get("password").and_then(|v| v.as_str());
                request = request.basic_auth(username, password);
            }
            "api_key" => {
                let key_value = a_data.get("key").and_then(|v| v.as_str()).unwrap_or("");
                let key_name = a_data.get("header_name")
                    .or_else(|| a_data.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("X-API-Key");
                let placement = a_data.get("placement").and_then(|v| v.as_str()).unwrap_or("header");

                match placement.to_lowercase().as_str() {
                    "query" => {
                        request = request.query(&[(key_name, key_value)]);
                    }
                    _ => {
                        // Default to header
                        request = request.header(key_name, key_value);
                    }
                }
            }
            _ => {}
        }
    }

    // Simple Templating
    if let Some(body_tmpl) = template {
        let escaped_plate = serde_json::to_string(licence_plate)
            .map_err(|_| AppError::bad_request("Failed to serialize licence plate"))?;
        let clean_plate = &escaped_plate[1..escaped_plate.len()-1]; 
        
        let final_body = body_tmpl.replace("${LICENCE_PLATE}", clean_plate);
        
        if serde_json::from_str::<Value>(&final_body).is_ok() {
            request = request.header("Content-Type", "application/json");
        }
        request = request.body(final_body);
    }

    // Execute
    let response = request.send().await
        .map_err(|e| AppError::bad_request(format!("Action failed to execute: {}", e)))?;
    
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_else(|_| "".to_string());

    Ok((status, body))
}