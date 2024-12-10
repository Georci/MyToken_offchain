use base64::engine::general_purpose;
use base64::Engine;
use reqwest::multipart;
use reqwest::Client;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;

/// 将图片放入ipfs中，并返回其cid
async fn upload_and_pin_file(
    file_path: &str,
    ipfs_api_url: &str,
) -> Result<String, Box<dyn Error>> {
    // 创建 HTTP 客户端
    let client = Client::new();

    // 读取文件内容
    let mut file = File::open(file_path)?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)?;

    // 创建 multipart 表单
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(file_contents).file_name("filename"),
    );

    // 上传文件到 IPFS
    let res = client
        .post(&format!("{}/api/v0/add", ipfs_api_url))
        .multipart(form)
        .send()
        .await?;

    let res_text = res.text().await?;
    // 解析返回的 JSON 数据，获取 CID
    let res_json: serde_json::Value = serde_json::from_str(&res_text)?;
    let cid = res_json["Hash"].as_str().unwrap().to_string();

    println!("file uploaded, CID: {}", cid);

    // 固定文件
    let res = client
        .post(&format!("{}/api/v0/pin/add?arg={}", ipfs_api_url, cid))
        .send()
        .await?;

    let res_text = res.text().await?;
    println!("file has been pinned: {}", res_text);

    Ok(cid)
}

pub async fn upload_and_pin_base64(
    base64_data: &str,
    ipfs_api_url: &str,
) -> Result<String, Box<dyn Error>> {
    // 解码 Base64 数据为字节流
    let file_contents = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // 创建 HTTP 客户端
    let client = reqwest::Client::new();

    // 创建 multipart 表单
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(file_contents).file_name("watermarked_image.jpg"),
    );

    // 上传文件到 IPFS
    let res = client
        .post(&format!("{}/api/v0/add", ipfs_api_url))
        .multipart(form)
        .send()
        .await?;

    let res_text = res.text().await?;
    // 解析返回的 JSON 数据，获取 CID
    let res_json: serde_json::Value = serde_json::from_str(&res_text)?;
    let cid = res_json["Hash"]
        .as_str()
        .ok_or("Failed to extract CID from response")?
        .to_string();

    println!("File uploaded, CID: {}", cid);

    // 固定文件
    let res = client
        .post(&format!("{}/api/v0/pin/add?arg={}", ipfs_api_url, cid))
        .send()
        .await?;

    let res_text = res.text().await?;
    println!("File has been pinned: {}", res_text);

    Ok(cid)
}

pub fn get_cid() -> String {
    let token_urls =
        "https://ipfs.io/ipfs/QmRackxfCSTUg1GBSFGy6xMhFzNcfnR5vJDY8HSmaySNXF".to_string();
    // let watermarks = vec!["Gzhu".to_string()];
    // let capture_times = vec![U256::from(10)];
    // let capture_devices = vec!["huawei_mate60".to_string()];
    // let capture_companies = vec!["huawei".to_string()];
    // let submission_times = vec![U256::from(20)];
    // let submission_receivers = vec!["xiaomi".to_string()];
    token_urls
}

#[tokio::test]
async fn test_upload_and_pin_file() -> Result<(), Box<dyn Error>> {
    let file_path = "/root/BlockchainImage/UserInfo/Image-01/wukong.jpg";
    let ipfs_api_url = "http://192.168.0.10:5001";

    let cid = upload_and_pin_file(file_path, ipfs_api_url).await?;

    println!("file cid:{}", cid);

    Ok(())
}

/// 根据 CID 从 IPFS 下载文件并保存到本地
pub async fn download_file_by_cid(
    cid: &str,
    ipfs_api_url: &str,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    // 创建 HTTP 客户端
    let client = Client::new();

    // 构造下载 URL（通过 IPFS API 或公共网关）
    let url = format!("{}/api/v0/cat?arg={}", ipfs_api_url, cid);

    // 发送 GET 请求下载文件
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(format!("Failed to download file: HTTP {}", res.status()).into());
    }

    // 将文件内容保存到本地
    let mut file = File::create(output_path)?;
    let content = res.bytes().await?;
    file.write_all(&content)?;

    println!("File downloaded and saved to {}", output_path);

    Ok(())
}

/// 根据 CID 从 IPFS 下载文件并返回 Base64 编码的字符串
pub async fn download_file_by_cid_as_base64(cid: &str) -> Result<String, Box<dyn Error>> {
    // 创建 HTTP 客户端
    let client = Client::new();

    let ipfs_api_url = "http://192.168.0.10:5001"; // 替换为你的 IPFS 节点地址
                                                   // 构造下载 URL（通过 IPFS API 或公共网关）
    let url = format!("{}/api/v0/cat?arg={}", ipfs_api_url, cid);

    // 发送 GET 请求下载文件
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(format!("Failed to download file: HTTP {}", res.status()).into());
    }

    // 获取文件内容并转换为 Base64
    let content = res.bytes().await?;
    let base64_encoded = general_purpose::STANDARD.encode(content);

    Ok(base64_encoded)
}

#[tokio::test]
async fn test_download_file_by_cid() -> Result<(), Box<dyn Error>> {
    // 示例 CID
    let cid = "QmRackxfCSTUg1GBSFGy6xMhFzNcfnR5vJDY8HSmaySNXF";
    let ipfs_api_url = "http://192.168.0.10:5001"; // 替换为你的 IPFS 节点地址
    let output_path = "downloaded_image.jpg";

    // 下载文件
    download_file_by_cid(cid, ipfs_api_url, output_path).await?;

    Ok(())
}
