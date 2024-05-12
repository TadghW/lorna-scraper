extern crate scraper;
extern crate reqwest;
extern crate tokio;
extern crate chrono;

use scraper::{Html, Selector};
use chrono::Utc;
use uuid::Uuid;
use tokio::task::JoinHandle;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {

    let url: &str = "https://www.lornasiggins.com/";
    let payload: String = reqwest::get(url).await?.text().await?;
    let html: Html = Html::parse_document(&payload);
    let article_selector: Selector = Selector::parse("div.showcase-item").unwrap();
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for article in html.select(&article_selector) {
        let article_html: String = article.html();
        let handle: JoinHandle<()> = tokio::task::spawn( async move {
            generate_article(article_html);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}

fn generate_article(article_html: String) -> (){

    let article = Html::parse_fragment(&article_html);
    let uuid: String = Uuid::new_v4().to_string();

    let image_selector: Selector = Selector::parse("img.showcase-image").unwrap();
    let title_selector: Selector = Selector::parse("div.showcase-title b").unwrap();
    let blurb_selector: Selector = Selector::parse("div.showcase-text:not(b)").unwrap();
    let link_selector: Selector = Selector::parse("a").unwrap();

    let time: chrono::prelude::DateTime<Utc> = Utc::now();
    let time_str: String = time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut image_href: &str = "NULL";
    let mut title: String = "NULL".to_string();
    let mut blurb: String = "NULL".to_string();
    let mut link: &str = "NULL";

    if let Some(title_element) = article.select(&title_selector).next() {
        title = title_element.text().collect::<Vec<_>>().join(" ");
    }
    
    if let Some(image_element) = article.select(&image_selector).next() {
        image_href = image_element.value().attr("src").unwrap_or("");
    }

    if let Some(link_element) = article.select(&link_selector).next() {
        link = link_element.value().attr("href").unwrap_or("");
    }

    if let Some(blurb_text) = article.select(&blurb_selector).next(){
        blurb = blurb_text.text().collect::<Vec<_>>().join(" ");
    }

    let mut file_path: String = String::from("output/");
    file_path.push_str(&uuid);
    file_path.push_str(".md");
    
    let mut file = File::create(Path::new(&file_path)).expect("Unable to create file");
    println!("{} - Writing {} to disk...", time_str, title);
    writeln!(file, "---").expect("Unable to write to file");
    writeln!(file, "title: '{}'", title ).expect("Unable to write to file");
    writeln!(file, "date: '{}'", time_str).expect("Unable to write to file");
    writeln!(file, "image: '{}'", image_href).expect("Unable to write to file");
    writeln!(file, "blurb: '{}'", blurb).expect("Unable to write to file");
    writeln!(file, "link: '{}'", link).expect("Unable to write to file");
    writeln!(file, "---").expect("Unable to write to file");

}