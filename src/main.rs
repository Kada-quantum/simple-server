use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use actix_files::Files;
use actix_web::{
    middleware::{ErrorHandlers, Logger, NormalizePath},
    App, HttpServer,
};

use clap::{ArgAction, Parser};

fn construct_path(p: &'static str) -> PathBuf {
    PathBuf::from(p)
}

fn verify_pathdir(s: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(s);
    if p.is_dir() {
        Ok(p)
    } else {
        Err(format!("{s} is not a directory"))
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "ADRESS", default_value_t = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), value_parser = str::parse::<IpAddr>)]
    adress: IpAddr,
    #[arg(short, long, value_name = "PORT", default_value_t = 8080, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
    #[arg(default_value = construct_path(".").into_os_string(), value_parser = verify_pathdir)]
    directory: PathBuf,
    #[arg(short, long, default_value_t = String::from("index.html"))]
    index: String,
    #[arg(short = 'e', long, default_value_t = true, action=ArgAction::SetFalse)]
    disable_etag: bool,
    #[arg(short = 'l', long, default_value_t = true, action=ArgAction::SetFalse)]
    disable_last_modified: bool,
    #[arg(short, long, default_value_t = false)]
    show_hidden: bool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .wrap(ErrorHandlers::default())
            .wrap(NormalizePath::trim())
            .wrap(Logger::default())
            .service({
                let mut service = Files::new("/", cli.directory.clone())
                    .prefer_utf8(true)
                    .show_files_listing()
                    .index_file(cli.index.clone())
                    .use_etag(cli.disable_etag)
                    .use_last_modified(cli.disable_last_modified);
                if cli.show_hidden {
                    service = service.use_hidden_files();
                }
                service
            })
    })
    .bind((cli.adress, cli.port))?
    .run()
    .await
}
