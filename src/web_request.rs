///! All requests are blocking for ease of use.

// --------------------------------------------
// ADJUST THESE BEFORE FINAL WEB RELEASE
const DAY_INDEX: &'static str = "6";
const GAME_SECRET: &'static str = "lalalo";
// leave this empty for final release!
const GAME_TOKEN: &'static str = "zimt:2a44f96df8c441028c4858ea846ea69b";
// --------------------------------------------

const URL: &'static str = "https://lametta.awesome-co.de/";
const API_HIGHSCORE_GET: &'static str = "api/highscore/get/";
const API_HIGHSCORE_PUBLISH: &'static str = "api/highscore/redeem";

use std::collections::HashMap;

pub fn query_highscore() -> reqwest::Result<HashMap<String, i32>> {
    let url = format!("{URL}{API_HIGHSCORE_GET}{DAY_INDEX}");
    reqwest::blocking::get(url).and_then(|result| result.json::<HashMap<String, i32>>())
}

pub fn publish_score(score: u32) -> reqwest::Result<reqwest::blocking::Response> {
    let mut url = format!("{URL}{API_HIGHSCORE_PUBLISH}?secret={GAME_SECRET}&points={score}");
    if !GAME_TOKEN.is_empty() {
        url = format!("{url}&game_token={GAME_TOKEN}");
    }
    reqwest::blocking::get(url)
}
