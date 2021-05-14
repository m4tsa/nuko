use crate::cli::CliConfig;
use anyhow::Result;
use bus_queue::{bounded, Publisher, Subscriber};
use futures::{SinkExt, StreamExt};
use hyper::{
    header,
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Error, Request, Response, StatusCode,
};
use hyper_websocket_lite::{server_upgrade, AsyncClient};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use nuko_core::{config::SiteConfig, site::Site};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::mpsc::channel,
    sync::Mutex,
    thread,
    time::Duration,
};
use websocket_codec::Message;

lazy_static! {
    static ref UPDATE_BUS: (Mutex<Publisher<usize>>, Subscriber<usize>) = {
        let (publisher, subscriber) = bounded(1);

        (Mutex::new(publisher), subscriber)
    };
}

// Broadcast the new revision on websocket
fn broadcast_update(revision: usize) -> Result<()> {
    futures::executor::block_on(UPDATE_BUS.0.lock().unwrap().send(revision))?;

    Ok(())
}

fn build_site(cli_config: CliConfig, socket_addr: SocketAddr, out_path: PathBuf) -> Result<()> {
    let site_config = SiteConfig::read_file(cli_config.manifest_path())?;

    let mut site = Site::new(cli_config.root_path(), site_config, out_path)?;

    site.set_baseurl(&format!("http://{}", &socket_addr));
    site.set_liveupdate(true);
    site.load_content()?;

    site.build()?;

    Ok(())
}

fn render_404(out_path: &Path) -> Result<Response<Body>> {
    let _404_path = out_path.join("404.html");

    if _404_path.is_file() {
        let content = fs::read_to_string(_404_path)?;

        let mut res = Response::new(Body::from(content));

        *res.status_mut() = StatusCode::NOT_FOUND;
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_str("text/html")?,
        );

        Ok(res)
    } else {
        let mut res = Response::new(Body::from("404 page not found"));

        *res.status_mut() = StatusCode::NOT_FOUND;

        Ok(res)
    }
}

async fn handle_ws(mut client: AsyncClient) {
    let mut subscriber = UPDATE_BUS.1.clone();

    // Listen for revision updates
    loop {
        if let Some(revision) = subscriber.next().await {
            if client
                .send(Message::text(revision.to_string()))
                .await
                .is_err()
            {
                break;
            }
        } else {
            break;
        }
    }
}

async fn handle_request(req: Request<Body>, out_path: PathBuf) -> Result<Response<Body>> {
    let url_path = req.uri().path().to_string();

    // Upgrade websocket connections for live update
    if url_path == "/websocket" {
        match server_upgrade(req, handle_ws).await {
            Ok(res) => {
                return Ok(res);
            }
            Err(_) => unreachable!(),
        };
    }

    // Avoid path traversal
    let unsafe_path = out_path.join(url_path.strip_prefix("/").unwrap_or_else(|| &url_path));
    let path = match fs::canonicalize(&unsafe_path) {
        Ok(path) if path.starts_with(&out_path) => path,
        Ok(_) | Err(_) => return render_404(&out_path),
    };

    println!("{} {:?}", req.method().as_str(), req.uri().to_string());

    // Check if it is a path, if then look for the index.html or 404
    if path.exists() {
        if path.is_dir() {
            let index_path = path.join("index.html");
            if index_path.is_file() {
                let content = fs::read_to_string(index_path)?;

                let mut res = Response::new(Body::from(content));

                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_str("text/html")?,
                );

                Ok(res)
            } else {
                render_404(&out_path)
            }
        // It is a file
        } else {
            let content = fs::read(&path)?;
            let mime = mime_guess::from_path(&path).first_or_text_plain();

            let mut res = Response::new(Body::from(content));

            res.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_str(mime.to_string().as_str())?,
            );

            Ok(res)
        }
    } else {
        render_404(&out_path)
    }
}

pub fn cmd_serve(cli_config: CliConfig, socket_addr: SocketAddr, out_path: PathBuf) -> Result<()> {
    let root_path = cli_config.root_path();

    println!("Building site...");
    build_site(cli_config.clone(), socket_addr, out_path.clone())?;

    // Setup watcher
    let (tx, rx) = channel();
    let watch = &["Nuko.toml", "content", "static", "themes"];
    let mut watcher = watcher(tx, Duration::from_secs_f32(0.5)).unwrap();

    for watch_path in watch {
        watcher.watch(root_path.join(watch_path), RecursiveMode::Recursive)?;
    }

    // Spawn http server
    let serve_out_path = out_path.clone();
    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let make_svc = make_service_fn(move |_| {
                    let out_path = serve_out_path.clone();

                    async {
                        Ok::<_, Error>(service_fn(move |req| handle_request(req, out_path.clone())))
                    }
                });

                let server = Server::bind(&socket_addr).serve(make_svc);
                println!("Serving content at http://{}\n", &socket_addr);
                server.await
            })
    });

    // Watch for updates
    let mut revision = 0;
    // The bus will always have to contain one element when accepting a ws subscriber
    broadcast_update(revision)?;

    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Create(_path)
                | DebouncedEvent::Remove(_path)
                | DebouncedEvent::Write(_path)
                | DebouncedEvent::Rename(_, _path) => {
                    println!("Rebuilding site...");
                    if let Err(err) = build_site(cli_config.clone(), socket_addr, out_path.clone())
                    {
                        println!("Error rebuilding site: {}", err.to_string());
                    }

                    revision += 1;
                    broadcast_update(revision)?;
                }
                _ => {}
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
