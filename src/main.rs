use std::env;
use std::error::Error;
use postgres::{Client, NoTls};

fn main() -> Result<(), Box<dyn Error>> {
    print!("{}", env::var("PASSWORD")?);
    let mut client = Client::connect(&format!(
        "host=tf-2025081018303032650000000a.cluster-c1y2k8e8uxdb.eu-central-1.rds.amazonaws.com user=postgres password={} dbname=db",
        env::var("PASSWORD")?
    ), NoTls)?;

    for row in client.query("SELECT id, url, sitemap FROM websites", &[])? {
        let id: i16 = row.get(0);
        let url: &str = row.get(1);
        let sitemap: &str = row.get(2);

        println!("found website: {} {} {:?}", id, url, sitemap);
    }

    Ok(())
}