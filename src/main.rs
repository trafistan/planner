use futures::StreamExt;
use http_body_util::BodyExt;
use http_body_util::Empty;
use hyper::http::Uri;
use hyper::Request;

static HTTP: std::sync::LazyLock<
    hyper_util::client::legacy::Client<
        hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
        http_body_util::Empty<bytes::Bytes>,
    >,
> = std::sync::LazyLock::new(|| {
    let mut tcp = hyper_util::client::legacy::connect::HttpConnector::new();
    tcp.enforce_http(false);
    tcp.set_nodelay(true);
    tcp.set_happy_eyeballs_timeout(Some(std::time::Duration::ZERO));

    hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .http2_adaptive_window(true)
        .pool_idle_timeout(None)
        .pool_max_idle_per_host(usize::MAX)
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_webpki_roots()
                .https_only()
                .enable_http1()
                .enable_http2()
                .wrap_connector(tcp),
        )
});

#[derive(Copy, Clone)]
struct Device {
    name: &'static str,
    oss: &'static [Os],
    screens: &'static [Screen],
    weight: f32,
}

#[derive(Copy, Clone)]
struct Os {
    name: &'static str,
    browsers: &'static [Browser],
    weight: f32,
}

#[derive(Copy, Clone)]
struct Browser {
    name: &'static str,
    user_agents: &'static [&'static str],
    weight: f32,
}

#[derive(Copy, Clone)]
struct Screen {
    resolution: &'static str,
    weight: f32,
}

static DATA: &[Device] = include!("data.gen.rs");

#[tokio::main]
async fn main() {
    let t = std::time::Instant::now();

    let (client, connect) = tokio_postgres::connect(
        &format!(
            "host=rds.trafistan.com user=postgres password={} dbname=db",
            std::env::var("PASSWORD").unwrap()
        ),
        tokio_postgres::NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(connect);

    let (websites, proxies, referrers) = tokio::try_join!(
        client.query("SELECT w.*, p.day, hour, FLOOR(amount * distribution / 100 * (0.95 + random() * 0.1))::int AS amount FROM websites w JOIN plans p ON w.id = p.website_id JOIN distributions d ON d.website_id = p.website_id AND d.day = EXTRACT(ISODOW FROM make_date(EXTRACT(YEAR FROM now())::int, EXTRACT(MONTH FROM now())::int, p.day)::date)::smallint WHERE p.day = EXTRACT(DAY FROM now())::smallint AND hour = EXTRACT(HOUR FROM now())::smallint;", &[]),
        client.query("SELECT website_id, url, weight FROM website_proxies JOIN proxies p ON proxy_id = p.id;", &[]),
        client.query("SELECT website_id, device, url, weight FROM website_referrers JOIN referrers p ON referrer_id = p.id;", &[]),
    ).unwrap();

    futures::stream::iter(websites).for_each_concurrent(None, |website| async move {
        let (url, sitemap): (&str, &str) = (website.get("url"), website.get("sitemap"));

        let request = Request::get(
            Uri::builder().scheme("https").authority(url).path_and_query({
                let mut path = String::with_capacity(sitemap.len() + 1);

                path.push('/');

                path += sitemap.strip_prefix('/').unwrap_or(sitemap);

                path
            }).build().unwrap(),
        ).body(Empty::new()).unwrap();

        let xml = 'o: {
            for _ in 0..5 {
                let Ok(response) = HTTP.request(request.clone()).await else {
                    continue;
                };

                if !response.status().is_success() {
                    continue;
                }

                let Ok(body) = response.into_body().collect().await else {
                    continue;
                };

                break 'o Some(body.to_bytes())
            }

            None
        };

        println!("{:?}", xml);
    }).await;

    println!("{:>7.3}", t.elapsed().as_secs_f64() * 1000.0);
}
