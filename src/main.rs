use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};
use std::env;

fn is_chinese(text: &str) -> bool {
    let re = Regex::new(r"[\u{4e00}-\u{9fff}]").unwrap();
    re.is_match(text)
}

fn get_translation(word: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
        ),
    );

    let url = format!("https://www.youdao.com/result?word={}&lang=en", word);
    let response = match client.get(&url).headers(headers).send() {
        Ok(resp) => resp,
        Err(_) => return "Failed to fetch translation.".to_string(),
    };

    let html = match response.text() {
        Ok(text) => text,
        Err(_) => return "Failed to read response.".to_string(),
    };

    let document = Html::parse_document(&html);
    let mut results = Vec::new();

    if is_chinese(word) {
        let word_exp_selector = Selector::parse("li.word-exp-ce.mcols-layout").unwrap();
        let point_selector = Selector::parse("a.point").unwrap();

        for exp in document.select(&word_exp_selector) {
            if let Some(word_text) = exp.select(&point_selector).next() {
                results.push(word_text.text().collect::<String>());
            }
        }
    } else {
        let trans_container_selector = Selector::parse("div.trans-container").unwrap();
        let word_exp_selector = Selector::parse("li.word-exp").unwrap();
        let pos_selector = Selector::parse("span.pos").unwrap();
        let trans_selector = Selector::parse("span.trans").unwrap();

        for container in document.select(&trans_container_selector) {
            for exp in container.select(&word_exp_selector) {
                if let (Some(pos), Some(trans)) = (
                    exp.select(&pos_selector).next(),
                    exp.select(&trans_selector).next(),
                ) {
                    let pos_text = pos.text().collect::<String>().trim().to_string();
                    let trans_text = trans.text().collect::<String>().trim().to_string();
                    results.push(format!("{}: {}", pos_text, trans_text));
                }
            }
        }
    }

    if results.is_empty() {
        "No results.".to_string()
    } else {
        results.join("\n")
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a word to translate");
        return;
    }
    println!("{}", get_translation(&args[1]));
}
