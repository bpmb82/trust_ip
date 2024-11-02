use actix_web::{
    dev::ConnectionInfo, get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use env_logger::Env;
use ip_in_subnet::iface_in_subnet;
use log::{error, info};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::Deserialize;
use std::env;
use std::process::exit;
use std::sync::Mutex;
s
#[derive(Deserialize, Debug)]
struct Ips {
    items: Vec<Items>,
}

#[derive(Deserialize, Debug)]
struct Items {
    cidr: String,
}

struct AppState {
    app_name: String,
    atlassian_ips: Mutex<Vec<String>>,
}

async fn fetch_atlassian_ips() -> Vec<String> {
    let url = match env::var("ATLASSIAN_IP_URL") {
        Ok(url) => url,
        Err(_) => return Vec::new(),
    };
    info!("Fetching Atlassian IPs from {}", url);

    let client = reqwest::Client::new();
    let response = match client
        .get(&url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await
    {
        Ok(result) => result,
        Err(err) => {
            error!("Error occurred querying the Atlassian URL: {}", err);
            return Vec::new();
        }
    };

    if response.status() != reqwest::StatusCode::OK {
        return Vec::new();
    }

    match response.json::<Ips>().await {
        Ok(parsed) => parsed.items.into_iter().map(|item| item.cidr).collect(),
        Err(err) => {
            error!("Unable to parse response from Atlassian URL: {}", err);
            Vec::new()
        }
    }
}


fn is_whitelisted_ip(ip: &str) -> bool {
    env::var("WHITELIST")
        .unwrap_or("".into())
        .split(',')
        .any(|v| v == ip)
}

async fn is_atlassian_ip(ip: &str, state: &web::Data<AppState>) -> bool {
    let stored_ips = state.atlassian_ips.lock().unwrap();
    stored_ips.iter().any(|range| iface_in_subnet(ip, range).unwrap_or(false))
}

#[get("/health")]
async fn health(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name;
    HttpResponse::Ok().body(format!("Application name: {app_name}\nHealth status: OK"))
}

#[get("/")]
async fn auth(ip: ConnectionInfo, data: web::Data<AppState>) -> impl Responder {
    if let Some(host_ip) = ip.realip_remote_addr() {
        if is_whitelisted_ip(host_ip) || is_atlassian_ip(host_ip, &data).await {
            return HttpResponse::Ok().body("Authorized");
        }
    }
    HttpResponse::Unauthorized().body("Unauthorized")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Seconds))
        .init();

    if env::var("WHITELIST").is_err() && env::var("ATLASSIAN_IP_URL").is_err() {
        error!("No environment variables were set, please set either WHITELIST or ATLASSIAN_IP_URL environment var");
        exit(1);
    }

    let app_state = web::Data::new(AppState {
        app_name: String::from("trust_ip"),
        atlassian_ips: Mutex::new(Vec::new()),
    });
    
    let state_clone = app_state.clone();
    tokio::spawn(async move {
        loop {
            let ips = fetch_atlassian_ips().await;
            {
                let mut stored_ips = state_clone.atlassian_ips.lock().unwrap();
                *stored_ips = ips;
            }
            tokio::time::sleep(std::time::Duration::from_secs(300)).await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%{r}a %r %s %b %{Referer}i %{User-Agent}i %T").log_target("trust_ip"))
            .app_data(app_state.clone())
            .service(health)
            .service(auth)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

