use score::score_stream_server::ScoreStream;
use score::{StreamScoreRequest, StreamScoreResponse};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub mod score {
    tonic::include_proto!("score");
}

#[derive(Debug, Default)]
struct ScoreStreamService {}

#[tonic::async_trait]
impl ScoreStream for ScoreStreamService {
    type StreamScoreStream = ReceiverStream<Result<StreamScoreResponse, tonic::Status>>;
    async fn stream_score(
        &self,
        request: tonic::Request<StreamScoreRequest>,
    ) -> Result<tonic::Response<Self::StreamScoreStream>, tonic::Status> {
        println!("{:#?}", request);
        let (tx, rx) = mpsc::channel(4);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3"),
        );

        let req_client: reqwest::Client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Unable to build reqwest clinet");

        let url = "https://www.cricbuzz.com/cricket-match/live-scores";
        let response = get_website(&req_client, url)
            .await
            .expect("Something went wrong while fetching");
        let (match_id, match_link) = get_match_id(&response)
            .await
            .expect("Something went wrong in getting the current match link");

        let match_link_api: String =
            "https://www.cricbuzz.com/api/cricket-match/commentary/".to_string() + &match_id;

        //
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                // Call the scrape_content function and send the result
                let scrape_result =
                    scrape_content(&match_link_api, &match_link, &match_id, &req_client)
                        .await
                        .unwrap();

                tx.send(Ok(scrape_result)).await.unwrap();
                interval.tick().await; // Wait for 1 second
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let scx = ScoreStreamService::default();

