use html_escape::{decode_html_entities, encode_safe};
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashMap;
use urlencoding::encode;

pub fn make_payloads_post(params: &str, payload: &str) -> Vec<String> {
    let mut weaponized_endpoints = Vec::new();

    if !params.is_empty() {
        let parameters = params.split('&').collect::<Vec<_>>();

        for param in parameters.iter() {
            if let Some((name, value)) = param.split_once('=') {
                // Append payload to each parameter's value
                let modified_value = format!("{}{}", value, encode(payload));

                // Reconstruct URL with modified parameter
                let weaponized_url = parameters
                    .iter()
                    .map(|p| {
                        if p.starts_with(name) {
                            format!("{}={}", name, modified_value)
                        } else {
                            p.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("&");

                weaponized_endpoints.push(weaponized_url);
            }
        }
    } else {
        // Handle case where URL does not have parameters
        weaponized_endpoints.push(payload.to_string());
    }

    weaponized_endpoints
}

pub fn json_sikkish(init_val: Value, payload: &str) -> Vec<Value> {
    match init_val {
        Value::Object(map) => {
            let mut results = Vec::new();
            let keys: Vec<_> = map.keys().cloned().collect();
            let len = keys.len();

            for i in 0..len {
                let mut new_map = map.clone();
                let key = &keys[i];
                new_map.insert(key.clone(), Value::String(payload.to_string()));
                results.push(Value::Object(new_map));
            }

            let mut all_payload_map = Map::new();
            for key in &keys {
                all_payload_map.insert(key.clone(), Value::String(payload.to_string()));
            }
            results.push(Value::Object(all_payload_map));

            results
        }
        _ => vec![Value::Object(Map::new())],
    }
}

pub fn json_sikkishter(init_val: Value, payloads: &[&str]) -> Vec<Value> {
    match init_val {
        Value::Object(map) => {
            let mut results = Vec::new();
            let keys: Vec<_> = map.keys().cloned().collect();
            let len = keys.len();
            let payloads_len = payloads.len();

            for i in 0..len {
                let mut new_map = map.clone();
                let key = &keys[i];
                new_map.insert(key.clone(), Value::String(payloads[i % payloads_len].to_string()));
                results.push(Value::Object(new_map));
            }

            let mut all_payload_map = Map::new();
            for (i, key) in keys.iter().enumerate() {
                all_payload_map
                    .insert(key.clone(), Value::String(payloads[i % payloads_len].to_string()));
            }
            results.push(Value::Object(all_payload_map));

            results
        }
        _ => vec![Value::Object(Map::new())],
    }
}

pub fn make_payloads_url(url: &str, payload: &str) -> Vec<String> {
    let mut weaponized_endpoints = Vec::new();
    let para_regex = Regex::new(r"(\?|\&)([^=]+)\=([^&]+)").unwrap();

    let parameters = para_regex
        .captures_iter(url)
        .filter_map(|cap| {
            cap.get(2).and_then(|name| cap.get(3).map(|value| (name.as_str(), value.as_str())))
        })
        .collect::<Vec<_>>();

    if !parameters.is_empty() {
        for (name, value) in &parameters {
            let modified_value = format!("{}{}", value, encode(&payload));

            let weaponized_url = parameters
                .iter()
                .map(|&(param_name, param_value)| {
                    if param_name == *name {
                        format!("{}={}", param_name, modified_value)
                    } else {
                        format!("{}={}", param_name, param_value)
                    }
                })
                .collect::<Vec<_>>()
                .join("&");

            let reconstructed_url =
                format!("{}?{}", &url[..url.find('?').unwrap() + 1], weaponized_url);

            let cleaned_url = reconstructed_url.replace("?&", "?").replace("??", "?");

            weaponized_endpoints.push(cleaned_url);
        }
    } else {
        let weaponized_url = format!("{}{}", url, payload);
        weaponized_endpoints.push(weaponized_url);
    }

    weaponized_endpoints
}

pub fn xml_sikkish(xml: &str, payload: &str) -> Vec<String> {
    let mut results = Vec::new();

    // Regex to match XML tags without backreferences
    let tag_regex = Regex::new(r"<([^>]+)>([^<]*)</[^>]+>").unwrap(); // Removed backreference from closing tag

    // Decode HTML entities before processing
    let decoded_xml = decode_html_entities(xml);

    let mut new_xml = String::new();
    let mut replaced = false;

    // Iterating over the decoded XML structure and replacing the first tag's value with the payload
    for caps in tag_regex.captures_iter(&decoded_xml) {
        let tag = caps.get(1).unwrap().as_str();
        let value = caps.get(2).unwrap().as_str();

        if !replaced {
            // Insert payload into the first tag value
            new_xml.push_str(&format!("<{}>{}</{}>", tag, payload, tag));
            replaced = true;
        } else {
            // Preserve the original value for other tags
            new_xml.push_str(&format!("<{}>{}</{}>", tag, value, tag));
        }
    }

    // Do not encode the final result; keep it in plain XML
    results.push(new_xml.to_string());

    // Also replace all tag values with the payload
    let all_payload_xml =
        tag_regex.replace_all(&decoded_xml, format!("<$1>{}</$1>", payload).as_str()).to_string();

    // Again, don't encode the final result
    results.push(all_payload_xml.to_string());

    results
}

// Multi-Payload XML Shuffling Function without backreferences
pub fn xml_sikkishter(xml: &str, payloads: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    let tag_regex = Regex::new(r"<([^>]+)>([^<]*)</[^>]+>").unwrap(); // Regex for matching XML tags
    let mut payload_index = 0;
    let payloads_len = payloads.len();

    // Decode HTML entities before processing
    let decoded_xml = decode_html_entities(xml);

    let mut new_xml = String::new();

    // Iterating over the decoded XML structure and replacing tag values with different payloads
    for caps in tag_regex.captures_iter(&decoded_xml) {
        let tag = caps.get(1).unwrap().as_str(); // Extracting tag name
        let payload = payloads[payload_index % payloads_len]; // Rotate through the payloads
        new_xml.push_str(&format!("<{}>{}</{}>", tag, payload, tag)); // Reconstruct the tag with new payload
        payload_index += 1;
    }

    // Encode the final result back to HTML
    results.push(encode_safe(&new_xml).to_string());

    // Replace all tags with the first payload for another result
    let all_payload_xml = tag_regex
        .replace_all(&decoded_xml, format!("<$1>{}</$1>", payloads[0]).as_str())
        .to_string();
    results.push(encode_safe(&all_payload_xml).to_string());

    results
}
