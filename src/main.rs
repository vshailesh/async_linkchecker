use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use std::env;
use std::io::{self, BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread::sleep;
use tokio::task;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::VecDeque;
use std::sync::LazyLock;
use regex::{Regex};
use std::sync::atomic::{AtomicI32, Ordering};

fn link_helper_function(md_string: &str) -> Result<String, bool>{
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[*\]\((.*)\)").unwrap());
    if RE.is_match(md_string) {
        let Some(caps) = RE.captures(md_string) else { return Err(false) };
        Ok(caps[1].to_string())
    }
    else {
        Err(false)
    }
}

fn get_title(html_string: &str) -> Result<String, bool>{
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r".*<title>(.*)</title>.*").unwrap());
    if RE.is_match(html_string) {
        let Some(caps) = RE.captures(html_string) else { return Err(false) };
        Ok(caps[1].to_string())
    }
    else {
        Err(false)
    }
}

#[tokio::main]
async fn main() {
    let mut env_args = std::env::args();
    let mut input_file: Option<String> = None;
    let mut output_file: Option<String> = None;
    if env_args.len() == 2 {
        input_file = env_args.nth(1).clone();
    }

    if env_args.len() == 3 {
        let args = env_args.collect::<Vec<_>>();
        input_file = Some(args[1].clone());
        output_file = Some(args[2].clone());
    }
    //create a new output file 
    let counter = Arc::new(std::sync::atomic::AtomicI32::new(0));
    let mut outfile = if let Some(path) = output_file {
        File::create(path).await.unwrap()
    } else {
        File::create("/tmp/output.md").await.unwrap()
    };

    // wrap the File instance in Arc<Mutex<File>>
    let arc_out_file = Arc::new(Mutex::new(outfile));   
    let links_queue = Arc::new(Mutex::new(VecDeque::new()));

    // spawn a task that reads the markdown file and
    // extracts the links from the file. 
    tokio::spawn(find_links(links_queue.clone(), input_file));
    let tracker = tokio_util::task::TaskTracker::new();
    loop {
        if counter.load(Ordering::SeqCst) >= 32 || links_queue.clone().lock().await.len() == 0 {
            continue;
        }
        let q_clone = links_queue.clone();
        let url = q_clone.lock().await.pop_front().unwrap();
        if url == "EOF".to_string() {
            break;
        }
        counter.fetch_add(1, Ordering::SeqCst);
        tracker.spawn(check_url_liveness(url, counter.clone(), arc_out_file.clone()));
    }
    tracker.close();
    tracker.wait().await;
}

async fn find_links(links_queue: Arc<Mutex<VecDeque<String>>>, input_file: Option<String>) {
    let curr_dir = std::env::current_dir().unwrap();
    let mut markdown_file_path = PathBuf::new();
    markdown_file_path.push(curr_dir);
    if let Some(path) = input_file {
        markdown_file_path.push(path);
    } else {
        markdown_file_path.push("input_file.md");
    }
    let in_file = tokio::fs::File::open(markdown_file_path).await.unwrap();
    let mut buf_reader = tokio::io::BufReader::new(in_file);

    loop {
        if links_queue.lock().await.len() >= 100 {
            continue;
        }
        let mut buf_line = String::new();
        let line = buf_reader.read_line(&mut buf_line).await;

        // condition for EOF, if EOF reached stop processing the file.
        if let Ok(0) = line {
            let mut q_lock = links_queue.lock().await;
            q_lock.push_back("EOF".to_string());
            break;
        }
        
        // use regular expression to read the link between [link](description) expression.
        let extracted_link = link_helper_function(buf_line.as_str()); 
        match extracted_link {
            Ok(ex_link) => {
                let mut q_lock = links_queue.lock().await;
                q_lock.push_back(ex_link);
            } 
            Err(e) => {
                eprintln!("Error: invalid link {}", buf_line);
            }
        }
    } 
}

async fn check_url_liveness(url: String, counter: Arc<AtomicI32>, output_file: Arc<Mutex<File>>) {
    let url_new = url.as_str();
    let res = reqwest::get(url.clone()).await;
    match res {
        Ok(resp) => {
            match resp.status() {
                reqwest::StatusCode::OK => {
                    let body = resp.text().await.unwrap();
                    // let html_page = scraper::html::Html::parse_fragment(body.as_str());
                    // let selector = scraper::selector::Selector::parse("title").unwrap();
                    // let title_ref = html_page.select(&selector).next().unwrap().text().collect::<Vec<_>>();
                    // let title_vec = title_ref.text().collect::<Vec<_>>();
                    // let title = title_ref.get(0).unwrap().to_string();

                    let title = get_title(&body);
                    if let Ok(title) = title {
                        let mut out_file_lock = output_file.lock().await;
                        let input_line = format!("[{title}] ({url_new})\n");
                        let _ = out_file_lock.write(input_line.as_bytes()).await;
                    } else {
                        eprintln!("Something went wrong in Title Extraction: URL - {}", url_new);
                    }
                },
                _ => {
                    eprintln!("Error Occured: URL - {}", url_new);
                    let mut out_file_lock = output_file.lock().await;
                    let input_line = format!("[{}] ({url_new})\n", resp.status());
                    let _ = out_file_lock.write(input_line.as_bytes()).await;
                }
            }
        }
        Err(e) => {
            eprintln!("Error getting the URL itself: {}", e);
        }
    }
    counter.fetch_sub(1, Ordering::SeqCst);
}