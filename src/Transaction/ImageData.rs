use crate::Transaction::sendTx::IERC721A::{ImageInfo, SaleInfo};
use alloy::primitives::aliases::TxHash;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use std::fmt;

/// 当前文件旨在实现图片信息快速构造等一系列方法
impl SaleInfo {
    pub fn new(saleTime: U256, buyer: Address, saleValue: U256) -> Self {
        Self {
            saleTime,
            buyer,
            saleValue,
        }
    }

    pub fn default() -> Self {
        Self {
            saleTime: U256::from(0),
            buyer: Address::ZERO,
            saleValue: U256::from(0),
        }
    }
}
impl fmt::Display for SaleInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "------------------------------------")?;
        write!(
            f,
            "\n saleTime: {}\n buyer: {}\n saleValue: {}",
            self.saleTime, self.buyer, self.saleValue
        )?;
        write!(f, "\n------------------------------------")?;
        Ok(())
    }
}

impl ImageInfo {
    pub fn new(
        cid: String,
        owner: Address,
        watermark: String,
        capture_time: U256,
        capture_device: String,
        capture_company: String,
        submission_time: U256,
        submission_receiver: String,
        sale_history: Vec<SaleInfo>,
    ) -> Self {
        Self {
            _tokenURIs: cid,
            owner,
            watermark,
            captureTime: capture_time,
            captureDevice: capture_device,
            captureCompany: capture_company,
            submissionTime: submission_time,
            submissionReceiver: submission_receiver,
            saleHistory: sale_history,
        }
    }

    pub fn default() -> Self {
        Self {
            _tokenURIs: "".to_string(),
            owner: Default::default(),
            watermark: "".to_string(),
            captureTime: Default::default(),
            captureDevice: "".to_string(),
            captureCompany: "".to_string(),
            submissionTime: Default::default(),
            submissionReceiver: "".to_string(),
            saleHistory: vec![],
        }
    }
}

impl fmt::Display for ImageInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "=================================")?;
        write!(
            f,
            "\n cid: {}\n owner: {}\n watermark: {}\n captureTime: {}\n captureDevice: {}\n captureCompany: {}\n submissionTime: {}\n submissionReceiver: {}",
            self._tokenURIs, self.owner, self.watermark, self.captureTime, self.captureDevice, self.captureCompany, self.submissionTime, self.submissionReceiver
        )?;

        let sale_history = self.saleHistory.iter();
        for v in sale_history {
            write!(f, "\n saleInfo: {}", v);
        }
        write!(f, "\n=================================")?;
        Ok(())
    }
}
