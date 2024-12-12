// /**
//     在本系统中，Rust服务端需要完成以下几个任务：
//         1.接收从前端传入的数据(图片、图片信息)
//             (1).前端传过来的信息(图片、图片信息)都是以什么样的格式传过来
//             (2).对于传过来的信息，我们需要在本地进行一下保存？
//             (3).如果是先将图片保存至本地的话，可以直接将图片的本地位置传递给python程序进行读取，否则要以什么样的进行将图片从服务端传递给数字水印脚本？

//         存：
//         2.将传入的图片应用数字水印技术 -> 保留数字水印用于验证身份
//         3.将图片保存至ipfs中 -> 获得每个图片的cid
//         4.将图片的cid以及图片信息上传至链上，我们以cid为键，数字水印、图片信息等为值存储在mapping中，然后对于每一张图片，我们返还一个cid给user

//         取：
//         5.user可以根据cid从链上获取数字水印以及图片信息
//         6.user可以使用cid从ipfs中获取带有数字水印的图片
//         7.user可以使用数字水印从系统中获取不带数字水印的原图
// */
// //! This example demonstrates how to interact with a contract that is already deployed onchain using
// //! the `ContractInstance` interface.
use BlockchainImageService::DataBase::*;
use BlockchainImageService::Router::Image_routers::*;
use BlockchainImageService::Router::User_routers::*;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    // let rb = get_db().await;
    rocket::build().mount("/", routes![login, register, upload_image])
    // .manage(rb)
}
