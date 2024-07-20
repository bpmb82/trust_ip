use actix_web::{
    dev::ConnectionInfo, get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use env_logger::Env;
use ip_in_subnet::iface_in_subnet;
use log::error;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::Deserialize;
use std::env;
use std::process::exit;

#[derive(Deserialize, Debug)]
struct Ips {
    items: Vec<Items>,
}

#[derive(Deserialize, Debug)]
struct Items {
    cidr: String,
}

// This struct represents state
struct AppState {
    app_name: String,
}

fn is_whitelisted_ip(ip: &str) -> bool {
    env::var("WHITELIST")
        .unwrap_or("".into())
        .split(',')
        .any(|v| v == ip)
}

async fn is_atlassian_ip(ip: &str) -> bool {
    let Ok(url) = env::var("ATLASSIAN_IP_URL") else {
        return false;
    };
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
            error!("Error occured querying the Atlassian URL: {}", err);
            return false;
        }
    };

    if response.status() != reqwest::StatusCode::OK {
        return false;
    }

    match response.json::<Ips>().await {
        Ok(parsed) => parsed
            .items
            .iter()
            .any(|range| iface_in_subnet(ip, &range.cidr).unwrap_or(false)),
        Err(err) => {
            error!("Unable to parse response from Atlassian URL: {}", err);
            false
        }
    }
}

async fn check_ip(ip: &str) -> bool {
    is_whitelisted_ip(ip) || is_atlassian_ip(ip).await
}

#[get("/health")]
async fn health(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name;
    HttpResponse::Ok().body(format!("Application name: {app_name}\nHealth status: OK"))
}

#[get("/")]
async fn auth(ip: ConnectionInfo) -> impl Responder {
    if let Some(host_ip) = ip.realip_remote_addr() {
        if check_ip(host_ip).await {
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

    HttpServer::new(|| {
        App::new()
            .wrap(
                Logger::new("%{r}a %r %s %b %{Referer}i %{User-Agent}i %T").log_target("trust_ip"),
            )
            .app_data(web::Data::new(AppState {
                app_name: String::from("trust_ip"),
            }))
            .service(health)
            .service(auth)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
