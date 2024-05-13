extern crate scraper;
extern crate reqwest;
extern crate tokio;
extern crate chrono;

use scraper::{Html, Selector};
use chrono::Utc;
use tokio::task::JoinHandle;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {

    let url: &str = "https://www.lornasiggins.com/";
    let response: String = reqwest::get(url).await?.text().await?;
    let document: Html = Html::parse_document(&response);

    let article_section_selector: Selector = Selector::parse("#showcase div.showcase-item").unwrap();
    let audio_section_selector: Selector = Selector::parse("#showcase2 div.showcase-item").unwrap();

    transpose_all_articles(&document, article_section_selector, "output/articles/".into()).await;
    transpose_all_articles(&document, audio_section_selector, "output/audio/".into()).await;

    Ok(())

}

async fn transpose_all_articles(document: &Html, selector: Selector, directory: String) -> (){

    for article in document.select(&selector) {
        let directory_copy: String = directory.clone();
        let article_html: String = article.html();
        let _handle: JoinHandle<()> = tokio::task::spawn( async move {
            generate_article(article_html, directory_copy);
        });
    }

}

fn generate_article(article_html: String, directory: String) -> (){

    let article = Html::parse_fragment(&article_html);

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
        let title_text: String = title_element.text().collect::<Vec<_>>().join("");
        title = title_text.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    
    if let Some(image_element) = article.select(&image_selector).next() {
        image_href = image_element.value().attr("src").unwrap_or("");
    }

    if let Some(link_element) = article.select(&link_selector).next() {
        link = link_element.value().attr("href").unwrap_or("");
    }

    if let Some(blurb_text) = article.select(&blurb_selector).next(){
        let blurb_text_raw = blurb_text.text().collect::<Vec<_>>().join(" ");
        blurb = blurb_text_raw.split_whitespace().collect::<Vec<_>>().join(" ");
    }

    let mut file_path: String = String::from(directory);
    let sanitized_title: String = sanitize_title(&title);
    file_path.push_str(&sanitized_title);
    file_path.push_str(".md");
    
    println!("{} - Writing {} to disk at {}...", time_str, title, file_path);
    let mut file = File::create(Path::new(&file_path)).expect("Unable to create file");
    writeln!(file, "---").expect("Unable to write to file");
    writeln!(file, "title: '{}'", title ).expect("Unable to write to file");
    writeln!(file, "date: '{}'", time_str).expect("Unable to write to file");
    writeln!(file, "image: '{}'", image_href).expect("Unable to write to file");
    writeln!(file, "blurb: '{}'", blurb).expect("Unable to write to file");
    writeln!(file, "link: '{}'", link).expect("Unable to write to file");
    writeln!(file, "---").expect("Unable to write to file");

}

fn sanitize_title(filename: &str) -> String {
    let mut sanitized = filename.to_owned(); // Only convert once
    let invalid_chars = [':', '<', '>', '/', '\\', '|', '?', '*', '"', '\''];
    sanitized.retain(|c| !invalid_chars.contains(&c)); // More efficient filtering
    sanitized.trim().to_owned()
}  
