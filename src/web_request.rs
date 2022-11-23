///! All requests are blocking for ease of use.

// --------------------------------------------
// ADJUST THESE BEFORE FINAL WEB RELEASE
const DAY_INDEX: &'static str = "6";
const GAME_SECRET: &'static str = "lalalo";
// leave this empty for final release!
const GAME_TOKEN: &'static str = "zimt:45cb239358d04d27a3f00ed749c3c8a3";
// --------------------------------------------

const URL: &'static str = "https://lametta.awesome-co.de/";
const API_HIGHSCORE_GET: &'static str = "api/highscore/get/";
const API_HIGHSCORE_PUBLISH: &'static str = "api/highscore/redeem";

pub fn query_highscore(on_done: impl 'static + Send + FnOnce(ehttp::Result<ehttp::Response>)) {
    let url = format!("{URL}{API_HIGHSCORE_GET}{DAY_INDEX}");
    ehttp::fetch(ehttp::Request::get(url), on_done);
}

pub fn publish_score(
    score: u32,
    on_done: impl 'static + Send + FnOnce(ehttp::Result<ehttp::Response>),
) {
    let mut url = format!("{URL}{API_HIGHSCORE_PUBLISH}?secret={GAME_SECRET}&points={score}");
    if !GAME_TOKEN.is_empty() {
        url = format!("{url}&game_token={GAME_TOKEN}");
    }
    ehttp::fetch(ehttp::Request::get(url), on_done);
}
