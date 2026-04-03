use regex::Regex;
use reqwest::{blocking::Client, Url};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::steam::WALLPAPER_ENGINE_APP_ID;

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkshopItem {
    pub id: u64,
    pub title: String,
    pub creator: String,
    pub tags: Vec<String>,
    pub accent: String,
    pub preview: String,
    pub description: String,
    pub file_size: Option<String>,
    pub subscriptions: Option<String>,
    pub favorited: Option<String>,
    pub source: String,
}

pub fn browse_workshop(query: Option<String>) -> Result<Vec<WorkshopItem>, String> {
    let query = query.unwrap_or_default();
    let url = build_browse_url(&query);
    let client = Client::builder()
        .user_agent("wallpaper-engine-linux/0.1")
        .build()
        .map_err(|e| format!("Failed to create workshop HTTP client: {e}"))?;

    let html = client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|e| format!("Failed to fetch Steam workshop results: {e}"))?
        .text()
        .map_err(|e| format!("Failed to read Steam workshop results: {e}"))?;

    let items = parse_workshop_html(&html);
    if items.is_empty() {
        return Err("No workshop items were parsed from the Steam page.".into());
    }

    Ok(items)
}

pub fn fetch_workshop_item_details(id: u64) -> Result<WorkshopItem, String> {
    let client = Client::builder()
        .user_agent("wallpaper-engine-linux/0.1")
        .build()
        .map_err(|e| format!("Failed to create workshop HTTP client: {e}"))?;
    let html = client
        .get(format!(
            "https://steamcommunity.com/sharedfiles/filedetails/?id={id}"
        ))
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|e| format!("Failed to fetch workshop item details: {e}"))?
        .text()
        .map_err(|e| format!("Failed to read workshop item details: {e}"))?;

    parse_item_detail_html(id, &html).ok_or_else(|| {
        "The workshop item page loaded, but the metadata could not be parsed.".into()
    })
}

fn build_browse_url(query: &str) -> String {
    let mut url = Url::parse("https://steamcommunity.com/workshop/browse/")
        .expect("steam browse URL should always be valid");
    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.append_pair("appid", &WALLPAPER_ENGINE_APP_ID.to_string());
        query_pairs.append_pair("browsesort", "trend");
        query_pairs.append_pair("section", "readytouseitems");
        query_pairs.append_pair("requiredtags[]", "Any");
        query_pairs.append_pair("actualsort", "trend");
        query_pairs.append_pair("p", "1");

        let trimmed_query = query.trim();
        if !trimmed_query.is_empty() {
            query_pairs.append_pair("searchtext", trimmed_query);
        }
    }

    url.into()
}

