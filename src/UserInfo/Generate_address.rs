use alloy::hex;
use alloy::primitives::{keccak256, Address};
use k256::ecdsa::SigningKey;
use k256::elliptic_curve::SecretKey;
use rand::rngs::OsRng;

/// 随机生成以太坊账户并返回私钥和地址
pub fn generate_random_account() -> ([u8; 32], Address) {
    // 生成一个新的随机私钥
    let signing_key = SigningKey::random(&mut OsRng);

    // 从私钥获取公钥
    let verifying_key = signing_key.verifying_key();

    // 获取未压缩的公钥字节（去除前缀字节）
    // let public_key_bytes = &verifying_key.to_encoded_point(false).as_bytes()[1..];
    let binding = verifying_key.to_encoded_point(false);
    let public_key_bytes = &binding.as_bytes()[1..];
    // 计算公钥的 Keccak256 哈希
    let hash = keccak256(public_key_bytes);

    // 以太坊地址是哈希值的最后 20 个字节
    let address_bytes = &hash[12..];

    // 创建一个 Address 对象
    let address = Address::from_slice(address_bytes);

    // 打印以太坊地址
    println!("address: 0x{}", hex::encode(address));

    // 可选：打印私钥的十六进制表示
    let private_key_bytes = signing_key.to_bytes();
    println!("private key: 0x{}", hex::encode(private_key_bytes));
    (private_key_bytes.into(), address)
}

#[test]
fn test_generate_random_account() {
    // 1. 生成随机账户
    let (private_key, address) = generate_random_account();
    println!("generate ethereum address: {}", address);
    println!(
        "generate ethereum privateKey: 0x{}",
        hex::encode(private_key)
    );
}
