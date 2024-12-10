#[macro_use]
extern crate rocket;

pub mod WatermarkService;

pub mod IPFSImageStorage;

pub mod Transaction;

pub mod DataBase;
pub mod Error;

pub mod UserInfo;

pub mod Router;

pub mod IdentityAuthentication;

use crate::IPFSImageStorage::storeImage::get_cid;
use crate::Transaction::sendTx::Handler;
use crate::Transaction::ContractMethod::ContractMethod;
use alloy::primitives::{Address, U256};
use std::str::FromStr;

pub async fn run() {
    // 为图像添加数字水印，并将带有水印的图片进行存储，获取数字水印
    // watermarkservice::execute_watermark();
    println!("excute down!");

    // 将带有数字水印的图片存放到ipfs中，获取cid
    let token_uri = get_cid();

    // 根据数字水印以及cid以及图片信息构建
    let _tokenURIs = vec![token_uri];
    let to: Address = "0x6d0d470a22c15a14817c51116932312a00ff00c8"
        .parse()
        .unwrap(); // 替换为实际的接收者地址
    let quantity = U256::from(1); // 一次 mint 一个
    let watermarks = vec!["Gzhu".to_string()];
    let capture_times = vec![U256::from(10)];
    let capture_devices = vec!["huawei_mate60".to_string()];
    let capture_companies = vec!["huawei".to_string()];
    let submission_times = vec![U256::from(20)];
    let submission_receivers = vec!["xiaomi".to_string()];
    let func_call = ContractMethod::safeMint {
        to,
        quantity,
        _tokenURIs,
        watermarks: Some(watermarks),
        captureTimes: Some(capture_times),
        captureDevices: Some(capture_devices),
        captureCompanies: Some(capture_companies),
        submissionTimes: Some(submission_times),
        submissionReceivers: Some(submission_receivers),
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
    let result = handler.match_func(func_call).await.unwrap();
    println!("result: {:?}", result);
}
//
#[tokio::test]
pub async fn test_run() {
    run().await
}
