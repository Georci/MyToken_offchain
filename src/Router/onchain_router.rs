use crate::IdentityAuthentication::Jwt::validate_token;
use crate::Router::routers::AuthenticatedUser;
use crate::Transaction::sendTx::Handler;
use crate::Transaction::ContractMethod::{ContractMethod, ContractMethodResult};
use alloy::primitives::U256;
use rbatis::Error;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::Request;

/// 1.将图片信息上传到链上 2.从链上获取图片信息

#[derive(Serialize, Deserialize, Clone)]
pub struct UploadImageInfoRequest {
    token_uris: Vec<String>,
    to: String,
    quantity: U256,
    watermarks: Vec<String>,
    capture_times: Vec<U256>,
    capture_devices: Vec<String>,
    capture_companies: Vec<String>,
    submission_times: Vec<U256>,
    submission_receivers: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UploadImageInfoResponse {
    result: ContractMethodResult,
    message: String,
}

#[post("/upload_imageInfo", data = "<image_info>")]
pub async fn upload_imageInfo(
    image_info: Json<UploadImageInfoRequest>,
    auth_user: AuthenticatedUser,
) -> Json<UploadImageInfoResponse> {
    let func_call = ContractMethod::safeMint {
        to: image_info.to.parse().unwrap(),
        quantity: image_info.clone().quantity,
        _tokenURIs: image_info.token_uris.clone(),
        watermarks: Some(image_info.watermarks.clone()),
        captureTimes: Some(image_info.capture_times.clone()),
        captureDevices: Some(image_info.capture_devices.clone()),
        captureCompanies: Some(image_info.capture_companies.clone()),
        submissionTimes: Some(image_info.submission_times.clone()),
        submissionReceivers: Some(image_info.submission_receivers.clone()),
    };

    // 构造链上链下连接
    let http_url = "http://localhost:8545";
    let pk = "0x3ba5c6a17da00c75e9377e03ae98aa3dcdca7c4e537c84399125dfefa89be521";
    let user_address = "0x6d0d470a22c15a14817c51116932312a00ff00c8";
    let contract_address = "0xa6a0110367e24c541FC29124E8E89E3556263177";
    let handler: Handler =
        Handler::initialize_contract(http_url, pk, user_address, contract_address)
            .await
            .unwrap();
    // 上传信息到链上
    match handler.match_func(func_call).await {
        Ok(result) => Json(UploadImageInfoResponse {
            result,
            message: "succeed upload image to blockchain".to_string(),
        }),

        Err(error) => Json(UploadImageInfoResponse {
            result: ContractMethodResult::Error,
            message: "failed upload image to blockchain".to_string(),
        }),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetImageInfoRequest {
    image_id: u8,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetImageInfoResponse {
    result: ContractMethodResult,
    message: String,
}
#[get("/get_imageInfo", data = "<image_info>")]
pub async fn get_imageInfo(
    image_info: Json<GetImageInfoRequest>,
    auth_user: AuthenticatedUser,
) -> Json<GetImageInfoResponse> {
    let image_id = image_info.image_id;

    let func_call = ContractMethod::_imageInfo {
        tokenId: U256::try_from(image_id).unwrap(),
    };

    // 构造链上链下连接
    let http_url = "http://localhost:8545";
    let pk = "0x3ba5c6a17da00c75e9377e03ae98aa3dcdca7c4e537c84399125dfefa89be521";
    let user_address = "0x6d0d470a22c15a14817c51116932312a00ff00c8";
    let contract_address = "0xa6a0110367e24c541FC29124E8E89E3556263177";
    let handler: Handler =
        Handler::initialize_contract(http_url, pk, user_address, contract_address)
            .await
            .unwrap();
    match handler.match_func(func_call).await {
        Ok(imageInfo) => Json(GetImageInfoResponse {
            result: imageInfo,
            message: "succeed get image on blockchain".to_string(),
        }),
        Err(error) => Json(GetImageInfoResponse {
            result: ContractMethodResult::Error,
            message: "failed get image on blockchain".to_string(),
        }),
    }
}
