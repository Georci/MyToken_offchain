use crate::Error::AppError;
use crate::IPFSImageStorage::storeImage::download_file_by_cid_as_base64;
use crate::IdentityAuthentication::Jwt::validate_token;
use crate::Router::routers::AuthenticatedUser;
use crate::WatermarkService::watermarkservice::{execute_watermark_base64, storage_image};
use base64::{decode, Engine};
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rocket::data::Data;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::io::AsyncWriteExt;
use rusttype::{Font, Scale};
use std::io;
use std::io::{Cursor, Error};
// #[derive(Deserialize)]
// pub struct ImageData {
//     pub base64_image: String, // 前端传来的 Base64 编码的图片
// }

// #[post("/upload", data = "<image_data>")]
// pub async fn upload_image(image_data: Json<ImageData>) -> String {
//     // 解码 Base64 图像
//     let img_data = match decode_base64_image(&image_data.base64_image) {
//         Ok(decoded_data) => decoded_data,
//         Err(_) => return "Failed to decode base64 image".to_string(),
//     };
//
//     // 将解码后的数据转为图片
//     let mut img = match DynamicImage::from_reader(Cursor::new(img_data)) {
//         Ok(image) => image,
//         Err(_) => return "Failed to create image from decoded data".to_string(),
//     };
//
//
//     // 调用外部 Python 脚本进行水印处理
//     match execute_watermark(img) {
//         Ok(_) => "Image processed and watermark applied.".to_string(),
//         Err(e) => format!("Error: {}", e),
//     }
// }
fn decode_base64_image(base64_image: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::decode(base64_image)
}

#[derive(Serialize, Deserialize)]
pub struct AddWatermarkRequest {
    base64_image: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddWatermarkResponse {
    pub cid: Option<String>,
    pub message: String,
}

/// upload_image
/// 上传图片 - 对原图进行数字水印，并将水印之后的图片保存到ipfs中
#[post("/upload_image", data = "<image_data>")]
pub async fn upload_image(
    image_data: Json<AddWatermarkRequest>, // 前端提交的图片数据
    auth_user: AuthenticatedUser,          // 从 JWT 提取的用户信息
) -> Json<AddWatermarkResponse> {
    // 从请求中提取 base64 编码的图片数据
    let base64_image = image_data.base64_image.clone();

    // 调用 execute_watermark 处理图片
    let (watermarked_base64, watermark_base64) = execute_watermark_base64(base64_image)
        .map_err(|e| AppError::Custom(e))
        .unwrap();

    // 调用 storage_image 存储到 IPFS 和数据库
    match storage_image(
        watermarked_base64,
        watermark_base64,
        auth_user.username, // 使用用户信息存储数据
    )
    .await
    {
        Ok(cid) => Json(AddWatermarkResponse {
            cid: Some(cid),
            message: "succeed save image ".to_string(),
        }),
        Err(error) => Json(AddWatermarkResponse {
            cid: None,
            message: "failed save image".to_string(),
        }),
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetImageRequest {
    image_cid: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetImageResponse {
    image_base64: Option<String>,
    message: String,
}

/// get_image
/// 获取图片 - 从ipfs中获取水印后的图片，任何人都可以从ipfs中获取图片，感觉也不需要身份验证
#[get("/get_image", data = "<image_data>")]
pub async fn get_image(
    image_data: Json<GetImageRequest>,
    auth_user: AuthenticatedUser,
) -> Json<GetImageResponse> {
    match download_file_by_cid_as_base64(&image_data.image_cid).await {
        Ok(image) => Json(GetImageResponse {
            image_base64: Some(image),
            message: "succeed to get image".to_string(),
        }),
        Err(error) => Json(GetImageResponse {
            image_base64: None,
            message: "failed to get image".to_string(),
        }),
    }
}
