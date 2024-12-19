use crate::Error::{ApiError, BlockchainError, ImageError};
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
    token_id: Vec<U256>,
}

#[post("/upload_imageInfo", data = "<image_info>")]
pub async fn upload_imageInfo(
    image_info: Json<UploadImageInfoRequest>,
    auth_user: AuthenticatedUser,
) -> Result<Json<UploadImageInfoResponse>, Box<dyn ApiError>> {
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
        match Handler::initialize_contract(http_url, pk, user_address, contract_address).await {
            Ok(handler) => handler,
            Err(_initialize_error) => {
                return Err(Box::new(_initialize_error));
            }
        };

    let find_call = ContractMethod::totalSupply;
    let start_id_option = match handler.match_func(find_call.clone()).await {
        Ok(id) => extract_u256(id),
        Err(e) => {
            return Err(Box::new(e));
        }
    };
    let start_id = match start_id_option {
        Some(id) => id,
        None => {
            return Err(Box::new(BlockchainError::ContractCallError(
                "convert error".to_string(),
            )));
        }
    };
    // 上传信息到链上
    match handler.match_func(func_call).await {
        Ok(result) => {
            let end_id_option = match handler.match_func(find_call.clone()).await {
                Ok(id) => extract_u256(id),
                Err(e) => {
                    return Err(Box::new(e));
                }
            };
            let end_id = match end_id_option {
                Some(id) => id,
                None => {
                    return Err(Box::new(BlockchainError::ContractCallError(
                        "convert error".to_string(),
                    )));
                }
            };
            let token_ids: Vec<U256> = U256Range::new(start_id, end_id).collect();
            Ok(Json(UploadImageInfoResponse {
                result,
                token_id: token_ids,
            }))
        }
        Err(error) => Err(Box::new(error)),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetImageInfoRequest {
    image_id: u8,
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
) -> Result<Json<GetImageInfoResponse>, Box<dyn ApiError>> {
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
        match Handler::initialize_contract(http_url, pk, user_address, contract_address).await {
            Ok(handler) => handler,
            Err(_initialize_error) => {
                return Err(Box::new(_initialize_error));
            }
        };

    match handler.match_func(func_call).await {
        Ok(imageInfo) => Ok(Json(GetImageInfoResponse {
            result: imageInfo,
            message: "succeed get image on blockchain".to_string(),
        })),
        Err(error) => Err(Box::new(error)),
    }
}

fn extract_u256(result: ContractMethodResult) -> Option<U256> {
    match result {
        ContractMethodResult::U256(value) => Some(value), // 提取 U256 值
        _ => None,                                        // 其他情况返回 None
    }
}

// 自定义范围迭代器
struct U256Range {
    current: U256,
    end: U256,
}

impl U256Range {
    fn new(start: U256, end: U256) -> Self {
        Self {
            current: start,
            end,
        }
    }
}

impl Iterator for U256Range {
    type Item = U256;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let next = self.current;
            self.current += U256::from(1); // 递增
            Some(next)
        } else {
            None
        }
    }
}