    println!("ScoreStreamServer listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(score::score_stream_server::ScoreStreamServer::new(scx))
        .serve(addr)
        .await?;
    println!("Succesfully Ran");
    println!("Well we finished , This will never run");
    Ok(())
}

async fn get_website(
    client: &reqwest::Client,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
}

async fn get_match_id(response: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    println!();

    let document = scraper::Html::parse_document(response);
    let selector = scraper::Selector::parse(r#"[ng-show="active_match_type == 'league-tab'"]"#)
        .expect("Something went wrong while creating selector");
    let league_tab: String = document
        .select(&selector)
        .map(|element| element.html())
        .collect();
    let league_tab_html = scraper::Html::parse_document(&league_tab);

    let match_link_selector = scraper::Selector::parse("a.text-hvr-underline.text-bold").unwrap();
    let mut match_link_href: String = "".to_string();
    for element in league_tab_html.select(&match_link_selector) {
        match_link_href = element.attr("href").unwrap().to_string();
        break;
    }
    match_link_href = match_link_href.replace("/live-cricket-scores/", "/live-cricket-scorecard/");
    let parts: Vec<&str> = match_link_href.split('/').collect();
    let match_id_str: String = parts
        .get(2)
        .expect("Failed getting the match_id from the vector in get_match_id fn")
        .to_string();

    println!("The match id: {}", match_id_str);
    let match_link = "https://www.cricbuzz.com".to_string() + &match_link_href;
    println!("The match link: {}", match_link);
    Ok((match_id_str, match_link))
}

async fn scrape_content(
    url: &str,
    score_card_link: &str,
    match_id: &str,
    req_client: &reqwest::Client,
) -> Result<StreamScoreResponse, Box<dyn std::error::Error>> {
    println!("Scrape_content Function Started.......");
    let api_response: String = get_website(req_client, url)
        .await
        .expect("Failed fetching api_response");
    let res: serde_json::Value =
        serde_json::from_str(&api_response).expect("Failed To make json out of response");

    let mini_score = &res["miniscore"];
    let batsman = &mini_score["batsmanStriker"]["batName"];
    let nonStriker = &mini_score["batsmanNonStriker"]["batName"];
    let bowler = &mini_score["bowlerStriker"]["bowlName"];
    let currentRunRate = &mini_score["currentRunRate"];
    let mut bowlingTeam: serde_json::Value;
    let mut battingTeam: serde_json::Value;
    let innings = &mini_score["inningsId"];
    if innings == "2" {
        battingTeam =
            mini_score["matchScoreDetails"]["matchTeamInfo"][1]["battingTeamShortName"].clone();
        bowlingTeam =
            mini_score["matchScoreDetails"]["matchTeamInfo"][1]["bowlingTeamShortName"].clone();
    } else {
        battingTeam =
            mini_score["matchScoreDetails"]["matchTeamInfo"][0]["battingTeamShortName"].clone();
        bowlingTeam =
            mini_score["matchScoreDetails"]["matchTeamInfo"][0]["bowlingTeamShortName"].clone();
    }
    let score = &mini_score["batTeam"]["teamScore"];
    let wickets = &mini_score["batTeam"]["teamWkts"];
    let overs = &mini_score["overs"];
    let toss_winner = &mini_score["matchScoreDetails"]["tossResults"]["tossWinnerName"];
    let tossDecision = &mini_score["matchScoreDetails"]["tossResults"]["decision"];
    let team1 = &res["matchHeader"]["team1"]["shortName"];
    let team2 = &res["matchHeader"]["team2"]["shortName"];
    let (first_umpire, second_umpire, third_umpire) = get_umpires(&req_client).await.unwrap();
    let res = StreamScoreResponse {
        score: score.to_string(),
        batsman: batsman.to_string(),
        currentrunrate: currentRunRate.to_string(),
        battingteam: battingTeam.to_string(),
        bowlingteam: bowlingTeam.to_string(),
        innings: innings.to_string(),
        wickets: wickets.to_string(),
        overs: overs.to_string(),
        tosswinner: toss_winner.to_string(),
        tossdecision: tossDecision.to_string(),
        umpire1: first_umpire,
        umpire2: second_umpire,
        umpire3: third_umpire,
        venue: scrape_extra_content(&score_card_link, &match_id, &req_client)
            .await
            .unwrap(),
        matchid: "match_id".to_string(),
        team1: team1.to_string(),
        team2: team2.to_string(),
        bowler: bowler.to_string(),
        non_striker: nonStriker.to_string(),
    };
    Ok(res)
}

async fn scrape_extra_content(
    match_link: &str,
    match_id: &str,
    client: &reqwest::Client,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("The match link is: {}", match_link);
    let response = get_website(client, match_link)
        .await
        .expect("Something went wrong while fetching the match_link");
    let html = scraper::Html::parse_document(&response);
    let venue_selector = scraper::Selector::parse("span[itemprop=\"name\"]").unwrap();
    let mut venue = String::new();
    for element in html.select(&venue_selector) {
        venue = element.inner_html();
        break; // Exit the loop after finding the first match
    }
    println!("The venue of the match is : {}", venue);

    Ok(venue)
}

async fn get_umpires(
    client: &reqwest::Client,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    println!("Started to fetch umpires ......");
    let match_link = "https://www.cricbuzz.com/cricket-match-facts/89710/lsg-vs-pbks-11th-match-indian-premier-league-2024";
    let response = get_website(client, match_link)
        .await
        .expect("Something went wrong while fetching the match_link");
    let html = scraper::Html::parse_document(&response);
    let match_info_selector =
        scraper::Selector::parse(r#"div[class="cb-col cb-col-100 cb-col-rt"]"#).unwrap();
    let mut temp_html: scraper::Html = html.clone();
    for element in html.select(&match_info_selector) {
        let element = element.html();
        temp_html = scraper::Html::parse_fragment(&element);
        break;
    }
    let mut found_umpires: bool = false;
    let mut umpires: Vec<String> = Vec::new();
    //Started
    let div_selector = scraper::Selector::parse("div").unwrap();
    for element in temp_html.select(&div_selector) {
        if found_umpires {
            let umpires_temp: String = element.inner_html();
            let umpires_temp: Vec<_> = umpires_temp
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            umpires.extend(umpires_temp);

            found_umpires = false;
        } else {
            if element.inner_html() == "Umpires:" || element.inner_html() == "Third Umpire:" {
                found_umpires = true;
            }
        }
    }

    //Ended
    let mut umpires_iter = umpires.iter();
    let first = umpires_iter.next().unwrap().clone();
    let second = umpires_iter.next().unwrap().clone();
    let third = umpires_iter.next().unwrap().clone();

    Ok((first, second, third))
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
