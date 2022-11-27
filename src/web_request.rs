use bevy::prelude::info;

// --------------------------------------------
// ADJUST THESE BEFORE FINAL WEB RELEASE
const DAY_INDEX: &'static str = "1";
const GAME_SECRET: &'static str = "ApfJ2844Spg";
// leave this empty for final release!
const GAME_TOKEN: &'static str = "";
const URL: &'static str = "/";
// --------------------------------------------

const API_HIGHSCORE_GET: &'static str = "api/highscore/get/";
const API_HIGHSCORE_PUBLISH: &'static str = "api/highscore/redeem";

pub fn query_highscore(on_done: impl 'static + Send + FnOnce(ehttp::Result<ehttp::Response>)) {
    info!("Querying highscore...");
    let url = format!("{URL}{API_HIGHSCORE_GET}{DAY_INDEX}");
    ehttp::fetch(ehttp::Request::get(url), on_done);
}

pub fn publish_score(
    score: u32,
    on_done: impl 'static + Send + FnOnce(ehttp::Result<ehttp::Response>),
) {
    info!("Submitting highscore {score}...");
    let mut url = format!("{URL}{API_HIGHSCORE_PUBLISH}?secret={GAME_SECRET}&points={score}");
    if !GAME_TOKEN.is_empty() {
        url = format!("{url}&game_token={GAME_TOKEN}");
    }
    ehttp::fetch(ehttp::Request::get(url), on_done);
}
