use clap::Parser;
use std::{collections::HashSet, path::Path, time::Duration};
use tokio::{fs, time::sleep, task};
use tracing_log::log;

mod file;
mod logger;

#[derive(Parser)]
#[command(author, version, about = "图片爬虫工具", long_about = None)]
struct Args {
    /// 要下载图片的URL地址
    #[arg(short, long)]
    url: String,
    /// 保存图片的目录
    #[arg(short, long, default_value = "./data")]
    path: String,
    /// 批量下载图片数量
    #[arg(short = 'n', long, default_value = "10")]
    count: u32,
}

#[tokio::main]
async fn main() {
    logger::log_init();
    let args = Args::parse();
    let url = args.url;
    let image_dir = args.path;
    if !Path::exists(Path::new(&image_dir)) {
        fs::create_dir(&image_dir).await.unwrap();
    }

    let mut saved_count = 0;
    let mut attempt = 0;
    let mut downloaded_md5s = HashSet::new();

    while saved_count < args.count {
        let batch_size = std::cmp::min(
            num_cpus::get(),
            (args.count - saved_count) as usize
        );

        let mut tasks = Vec::new();
        for _ in 0..batch_size {
            let url_clone = url.clone();
            let task = task::spawn(async move {
                match tokio::time::timeout(Duration::from_secs(30), file::download_file(&url_clone)).await {
                    Ok(Ok(bytes)) => Some(bytes),
                    _ => None,
                }
            });
            tasks.push(task);
        }

        let results = futures::future::join_all(tasks).await;

        for result in results {
            attempt += 1;
            match result {
                Ok(Some(bytes)) => {
                    if !file::is_image_content(&bytes) {
                        log::info!("第{}次下载跳过，内容不是图片", attempt);
                        continue;
                    }

                    let md5 = file::get_file_md5(&bytes).await;

                    if downloaded_md5s.contains(&md5) {
                        log::info!("第{}次下载跳过，MD5 {} 已存在", attempt, md5);
                        continue;
                    }

                    let extension = file::get_file_extension(&bytes);
                    let file_path = format!("{}/{}.{}", image_dir, md5, extension);

                    match file::save_file(&bytes, &file_path).await {
                        Ok(_) => {
                            saved_count += 1;
                            downloaded_md5s.insert(md5.clone());
                            log::info!(
                                "第{}次下载完成，文件MD5: {}，保存路径: {}",
                                saved_count,
                                md5,
                                file_path
                            );
                        }
                        Err(e) => {
                            log::error!("第{}次保存文件失败: {}", attempt, e);
                        }
                    }
                }
                Ok(None) => {
                    log::error!("第{}次下载失败或超时", attempt);
                }
                Err(e) => {
                    log::error!("第{}次下载任务执行出错: {}", attempt, e);
                }
            }

            if args.count != 50 && attempt % 50 == 0 {
                log::info!("已下载{}次，暂停5秒...", attempt);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
