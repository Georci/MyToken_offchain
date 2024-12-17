use log::{log, LevelFilter};
use rbatis::crud;
use rbatis::dark_std::defer;
use rbatis::executor::RBatisTxExecutor;
use rbatis::rbdc::datetime::DateTime;
use rbatis::table_sync::SqliteTableMapper;
use rbatis::{impl_insert, impl_select, Error, RBatis};
use rbdc_mysql::driver::MysqlDriver;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::result;
use tokio::sync::OnceCell;
// table1: users
/**
1.ID
2.company_name
3.username
4.password
5.watermark_base64
6.address
7.privatekey
*/
// table2: images
/**
1.ID
2.cid
3.user_id
*/
static RB: OnceCell<RBatis> = OnceCell::const_new();

// 表users
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Users {
    pub id: Option<i32>,                  // 对应数据库中的 id 列
    pub company_name: Option<String>,     // 对应数据库中的 company_name 列
    pub username: Option<String>,         // 对应数据库中的 username 列
    pub password: Option<String>,         // 对应数据库中的 password 列
    pub watermark_base64: Option<String>, // 对应数据库中的 watermark_base64 列
    pub address: Option<String>,          // 对应数据库中的 address 列
    pub privatekey: Option<String>,       // 对应数据库中的 privatekey 列
}
crud!(Users {});
impl_select!(Users{select_by_username(username:&str) -> Option => "`where username = #{username} limit 1`"});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Images {
    pub id: Option<i32>,
    pub cid: Option<String>,
    pub user_id: Option<i32>,
}
crud!(Images {});

/// 确保数据库连接池已初始化（懒加载）
async fn ensure_db_initialized() -> Result<&'static RBatis, Error> {
    if RB.get().is_none() {
        let rb = RBatis::new();
        rb.link(
            MysqlDriver {},
            "mysql://root:123ABCd!@localhost:3306/ImageInfo",
        )
        .await?;
        RB.set(rb).unwrap();
        println!("Database connection pool initialized.");
    }
    Ok(RB.get().unwrap())
}

/// 获取数据库连接池（隐式初始化）
pub async fn get_db() -> &'static RBatis {
    ensure_db_initialized()
        .await
        .expect("Failed to initialize database")
}

#[tokio::test]
async fn test_get_db() -> Result<(), Error> {
    let rb = get_db().await;
    // 查询 `users` 表中的所有数据
    let sql = "SELECT * FROM users";
    let data = rb.query(sql, vec![]).await?;

    // 打印数据
    println!("Data in table `users`:");
    for row in data {
        println!("{:?}", row);
    }
    Ok(())
}

#[tokio::test]
pub async fn test_get_table_info() -> Result<(), Error> {
    let rb = RBatis::new();
    // 初始化数据库连接
    rb.link(
        MysqlDriver {},
        "mysql://root:123ABCd!@localhost:3306/ImageInfo",
    )
    .await
    .expect("connect failed!");
    println!("connnect database successful");

    // 查询 `users` 表中的所有数据
    let sql = "SELECT * FROM users";
    let data = rb.query(sql, vec![]).await?;

    // 打印数据
    println!("Data in table `users`:");
    for row in data {
        println!("{:?}", row);
    }

    Ok(())
}
