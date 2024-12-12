use crate::DataBase::{get_db, Images, Users};
use crate::IPFSImageStorage::storeImage::*;
use base64::engine::general_purpose;
use base64::Engine;
use image::{codecs::jpeg::JpegEncoder, DynamicImage, ImageFormat};
use rbatis::Error;
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::process::{Command, Stdio};
use std::thread;

#[derive(Deserialize)]
struct PythonOutput {
    watermarked_image: Option<String>,
    watermark_image: Option<String>,
    error: Option<String>,
}

// 调用外部 Python 脚本
pub fn execute_watermark_base64(base64_image: String) -> Result<(String, String), String> {
    let python_script = "/root/BlockchainImage/python-watermark/main_class.py";

    // 解码 Base64 图片为字节流
    let img_bytes = general_purpose::STANDARD
        .decode(&base64_image)
        .map_err(|_| "Failed to decode base64 image".to_string())?;

    // 将字节流转换为 DynamicImage
    let img = image::load_from_memory(&img_bytes)
        .map_err(|_| "Failed to decode image bytes".to_string())?;

    // 将图片保存为 .jpg 格式并处理
    let mut img_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut img_bytes);

    img.write_to(&mut cursor, ImageFormat::Jpeg)
        .map_err(|_| "Failed to write image to bytes")?;

    // 获取处理后的字节流并进行 Base64 编码
    let encoded_image = general_purpose::STANDARD.encode(&img_bytes);

    // 创建 JSON 对象
    let json_data = json!({
        "base64_image": encoded_image
    });

    // 将 JSON 数据写入文件
    let file_path = "encoded_image.json";
    let mut file = File::create(file_path).expect("Unable to create file");
    file.write_all(json_data.to_string().as_bytes())
        .expect("Unable to write data");

    // 调用外部 Python 脚本处理水印
    let mut child = Command::new("python3")
        .arg(python_script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    // 写入 JSON 数据到 Python 脚本的 stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(json_data.to_string().as_bytes())
            .map_err(|_| "Failed to write to stdin".to_string())?;
        // 写入完毕后立即关闭stdin，告知python输入结束
        drop(stdin);
    }

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // 使用线程并行读取 stdout 和 stderr
    let stdout_thread = thread::spawn(move || {
        let mut output = String::new();
        let mut reader = std::io::BufReader::new(stdout);
        reader
            .read_to_string(&mut output)
            .map(|_| output)
            .map_err(|e| format!("Failed to read stdout: {}", e))
    });

    let stderr_thread = thread::spawn(move || {
        let mut output = String::new();
        let mut reader = std::io::BufReader::new(stderr);
        reader
            .read_to_string(&mut output)
            .map(|_| output)
            .map_err(|e| format!("Failed to read stderr: {}", e))
    });

    // 等待线程完成并收集结果
    let stdout_result = stdout_thread.join().unwrap();
    let stderr_result = stderr_thread.join().unwrap();

    let output = stdout_result?;
    let stderr = stderr_result?;

    // 等待子进程完成
    let status = child
        .wait()
        .map_err(|_| "Failed to wait on child process".to_string())?;

    if status.success() {
        // 解析 Python 脚本的输出
        let parsed_output: PythonOutput = serde_json::from_str(&output)
            .map_err(|e| format!("Failed to parse JSON output: {}", e))?;

        if let (Some(wm_image), Some(wm)) = (
            parsed_output.watermarked_image,
            parsed_output.watermark_image,
        ) {
            return Ok((wm_image, wm));
        }

        if let Some(error_msg) = parsed_output.error {
            Err(error_msg)
        } else {
            Err("Python script did not return expected data".to_string())
        }
    } else {
        if !stderr.is_empty() {
            Err(format!("Python script error: {}", stderr))
        } else {
            Err("Python script failed".to_string())
        }
    }
}

