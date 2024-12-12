// account: 0x6d0d470a22c15a14817c51116932312a00ff00c8
// pk: 0x3ba5c6a17da00c75e9377e03ae98aa3dcdca7c4e537c84399125dfefa89be521
// 部署合约
/**
图片信息存储在ipfs，还是区块链？
// cid仅仅只是图片的一个标识，任何人都可以通过该标识访问nft图片，真的所有权还是通过合约控制的 ——如果是这样的话，那还是直接将图片id作为索引就好了
@notice: 图片信息存放在ipfs上与存放在区块链上的区别
1.图片信息还是得存放在区块链上，因为如果区块链上发生交易的话，这些信息其实是需要发生改变的，而且可能也需要第一时间进行索引
2.所以nft所有权，可能需要采取这样的方案：如果用户有输入地址，则直接将地址作为owner，否则随机生成一个地址作为nft owner，程序将地址和cid同时返回给用户
*/
use crate::Error::TxError;
use crate::Transaction::ContractMethod::{ContractMethod, ContractMethodResult};
use alloy::network::NetworkWallet;
use alloy::providers::fillers::{FillProvider, JoinFill, WalletFiller};
use alloy::providers::{Identity, PendingTransactionBuilder, ReqwestProvider, WalletProvider};
use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{address, hex, Address, U256},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::{SolCall, SolStruct, SolType, SolValue},
    transports::http::{Client, Http},
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value::Null;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;
use std::{convert::TryFrom, process, sync::Arc};

sol!(
    #[allow(missing_docs)]
    #[derive(Debug, Serialize, Deserialize)]
    #[sol(rpc)]
    ImageToken,
    "./ImageToken_sol_MyToken.json"
);
use crate::Transaction::sendTx::ImageToken::{totalSupplyReturn, ImageTokenInstance};
#[derive(Debug, Clone)]
pub struct Handler {
    http_url: String,
    wallet_pk: String,
    contract_address: Address,
    user_address: Address,
    contract: Option<ImageTokenInstance<Http<Client>, ReqwestProvider>>,
}
impl Handler {
    pub fn new() -> Self {
        Self {
            http_url: "".to_string(),
            wallet_pk: "".to_string(),
            contract_address: Default::default(),
            user_address: Default::default(),
            contract: None,
        }
    }
    pub async fn initialize_contract(
        http_url: &str,
        wallet_pk: &str,
        user_address: &str,
        contract_address: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. 获取 provider
        let rpc_url = http_url.parse()?;
        let signer: PrivateKeySigner = wallet_pk.parse()?;
        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new().on_http(rpc_url);

        let wallet_address =
            <EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(&wallet);

        if Address::from_str(user_address).unwrap() != wallet_address {
            println!("privateKey and user_address are not matched!");
            process::exit(1)
        }

        // 验证连接
        let latest_block = provider.get_block_number().await?;
        println!("Connected to blockchain. Latest block: {}", latest_block);

        // 2. 格式化合约地址
        let address = Address::parse_checksummed(contract_address, None).expect("valid checksum");

        // 3. 生成合约实例
        let contract: ImageTokenInstance<Http<Client>, ReqwestProvider> =
            ImageToken::new(address, provider);

        Ok(Self {
            http_url: http_url.to_string(),
            wallet_pk: wallet_pk.to_string(),
            contract_address: Address::from_str(contract_address).unwrap(),
            user_address: Address::from_str(user_address).unwrap(),
            contract: Some(contract),
        })
    }

