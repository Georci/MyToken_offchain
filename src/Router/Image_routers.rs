use crate::Error::{ApiError, ImageError};
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

fn decode_base64_image(base64_image: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::decode(base64_image)
}

#[derive(Serialize, Deserialize)]
pub struct AddWatermarkRequest {
    base64_image: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddWatermarkResponse {
    pub cid: String,
    pub message: String,
}

#[post("/upload_image", data = "<image_data>")]
pub async fn upload_image(
    image_data: Json<AddWatermarkRequest>, // 前端提交的图片数据
    auth_user: AuthenticatedUser,          // 从 JWT 提取的用户信息
) -> Result<Json<AddWatermarkResponse>, Box<dyn ApiError>> {
    // 从请求中提取 base64 编码的图片数据
    let base64_image = image_data.base64_image.clone();

    // 调用 execute_watermark 处理图片
    let (watermarked_base64, watermark_base64) = match execute_watermark_base64(base64_image) {
        Ok((_watermarked_base64, _watermark_base64)) => (_watermarked_base64, _watermark_base64),
        Err(e) => {
            eprintln!("Watermark processing failed: {}", e);
            return Err(Box::new(e));
        }
    };

    // 调用 storage_image 存储到 IPFS 和数据库
    match storage_image(
        watermarked_base64,
        watermark_base64,
        auth_user.username, // 使用用户信息存储数据
    )
    .await
    {
        Ok(cid) => Ok(Json(AddWatermarkResponse {
            cid,
            message: "succeed save image".to_string(),
        })),
        Err(error) => Err(Box::new(error)),
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
) -> Result<Json<GetImageResponse>, Box<dyn ApiError>> {
    match download_file_by_cid_as_base64(&image_data.image_cid).await {
        Ok(image) => Ok(Json(GetImageResponse {
            image_base64: Some(image),
            message: "succeed to get image".to_string(),
        })),
        Err(error) => Err(Box::new(error)),
    }
}
