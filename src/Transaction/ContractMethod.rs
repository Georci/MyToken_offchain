use crate::Transaction::sendTx::IERC721A::{ImageInfo, SaleInfo};
use alloy::primitives::aliases::TxHash;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use serde::{Deserialize, Serialize};

///当前文件内容旨在静态指定合约中所有可通过外部调用的函数以及这些函数可能的返回值类型

// include all functions can be called
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContractMethod {
    /// view
    name,
    /// view
    symbol,
    /// view
    totalSupply,
    /// view
    balanceOf {
        owner: Address,
    },
    /// view
    ownerOf {
        tokenId: U256,
    },
    /// view
    isApprovedForAll {
        owner: Address,
        operator: Address,
    },
    /// view
    supportsInterface {
        interfaceId: FixedBytes<4>,
    },
    /// view
    tokenURI {
        tokenId: U256,
    },
    /// view
    _imageInfo {
        tokenId: U256,
    },
    /// view
    _imageSaleHistory {
        tokenId: U256,
        index: U256,
    },
    safeTransferFrom {
        from: Address,
        to: Address,
        tokenId: U256,
        amount: Option<Bytes>,
    },
    safeTransferFromWithValue {
        from: Address,
        to: Address,
        tokenId: U256,
        value: U256,
    },
    transferFrom {
        from: Address,
        to: Address,
        tokenId: U256,
    },
    approve {
        to: Address,
        tokenId: U256,
    },
    getApproved {
        tokenId: U256,
    },
    setApprovalForAll {
        operator: Address,
        approved: bool,
    },
    modifyImageInfo {
        tokenId: U256,
        _tokenURIs: String,
        owner: Address,
        watermark: String,
        captureTime: U256,
        captureDevice: String,
        captureCompany: String,
        submissionTime: U256,
        submissionReceiver: String,
    },
    modifyCaptureInfo {
        tokenId: U256,
        captureTime: U256,
        captureDevice: String,
        captureCompany: String,
    },
    safeMint {
        to: Address,
        quantity: U256,
        _tokenURIs: Vec<String>,
        watermarks: Option<Vec<String>>,
        captureTimes: Option<Vec<U256>>,
        captureDevices: Option<Vec<String>>,
        captureCompanies: Option<Vec<String>>,
        submissionTimes: Option<Vec<U256>>,
        submissionReceivers: Option<Vec<String>>,
    },
    safeBatchTransferFrom {
        by: Address,
        from: Address,
        to: Address,
        tokenIds: Vec<U256>,
        data: Bytes,
    },
    batchTransferFrom {
        from: Address,
        to: Address,
        tokenIds: Vec<U256>,
    },
    burn {
        tokenId: U256,
    },
    batchBurn {
        tokenIds: Vec<U256>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContractMethodResult {
    Defalut,
    U256(U256),
    Bool(bool),
    // 其他返回类型
    Bytes4(FixedBytes<4>),
    String(String),
    Address(Address),
    ImageInfo(ImageInfo),
    SaleInfo(SaleInfo),
    TxHash(TxHash),
    Error,
}
