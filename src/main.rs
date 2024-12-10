#[macro_use]
extern crate rocket;

// mod routes; // 引入路由管理模块

#[launch]
fn rocket() -> _ {
    // 初始化 Rocket 实例，并加载路由
    rocket::build().mount("/", routes![index]) // 保留现有的 index 路由
                                               // .attach(routes::configure_routes()) // 挂载其他路由
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
