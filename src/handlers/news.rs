use axum::{response::IntoResponse, Json};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct News {
    pub id: String,
    pub title: String,
    pub url: String,
    pub image_url: String,
    pub date: String,
    pub category: String,
}

pub async fn get_news() -> impl IntoResponse {
    let client = Client::new();
    let url = "https://www.rsudkotabaubau.com/article";

    let res = match client.get(url).send().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("[news] HTTP error: {}", e);
            return Json(json!({"status": "error", "message": "Gagal menghubungi website RSUD"}));
        }
    };

    let body = match res.text().await {
        Ok(body) => body,
        Err(e) => {
            eprintln!("[news] Read body error: {}", e);
            return Json(json!({"status": "error", "message": "Gagal membaca konten website RSUD"}));
        }
    };

    let document = Html::parse_document(&body);
    let link_selector = Selector::parse("a").unwrap();
    let h2_selector = Selector::parse("h2").unwrap();

    let mut news_list = Vec::new();

    for (i, element) in document.select(&h2_selector).enumerate() {
        if i >= 5 {
            break;
        }

        if let Some(link_element) = element.select(&link_selector).next() {
            let title = link_element.text().collect::<Vec<_>>().join("");
            let href = link_element.value().attr("href").unwrap_or("").to_string();
            let full_url = if href.starts_with("http") {
                href
            } else {
                format!("https://www.rsudkotabaubau.com{}", href)
            };

            // Try to find an image in the parent or siblings
            let mut image_url = "https://images.unsplash.com/photo-1519494026892-80bbd2d6fd0d?q=80&w=800&auto=format&fit=crop".to_string();

            // Look into parent element (usually a container) for an img tag
            if let Some(parent) = element.parent().and_then(scraper::ElementRef::wrap) {
                let img_selector = Selector::parse("img").unwrap();
                if let Some(img) = parent.select(&img_selector).next() {
                    if let Some(src) = img.value().attr("src") {
                        image_url = if src.starts_with("http") {
                            src.to_string()
                        } else {
                            if src.starts_with('/') {
                                format!("https://www.rsudkotabaubau.com{}", src)
                            } else {
                                format!("https://www.rsudkotabaubau.com/{}", src)
                            }
                        };
                    }
                }
            }

            news_list.push(News {
                id: i.to_string(),
                title,
                url: full_url,
                image_url,
                date: "Terbaru".to_string(),
                category: "Berita".to_string(),
            });
        }
    }

    Json(json!({
        "status": "success",
        "data": news_list
    }))
}
