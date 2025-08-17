use reqwest::Client;
use tokio::fs;
pub async fn download_file(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let resp = client.get(url).send().await?;
    let bytes = resp.bytes().await?;
    Ok(bytes.to_vec())
}

pub async fn get_file_md5(file: &[u8]) -> String {
    format!("{:x}", md5::compute(file))
}

pub async fn save_file(file: &[u8], path: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, file).await?;
    Ok(())
}

pub fn get_file_extension(bytes: &[u8]) -> &str {
    if bytes.len() < 4 {
        return "bin";
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return "jpg";
    }
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return "png";
    }

    if bytes.len() > 12 && bytes[8..12] == [0x57, 0x45, 0x42, 0x50] {
        return "webp";
    }
    "bin"
}

pub fn is_image_content(bytes: &[u8]) -> bool {
    match get_file_extension(bytes) {
        "jpg" | "png" | "webp" => true,
        _ => false,
    }
}
