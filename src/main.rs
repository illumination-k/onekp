use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};

use reqwest::{Response, StatusCode};
use select::{document::Document, predicate::Name};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    thread::sleep,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SequenceType {
    Nucleotide,
    Protein,
    Both,
}

impl SequenceType {
    fn to_filenames(self) -> Vec<&'static str> {
        let nucleotide = "nucleotides.fa.gz";
        let protein = "protein.fa.gz";
        match self {
            Self::Nucleotide => vec![nucleotide],
            Self::Protein => vec![protein],
            Self::Both => vec![nucleotide, protein],
        }
    }
}

#[derive(Debug, Clone)]
pub struct OneKpRecord {
    id: String,
    clade: String,
    order: String,
    family: String,
    species: String,
    tissue_type: String,
    prefix: String,
}

impl OneKpRecord {
    pub fn to_filename(&self, filename: &str) -> String {
        format!("{}-{}", self.prefix, filename)
    }
    pub fn to_gigadb_url(&self, filename: &str) -> String {
        // https://ftp.cngb.org/pub/gigadb/pub/10.5524/100001_101000/100627/assemblies/
        format!("https://ftp.cngb.org/pub/gigadb/pub/10.5524/100001_101000/100627/assemblies/{}/{}-translated-{}", self.prefix, self.id, filename)
    }
}

#[derive(Debug, Clone)]
pub struct OneKp {
    links: Vec<String>,
    records: Vec<OneKpRecord>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OneKpKey {
    Id,
    Clade,
    Order,
    Family,
    Species,
    TissueType,
}

impl OneKp {
    pub fn new(table_index: &str) -> Self {
        let links = Document::from(table_index)
            .find(Name("a"))
            .filter_map(|n| n.attr("href"))
            .map(|n| n.trim_end_matches('/').to_string())
            .collect();

        Self {
            records: vec![],
            links,
        }
    }

    pub fn push_record(&mut self, attrs: Vec<&str>) -> Result<()> {
        let id = attrs[0].to_string();

        let prefix = self
            .links
            .iter()
            .find(|l| l.starts_with(&id))
            .ok_or_else(|| anyhow!("{} dirname is not found", id))?
            .to_owned();

        self.records.push(OneKpRecord {
            id,
            clade: attrs[1].to_string(),
            order: attrs[2].to_string(),
            family: attrs[3].to_string(),
            // clean data for gigadb
            species: attrs[4].to_string(),
            tissue_type: attrs[5].to_string(),
            prefix,
        });
        Ok(())
    }

    pub fn filter(&self, key: OneKpKey, values: &[String]) -> Vec<OneKpRecord> {
        match key {
            OneKpKey::Id => self
                .records
                .iter()
                .cloned()
                .filter(|r| values.contains(&r.id))
                .collect(),
            OneKpKey::Clade => self
                .records
                .iter()
                .cloned()
                .filter(|r| values.contains(&r.clade))
                .collect(),
            OneKpKey::Order => self
                .records
                .iter()
                .cloned()
                .filter(|r| values.contains(&r.order))
                .collect(),
            OneKpKey::Family => self
                .records
                .iter()
                .cloned()
                .filter(|r| values.contains(&r.family))
                .collect(),
            OneKpKey::Species => self
                .records
                .iter()
                .cloned()
                .filter(|r| values.contains(&r.species))
                .collect(),
            OneKpKey::TissueType => self
                .records
                .iter()
                .cloned()
                .filter(|r| values.contains(&r.tissue_type))
                .collect(),
        }
    }
}

#[derive(Debug)]
struct Client {
    interval_time: u64,
    max_retry: usize,
    last_fetch_time: Instant,
}

impl Client {
    pub fn new(interval_time: u64, max_retry: usize) -> Self {
        Self {
            interval_time,
            max_retry,
            last_fetch_time: Instant::now(),
        }
    }

    async fn _get(&mut self, url: &str) -> Result<Response> {
        let now = Instant::now();
        let duration = now.duration_since(self.last_fetch_time).as_secs();

        if duration < self.interval_time {
            sleep(Duration::from_secs(self.interval_time));
        }

        let resp = reqwest::get(url).await?;

        if resp.status() != StatusCode::OK {
            return Err(anyhow!("Error: {}", resp.status()));
        }

        self.last_fetch_time = Instant::now();

        Ok(resp)
    }

    pub async fn get(&mut self, url: &str) -> Result<Response> {
        for _ in 0..self.max_retry {
            match self._get(url).await {
                Ok(data) => return Ok(data),
                Err(err) => eprintln!("{}", err),
            }
        }

        Err(anyhow!(
            "Error {} times when fetching {}",
            self.max_retry,
            url
        ))
    }
}

async fn fetch_and_save(
    rec: &OneKpRecord,
    basedir: &Path,
    sequence_type: SequenceType,
    client: &mut Client,
) -> Result<()> {
    for filename in sequence_type.to_filenames().iter() {
        let path = basedir.join(rec.to_filename(filename));

        let f = File::create(path)?;
        let mut bw = BufWriter::new(f);
        bw.write_all(
            &client
                .get(&rec.to_gigadb_url(filename))
                .await?
                .bytes()
                .await?,
        )?;
    }

    Ok(())
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Fetch {
        #[arg(long, short)]
        rootdir: PathBuf,
        #[arg(long)]
        filter_key: OneKpKey,
        #[arg(long)]
        filter_values: Vec<String>,
        #[arg(long, short)]
        sequence_type: SequenceType,
    },
    Show {},
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut client = Client::new(3, 5);

    let f = client.get("https://ftp.cngb.org/pub/gigadb/pub/10.5524/100001_101000/100627/Sample-List-with-Taxonomy.tsv.csv").await?.text().await?;
    let table_index = client
        .get("https://ftp.cngb.org/pub/gigadb/pub/10.5524/100001_101000/100627/assemblies/")
        .await?
        .text()
        .await?;

    let mut onekp = OneKp::new(&table_index);
    for (i, line) in f.split('\n').map(|l| l.trim()).enumerate() {
        if i == 0 {
            continue;
        }

        // 0: sample_id, 1: clade, 2: order, 3: family, 4: species, 5: tissue_type
        let mut attrs: Vec<&str> = line.split('\t').collect();
        while attrs.len() < 6 {
            attrs.push("No data");
        }
        onekp.push_record(attrs)?;
    }

    match cli.commands {
        Commands::Fetch {
            rootdir,
            filter_key,
            filter_values,
            sequence_type,
        } => {
            for rec in onekp.filter(filter_key, filter_values.as_ref()).iter() {
                match fetch_and_save(rec, &rootdir, sequence_type, &mut client).await {
                    Ok(()) => {
                        eprintln!("Fetching sucess: {}", rec.species)
                    }
                    Err(err) => {
                        eprintln!("Fetchng {} failed because {}", rec.species, err)
                    }
                }
            }
        }
        Commands::Show {} => {}
    }
    Ok(())
}