fn parse_workshop_html(html: &str) -> Vec<WorkshopItem> {
    let container_re = Regex::new(
        r#"(?s)<a[^>]*href="(?:https://steamcommunity\.com)?/sharedfiles/filedetails/\?id=(\d+)[^"]*"[^>]*class="[^"]*workshopItem[^"]*"[^>]*>(.*?)</a>"#,
    )
    .expect("valid workshop container regex");
    let title_re = Regex::new(r#"(?s)<div[^>]*class="workshopItemTitle"[^>]*>(.*?)</div>"#)
        .expect("valid title regex");
    let creator_re = Regex::new(r#"(?s)<div[^>]*class="workshopItemAuthorName"[^>]*>(.*?)</div>"#)
        .expect("valid creator regex");
    let preview_re = Regex::new(r#"src="([^"]+)""#).expect("valid preview regex");
    let stats_re = Regex::new(r#"(?s)<div[^>]*class="workshopItemShortDesc"[^>]*>(.*?)</div>"#)
        .expect("valid desc regex");
    let subscriptions_re = Regex::new(r#"(?s)<span[^>]*class="subscriptions"[^>]*>(.*?)</span>"#)
        .expect("valid subscriptions regex");
    let favorited_re = Regex::new(r#"(?s)<span[^>]*class="favorited"[^>]*>(.*?)</span>"#)
        .expect("valid favorited regex");
    let filesize_re = Regex::new(r#"(?s)<span[^>]*class="fileSize"[^>]*>(.*?)</span>"#)
        .expect("valid size regex");

    container_re
        .captures_iter(html)
        .take(12)
        .filter_map(|capture| {
            let id = capture.get(1)?.as_str().parse::<u64>().ok()?;
            let body = capture.get(2)?.as_str();
            let title = title_re
                .captures(body)
                .and_then(|c| c.get(1))
                .map(|m| strip_html(m.as_str()))
                .filter(|text| !text.is_empty())?;

            let creator = creator_re
                .captures(body)
                .and_then(|c| c.get(1))
                .map(|m| strip_html(m.as_str()))
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| "Steam Workshop".into());

            let preview = preview_re
                .captures(body)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().replace("&amp;", "&"))
                .unwrap_or_default();

            let description = stats_re
                .captures(body)
                .and_then(|c| c.get(1))
                .map(|m| strip_html(m.as_str()))
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| "Steam workshop item".into());

            Some(WorkshopItem {
                id,
                title,
                creator,
                tags: infer_tags(&description),
                accent: accent_from_id(id),
                preview,
                description,
                file_size: filesize_re
                    .captures(body)
                    .and_then(|c| c.get(1))
                    .map(|m| strip_html(m.as_str()))
                    .filter(|value| !value.is_empty()),
                subscriptions: subscriptions_re
                    .captures(body)
                    .and_then(|c| c.get(1))
                    .map(|m| strip_html(m.as_str()))
                    .filter(|value| !value.is_empty()),
                favorited: favorited_re
                    .captures(body)
                    .and_then(|c| c.get(1))
                    .map(|m| strip_html(m.as_str()))
                    .filter(|value| !value.is_empty()),
                source: "live".into(),
            })
        })
        .collect()
}

fn parse_item_detail_html(id: u64, html: &str) -> Option<WorkshopItem> {
    let title_re = Regex::new(r#"(?s)<div[^>]*class="workshopItemTitle"[^>]*>(.*?)</div>"#).ok()?;
    let description_re =
        Regex::new(r#"(?s)<div[^>]*class="workshopItemDescription"[^>]*>(.*?)</div>"#).ok()?;
    let preview_re = Regex::new(r#"(?s)<img[^>]*id="previewImageMain"[^>]*src="([^"]+)""#).ok()?;
    let creator_re =
        Regex::new(r#"(?s)<div[^>]*class="friendBlockContent"[^>]*>\s*(.*?)<br"#).ok()?;
    let subscriptions_re =
        Regex::new(r#"(?s)<div[^>]*class="detailsStatRight"[^>]*>(.*?)</div>"#).ok()?;
    let tags_re = Regex::new(r#"(?s)<a[^>]*class="workshopTags"[^>]*>(.*?)</a>"#).ok()?;

    let title = title_re
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| strip_html(m.as_str()))?;

    let description = description_re
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| strip_html(m.as_str()))
        .unwrap_or_else(|| "Steam workshop item".into());

    let preview = preview_re
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().replace("&amp;", "&"))
        .unwrap_or_default();

    let creator = creator_re
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| strip_html(m.as_str()))
        .unwrap_or_else(|| "Steam Workshop".into());

    let tags = tags_re
        .captures_iter(html)
        .take(4)
        .filter_map(|capture| capture.get(1))
        .map(|m| strip_html(m.as_str()))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    let mut stats = subscriptions_re
        .captures_iter(html)
        .take(6)
        .filter_map(|capture| capture.get(1))
        .map(|m| strip_html(m.as_str()))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    let subscriptions = stats.first().cloned();
    let favorited = if stats.len() > 1 {
        Some(stats.remove(1))
    } else {
        None
    };

    Some(WorkshopItem {
        id,
        title,
        creator,
        tags: if tags.is_empty() {
            infer_tags(&description)
        } else {
            tags
        },
        accent: accent_from_id(id),
        preview,
        description,
        file_size: None,
        subscriptions,
        favorited,
        source: "live".into(),
    })
}

fn infer_tags(description: &str) -> Vec<String> {
    let description = description.to_lowercase();
    let mut tags = Vec::new();

    if description.contains("anime") {
        tags.push("Anime".into());
    }
    if description.contains("nature") || description.contains("forest") {
        tags.push("Nature".into());
    }
    if description.contains("city") || description.contains("neon") {
        tags.push("City".into());
    }
    if description.contains("abstract") || description.contains("ambient") {
        tags.push("Ambient".into());
    }
    if tags.is_empty() {
        tags.push("Workshop".into());
    }
    if tags.len() < 2 {
        tags.push("Wallpaper".into());
    }

    tags
}

fn strip_html(value: &str) -> String {
    let tag_re = Regex::new(r"<[^>]+>").expect("valid strip regex");
    let stripped = tag_re.replace_all(value, " ");
    stripped
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn accent_from_id(id: u64) -> String {
    let palette = [
        "#7ce2c3", "#ff7a59", "#6ea8fe", "#f6c666", "#d18fff", "#8bc34a",
    ];
    palette[(id as usize) % palette.len()].into()
}

pub fn unix_timestamp_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

#[cfg(test)]
mod tests {
    use super::{build_browse_url, parse_item_detail_html, parse_workshop_html};

    #[test]
    fn browse_url_encodes_search_text_safely() {
        let url = build_browse_url("rain & neon");

        assert!(url.contains("appid=431960"));
        assert!(url.contains("searchtext=rain+%26+neon"));
    }

    #[test]
    fn browse_parser_extracts_relative_links_and_creator_names() {
        let html = r#"
            <a href="/sharedfiles/filedetails/?id=2813557391" class="workshopItem">
              <img src="https://cdn.example.test/preview.jpg" />
              <div class="workshopItemTitle"> Neon City </div>
              <div class="workshopItemAuthorName"> Pixel Maker </div>
              <div class="workshopItemShortDesc">A neon city wallpaper with ambient rain.</div>
              <span class="subscriptions">12,345</span>
              <span class="favorited">678</span>
              <span class="fileSize">245 MB</span>
            </a>
        "#;

        let items = parse_workshop_html(html);
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.id, 2_813_557_391);
        assert_eq!(item.title, "Neon City");
        assert_eq!(item.creator, "Pixel Maker");
        assert_eq!(item.preview, "https://cdn.example.test/preview.jpg");
        assert_eq!(item.subscriptions.as_deref(), Some("12,345"));
        assert_eq!(item.favorited.as_deref(), Some("678"));
        assert_eq!(item.file_size.as_deref(), Some("245 MB"));
        assert!(item.tags.iter().any(|tag| tag == "City"));
    }

    #[test]
    fn detail_parser_extracts_title_creator_and_tags() {
        let html = r#"
            <div class="workshopItemTitle">Aurora Flow</div>
            <img id="previewImageMain" src="https://cdn.example.test/aurora.jpg&amp;size=large" />
            <div class="friendBlockContent">
              Aurora Artist<br>
            </div>
            <a class="workshopTags">Nature</a>
            <a class="workshopTags">Animated</a>
            <div class="workshopItemDescription">Calm ambient wallpaper with soft colors.</div>
            <div class="detailsStatRight">54,321</div>
            <div class="detailsStatRight">1,234</div>
        "#;

        let item = parse_item_detail_html(99, html).expect("item should parse");
        assert_eq!(item.id, 99);
        assert_eq!(item.title, "Aurora Flow");
        assert_eq!(item.creator, "Aurora Artist");
        assert_eq!(
            item.preview,
            "https://cdn.example.test/aurora.jpg&size=large"
        );
        assert_eq!(
            item.tags,
            vec!["Nature".to_string(), "Animated".to_string()]
        );
        assert_eq!(item.subscriptions.as_deref(), Some("54,321"));
        assert_eq!(item.favorited.as_deref(), Some("1,234"));
    }
}
