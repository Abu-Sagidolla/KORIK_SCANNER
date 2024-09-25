extern crate regex;
extern crate scraper;

use regex::Regex;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub parameters: Vec<String>,
    pub url: String,
}

pub fn get_methods(html_content: String) -> Vec<Method> {
    let document = Html::parse_document(&html_content);
    let form_selector = Selector::parse("form").unwrap();
    let input_selector = Selector::parse("input").unwrap();
    let textarea_selector = Selector::parse("textarea").unwrap();
    let button_selector = Selector::parse("button").unwrap();
    let select_selector = Selector::parse("select").unwrap();
    let option_selector = Selector::parse("option").unwrap();
    let script_selector = Selector::parse("script").unwrap(); // To capture JavaScript

    let mut output_data = Vec::new();

    // Regular expression to detect the XML payload inside xmlHttp.send()
    let xml_regex = Regex::new(r#"xmlHttp\.send\("(.*?)"\);"#).unwrap();
    let url_regex = Regex::new(r#"xmlHttp\.open\("POST","(.*?)""#).unwrap(); // To capture the URL used in xmlHttp.open()

    // Parsing HTML Forms
    for form in document.select(&form_selector) {
        let action = form.value().attr("action").unwrap_or("").to_string();
        let method = form.value().attr("method").unwrap_or("get").to_uppercase();
        let mut params = Vec::new();
        let mut submit_parameters = Vec::new();

        // Parse textarea elements inside form
        for textarea in form.select(&textarea_selector) {
            if let Some(name) = textarea.value().attr("name") {
                params.push(format!("{}=data", name));
            }
        }

        // Parse select elements and their options
        for select in form.select(&select_selector) {
            let name = select.value().attr("name").unwrap_or("").to_string();
            for option in select.select(&option_selector) {
                let value = option.value().attr("value").unwrap_or("");
                params.push(format!("{}={}", name, value));
            }
        }

        // Parse input elements inside form
        for input in form.select(&input_selector) {
            let name = input.value().attr("name").unwrap_or("");
            let input_type = input.value().attr("type").unwrap_or("");
            let value = input.value().attr("value").unwrap_or("");

            if input_type.contains("submit") {
                if !name.is_empty() {
                    submit_parameters.push(format!("{}={}", name, value));
                } else {
                    submit_parameters.push("#".to_string());
                }
            } else {
                params.push(format!("{}={}", name, value));
            }
        }

        // Parse button elements inside form
        for button in form.select(&button_selector) {
            let name = button.value().attr("name").unwrap_or("");
            let button_type = button.value().attr("type").unwrap_or("");
            let value = button.value().attr("value").unwrap_or("");

            if button_type.contains("submit") {
                submit_parameters.push(format!("{}={}", name, value));
            }
        }

        // Function to filter out parameters containing "bug=" or "security="
        let filter_params = |params: &Vec<String>| {
            !params.iter().any(|param| param.contains("bug=") || param.contains("security="))
        };

        // Add form data to output if it passes the filter
        for submit in &submit_parameters {
            let mut current_params = params.clone();
            current_params.push(submit.clone());
            if filter_params(&current_params) {
                output_data.push(Method {
                    name: method.clone(),
                    parameters: current_params,
                    url: action.clone(),
                });
            }
        }

        if submit_parameters.is_empty() && filter_params(&params) {
            output_data.push(Method {
                name: method.clone(),
                parameters: params.clone(),
                url: action.clone(),
            });
        }
    }

    // Check script tags for XML payloads and URLs
    for script in document.select(&script_selector) {
        let script_content = script.inner_html();

        // Extract XML from xmlHttp.send() calls
        if let Some(captures) = xml_regex.captures(&script_content) {
            if let Some(xml_payload) = captures.get(1) {
                // Find corresponding URL from xmlHttp.open() calls
                let url = if let Some(url_captures) = url_regex.captures(&script_content) {
                    url_captures
                        .get(1)
                        .map_or("Found in JavaScript".to_string(), |m| m.as_str().to_string())
                } else {
                    "Found in JavaScript".to_string()
                };

                if !xml_payload.as_str().contains("bug=")
                    && !xml_payload.as_str().contains("security=")
                {
                    output_data.push(Method {
                        name: "POST".to_string(),
                        parameters: vec![xml_payload.as_str().to_string()],
                        url,
                    });
                }
            }
        }
    }

    output_data
}