// 调用外部 Python 脚本
pub fn execute_watermark_jpg(img: DynamicImage) -> Result<(String, String), String> {
    let python_script = "/root/BlockchainImage/python-watermark/main_class.py";

    // 将图片保存为 .jpg 格式
    let mut img_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut img_bytes);

    img.write_to(&mut cursor, ImageFormat::Jpeg)
        .map_err(|_| "Failed to write image to bytes")?;

    // 获取处理后的字节流并进行 Base64 编码
    let encoded_image = general_purpose::STANDARD.encode(&img_bytes);

    // 创建 JSON 对象
    let json_data = json!({
        "base64_image": encoded_image
    });

    // 调用外部 Python 脚本处理水印
    let mut child = Command::new("python3")
        .arg(python_script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    // 写入 JSON 数据到 Python 脚本的 stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(json_data.to_string().as_bytes())
            .map_err(|_| "Failed to write to stdin".to_string())?;
        // 写入完毕后立即关闭stdin，告知python输入结束
        drop(stdin);
    }

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // 使用线程并行读取 stdout 和 stderr
    let stdout_thread = thread::spawn(move || {
        let mut output = String::new();
        let mut reader = std::io::BufReader::new(stdout);
        reader
            .read_to_string(&mut output)
            .map(|_| output)
            .map_err(|e| format!("Failed to read stdout: {}", e))
    });

    let stderr_thread = thread::spawn(move || {
        let mut output = String::new();
        let mut reader = std::io::BufReader::new(stderr);
        reader
            .read_to_string(&mut output)
            .map(|_| output)
            .map_err(|e| format!("Failed to read stderr: {}", e))
    });

    // 等待线程完成并收集结果
    let stdout_result = stdout_thread.join().unwrap();
    let stderr_result = stderr_thread.join().unwrap();

    let output = stdout_result?;
    let stderr = stderr_result?;

    // 等待子进程完成
    let status = child
        .wait()
        .map_err(|_| "Failed to wait on child process".to_string())?;

    if status.success() {
        // 解析 Python 脚本的输出
        let parsed_output: PythonOutput = serde_json::from_str(&output)
            .map_err(|e| format!("Failed to parse JSON output: {}", e))?;

        if let (Some(wm_image), Some(wm)) = (
            parsed_output.watermarked_image,
            parsed_output.watermark_image,
        ) {
            return Ok((wm_image, wm));
        }

        if let Some(error_msg) = parsed_output.error {
            Err(error_msg)
        } else {
            Err("Python script did not return expected data".to_string())
        }
    } else {
        if !stderr.is_empty() {
            Err(format!("Python script error: {}", stderr))
        } else {
            Err("Python script failed".to_string())
        }
    }
}

/// todo: 换到ipfs中

// 将带水印的图片存放到ipfs，将水印本身存放到数据库
pub async fn storage_image(
    watermarked_base64: String,
    watermark_base64: String,
    username: String,
) -> Result<String, Error> {
    // 获取数据库连接池
    let rb = get_db().await;
    let mut user_id;
    let mut image_cid = String::new();

    // 将数字水印保存到数据库
    match Users::select_by_username(rb, &username).await {
        Ok(Some(mut user)) => {
            user_id = user.id;
            user.watermark_base64 = Some(watermark_base64.clone());
            println!("successed to save watermark {}", watermark_base64.clone());
        }
        Err(_) => {
            println!("Failed to find user with username {}", username);
            return Err(Error::E("Failed to find user with username".to_string()));
        }
        _ => {
            println!("Failed to find user with username {}", username);
            return Err(Error::E("Failed to find user with username".to_string()));
        }
    }

    // 将水印图片保存到ipfs
    let ipfs_api_url = "http://192.168.0.7:5001";
    match upload_and_pin_base64(&watermarked_base64, ipfs_api_url).await {
        Ok(cid) => {
            image_cid = cid.clone();
            println!("successed to save picture {}", cid);
        }
        Err(e) => {
            println!("Failed to save picture in ipfs{}", e);
            return Err(Error::E("Failed to save picture in ipfs".to_string()));
        }
    };

    // 保存图片到数据库中
    let image_table = Images {
        id: None,
        cid: Some(image_cid.clone()),
        user_id,
    };
    let data = Images::insert(rb, &image_table).await?;
    println!("insert = {}", json!(data));
    Ok(image_cid)
}

// #[test]
// pub fn test_execute_watermark() {
//     let image_path = "/home/kenijima/usr/work/ImageService/UserInfo/Image-01/wukong.jpg";
//     let image = image::open(image_path).expect("Failed to load test image");

//     let result = execute_watermark_base64(image.to_string());

//     // 获取带水印的图像和水印图像的 Base64 编码
//     let (watermarked_base64, watermark_base64) = result.clone().unwrap();

//     // 解码 Base64 字符串为字节流
//     let watermarked_bytes = general_purpose::STANDARD
//         .decode(watermarked_base64)
//         .expect("Failed to decode watermarked image base64");
//     let watermark_bytes = general_purpose::STANDARD
//         .decode(watermark_base64)
//         .expect("Failed to decode watermark image base64");

//     // 保存到文件以便手动验证
//     let output_watermarked_path = "UserInfo/Image-01/wukong_watermarked_test.jpg";
//     let output_watermark_path = "UserInfo/Image-01/wukong_watermark_test.jpg";

//     std::fs::write(output_watermarked_path, watermarked_bytes)
//         .expect("Failed to save watermarked image");
//     std::fs::write(output_watermark_path, watermark_bytes).expect("Failed to save watermark image");
//     assert!(result.is_ok(), "Watermarking failed: {:?}", result.err());
// }