    // 默认调用safeMint函数，上传图片信息
    pub async fn upload_info(
        &self,
        func: ContractMethod,
    ) -> Result<ContractMethodResult, Box<dyn std::error::Error>> {
        let mut contract_method_result: ContractMethodResult = ContractMethodResult::Defalut;

        let uer_address = self.user_address.clone();

        if let Some(contract) = &self.contract {
            match func {
                ContractMethod::safeMint {
                    to,
                    quantity,
                    _tokenURIs,
                    watermarks,
                    captureTimes,
                    captureDevices,
                    captureCompanies,
                    submissionTimes,
                    submissionReceivers,
                } => {
                    // 根据传入的参数调用合约方法
                    let tx_builder = if watermarks.is_none()
                        && captureTimes.is_none()
                        && captureDevices.is_none()
                        && captureCompanies.is_none()
                        && submissionTimes.is_none()
                        && submissionReceivers.is_none()
                    {
                        let builder = contract
                            .safeMint_1(to, quantity, _tokenURIs)
                            .from(uer_address);
                        let result = builder.send().await?;
                        result
                    } else {
                        let builder = contract
                            .safeMint_0(
                                to,
                                quantity,
                                _tokenURIs,
                                watermarks.unwrap(),
                                captureTimes.unwrap(),
                                captureDevices.unwrap(),
                                captureCompanies.unwrap(),
                                submissionTimes.unwrap(),
                                submissionReceivers.unwrap(),
                            )
                            .from(uer_address);
                        let result = builder.send().await?;
                        result
                    };
                    let hash = tx_builder.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                _ => {
                    println!("This function only call safeMint");
                }
            }
        }
        Ok(contract_method_result)
    }

    pub async fn match_func(
        &self,
        func: ContractMethod,
    ) -> Result<ContractMethodResult, Box<dyn std::error::Error>> {
        let mut contract_method_result: ContractMethodResult = ContractMethodResult::Defalut;
        match self.contract.clone() {
            None => {
                println!("please intialize contract first")
            }
            Some(contract) => match func {
                ContractMethod::name => {
                    let result = contract.name().call().await?._0;
                    contract_method_result = ContractMethodResult::String(result);
                }
                ContractMethod::symbol => {
                    let result = contract.symbol().call().await?._0;
                    contract_method_result = ContractMethodResult::String(result);
                }
                ContractMethod::totalSupply => {
                    let result = contract.totalSupply().call().await?.result;
                    contract_method_result = ContractMethodResult::U256(result);
                    println!("contract_method_result is {:?}", contract_method_result);
                }
                ContractMethod::balanceOf { owner } => {
                    let result = contract.balanceOf(owner).call().await?._0;
                    contract_method_result = ContractMethodResult::U256(result);
                }
                ContractMethod::ownerOf { tokenId } => {
                    let result = contract.ownerOf(tokenId).call().await?._0;
                    contract_method_result = ContractMethodResult::Address(result);
                }
                ContractMethod::isApprovedForAll { owner, operator } => {
                    let result = contract.isApprovedForAll(owner, operator).call().await?._0;
                    contract_method_result = ContractMethodResult::Bool(result);
                }
                ContractMethod::supportsInterface { interfaceId } => {
                    let result = contract.supportsInterface(interfaceId).call().await?._0;
                    contract_method_result = ContractMethodResult::Bool(result);
                }
                ContractMethod::tokenURI { tokenId } => {
                    let result = contract.tokenURI(tokenId).call().await?._0;
                    contract_method_result = ContractMethodResult::String(result);
                }
                ContractMethod::_imageInfo { tokenId } => {
                    let result = contract._imageInfo(tokenId).call().await?.imageInfo;
                    contract_method_result = ContractMethodResult::ImageInfo(result);
                }
                ContractMethod::_imageSaleHistory { tokenId, index } => {
                    let result = contract
                        ._imageSaleHistory(tokenId, index)
                        .call()
                        .await?
                        .saleInfo;
                    contract_method_result = ContractMethodResult::SaleInfo(result);
                }
                ContractMethod::safeTransferFrom {
                    from,
                    to,
                    tokenId,
                    amount,
                } => {
                    let result: PendingTransactionBuilder<Http<Client>, Ethereum> =
                        if amount.is_none() {
                            contract
                                .safeTransferFrom_0(from, to, tokenId)
                                .send()
                                .await?
                        } else {
                            contract
                                .safeTransferFrom_1(from, to, tokenId, amount.unwrap())
                                .send()
                                .await?
                        };
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::safeTransferFromWithValue {
                    from,
                    to,
                    tokenId,
                    value,
                } => {
                    let result = contract
                        .safeTransferFromWithValue(from, to, tokenId, value)
                        .send()
                        .await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::transferFrom { from, to, tokenId } => {
                    let result = contract.transferFrom(from, to, tokenId).send().await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::approve { to, tokenId } => {
                    let result = contract.approve(to, tokenId).send().await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::getApproved { tokenId } => {
                    let result = contract.getApproved(tokenId).call().await?._0;
                    contract_method_result = ContractMethodResult::Address(result);
                }
                ContractMethod::setApprovalForAll { operator, approved } => {
                    let result = contract
                        .setApprovalForAll(operator, approved)
                        .send()
                        .await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::modifyImageInfo {
                    tokenId,
                    _tokenURIs,
                    owner,
                    watermark,
                    captureTime,
                    captureDevice,
                    captureCompany,
                    submissionTime,
                    submissionReceiver,
                } => {
                    let result = contract
                        .modifyImageInfo(
                            tokenId,
                            _tokenURIs,
                            owner,
                            watermark,
                            captureTime,
                            captureDevice,
                            captureCompany,
                            submissionTime,
                            submissionReceiver,
                        )
                        .send()
                        .await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::modifyCaptureInfo {
                    tokenId,
                    captureTime,
                    captureDevice,
                    captureCompany,
                } => {
                    let result = contract
                        .modifyCaptureInfo(tokenId, captureTime, captureDevice, captureCompany)
                        .send()
                        .await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::safeMint {
                    to,
                    quantity,
                    _tokenURIs,
                    watermarks,
                    captureTimes,
                    captureDevices,
                    captureCompanies,
                    submissionTimes,
                    submissionReceivers,
                } => {
                    let result = if watermarks.is_none() {
                        contract.safeMint_1(to, quantity, _tokenURIs).send().await?
                    } else {
                        contract
                            .safeMint_0(
                                to,
                                quantity,
                                _tokenURIs,
                                watermarks.unwrap(),
                                captureTimes.unwrap(),
                                captureDevices.unwrap(),
                                captureCompanies.unwrap(),
                                submissionTimes.unwrap(),
                                submissionReceivers.unwrap(),
                            )
                            .send()
                            .await?
                    };
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::safeBatchTransferFrom {
                    by,
                    from,
                    to,
                    tokenIds,
                    data,
                } => {
                    let result = contract
                        .safeBatchTransferFrom(by, from, to, tokenIds, data)
                        .send()
                        .await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::batchTransferFrom { from, to, tokenIds } => {
                    let result = contract
                        .batchTransferFrom(from, to, tokenIds)
                        .send()
                        .await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::burn { tokenId } => {
                    let result = contract.burn(tokenId).send().await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
                ContractMethod::batchBurn { tokenIds } => {
                    let result = contract.batchBurn(tokenIds).send().await?;
                    let hash = result.watch().await?;
                    contract_method_result = ContractMethodResult::TxHash(hash);
                }
            },
        }
        Ok(contract_method_result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safeMint() {
        let http_url = "http://localhost:8545";
        let pk = "0x3ba5c6a17da00c75e9377e03ae98aa3dcdca7c4e537c84399125dfefa89be521";
        let user_address = "0x6d0d470a22c15a14817c51116932312a00ff00c8";
        let contract_address = "0xa6a0110367e24c541FC29124E8E89E3556263177";
        let handler = Handler::initialize_contract(http_url, pk, user_address, contract_address)
            .await
            .unwrap();

        // let result = handler
        //     .match_func(ContractMethod::_imageInfo {
        //         tokenId: U256::from(0),
        //     })
        //     .await;
        // println!("result is {:?}", result);

        let _tokenURIs = vec!["QmcbD1QEKKWkQQumdrhVUBHgdkPyU5GYzwMk5PkoRzQiP7".to_string()];
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

        let result = handler.match_func(func_call).await.unwrap();
        println!("result: {:?}", result);
    }

        #[tokio::test]
        async fn test_getTotalToken() {
            let http_url = "http://localhost:8545";
            let pk = "0x3ba5c6a17da00c75e9377e03ae98aa3dcdca7c4e537c84399125dfefa89be521";
            let user_address = "0x6d0d470a22c15a14817c51116932312a00ff00c8";
            let contract_address = "0xa6a0110367e24c541FC29124E8E89E3556263177";
            let handler = Handler::initialize_contract(http_url, pk, user_address, contract_address)
                .await
                .unwrap();

            let func_call = ContractMethod::totalSupply;
            let total_supply = handler
                .match_func(func_call)
                .await
                .unwrap();
            println!("total_supply is {:?}", total_supply);
        }

    //     #[tokio::test]
    //     async fn test_getImageInfo() {
    //         let http_url = "http://localhost:8545";
    //         let pk = "0x3ba5c6a17da00c75e9377e03ae98aa3dcdca7c4e537c84399125dfefa89be521";
    //         let abi_path = "./ImageToken_sol_MyToken.abi";
    //         let contract_address = "0xa6a0110367e24c541FC29124E8E89E3556263177";
    //         let handler = Handler::initialize_contract(http_url, pk, abi_path, contract_address)
    //             .await
    //             .unwrap();

    //         let call_params = (U256::from(6),);

    //         println!("wuxizhi1");
    //         let image_info: ImageInfo = handler
    //             .query_function("_imageInfo", call_params)
    //             .await
    //             .unwrap();
    //         println!("image_info is {:?}", image_info);
    //     }
}
