// misc.rs
// 杂项函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称						参数								返回值
----    --------    ----						----								------
69		pub			sql_inline					Vec<T>, String, Option<usize>		Option<Vec<T>>
113		pub			noback_sql_inline			String								()
145		pub			standard_sql				Vec<T>, String, Option<usize>		Option<Vec<T>>
197		pub			generate_keys				/									Key

#==============================#
	本模块定义的宏有：
行号	是否公有	名称			可接受参数数量
----    --------    ----			--------------
127		export		sql_inline		1或4

*/

use actix_web::cookie::Key;
use mysql::{PooledConn, Row, Pool};
use mysql::prelude::Queryable;
use sha2::{Digest, Sha256};
use crate::ear_v3_config::{KEY_CODE, SQL_URL};

// 数据库连接函数 ############################################

// user_info
/*
	id				INT
	name			TEXT
	set_time		INT
	passwd			VARCHAR
	email			VARCHAR
	phone			CHAR

*/

// book_info
/*
	id				INT
	name			TEXT
	book_images		BLOB
	author			TEXT
	publish			TEXT
	public_date		TEXT
	introduction	TEXT

*/

/*
	新一代 标准 数据库连接函数
	
	该函数使用内联函数形式，集成了建立连接
	池与结果检查等步骤，并且扩大了解析格式。
	但它依然有发展潜力，检查步骤冗余等问题
	依然限制着网站性能。
	新一代的函数已然是呼之欲出。

	该函数应在热点调用点使用。
*/
#[inline(always)]
pub fn sql_inline<'a, T>(
	mut sql_data: Vec<T>,
	tmp: String,
	exp_len: Option<usize>
) -> Option<Vec<T>>
where
	T: mysql::prelude::FromRow
{	
	println!("    [\x1b[34m/*/\x1b[0m] \x1b[34m 数据库请求语句：{tmp} \x1b[0m");
	let pool: Pool = Pool::new(SQL_URL).expect("建立池失败");
	let mut connect: PooledConn = pool.get_conn().expect("建立链接失败");

	connect.query_iter(&tmp)
		.expect(&format!("SQL查询失败，查询语句:{tmp} 错误："))
		.for_each( |row|
			{
				let tmp1: Row = row.expect("SQL行无法解析：");
				match mysql::from_row_opt(tmp1)
				{
					Ok(t)	=> sql_data.push(t),
					Err(e)	=> println!("    [\x1b[31m/X/\x1b[0m] \x1b[34m sql_inline SQL处理失败：{e}\x1b[0m"),
				}
			});

	println!("[\x1b[1;32m/+/\x1b[0m] \x1b[1;34m sql_inline 成功返回。\n\x1b[0m");

	/*
	if (exp_len.is_some()
		&& Some(sql_data.len()) != exp_len)
	|| sql_data.len() == 0
	*/
	if exp_len.is_some()
	&& Some(sql_data.len()) != exp_len
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m SQL语句{tmp}结果检查失败。\x1b[0m");
		None
	}
	else
	{
		Some(sql_data)
	}
}

// 无返回值的 数据库连接函数
pub fn noback_sql_inline<'a>(
	tmp: String,
) -> ()
{
	println!("    [\x1b[34m/*/\x1b[0m] \x1b[34m 数据库请求语句：{tmp} \x1b[0m");
	let pool: Pool = Pool::new(SQL_URL).expect("建立池失败");
	let mut connect: PooledConn = pool.get_conn().expect("建立链接失败");

	let _ = connect.query_iter(&tmp).expect(&format!("SQL查询失败，查询语句:{tmp} 错误："));

	println!("[\x1b[1;32m/+/\x1b[0m] \x1b[1;34m sql_inline 成功返回。\n\x1b[0m");
}

#[macro_export]
macro_rules! sql_inline
{
	($T:ty,
	$tmp1: expr,
	$tmp2: expr,
	$tmp3: expr) =>
	{
		sql_inline::<$T>($tmp1, $tmp2, $tmp3)
	};

	($tmp1: expr) =>
	{
		noback_sql_inline($tmp1)
	};
}

// 该函数应在冷点调用点使用。
#[inline(never)]
pub fn standard_sql<'a, T>(
	sql_data: Vec<T>,
	tmp: String,
	exp_len: Option<usize>
) -> Option<Vec<T>>
where
	T: mysql::prelude::FromRow
{
	sql_inline(sql_data, tmp, exp_len)
}

//############################################

/*
	“虽然 WebSockets 提供了强大的实时通信功能，
	但谨慎管理服务器资源并实施强大的安全措施以
	确保应用程序的可扩展性和安全性非常重要。通
	过解决这些问题，您可以有效地将 WebSockets
	集成到 Actix-web 应用程序中以增强用户体验。”

	该模块应配合SSL使用，目前依然在开发中。

	该模块完成后应具备以下功能：
	·当前会话过期时即刻进行消息推送
	·良好的安全性
	·良好的性能

	目前的思路包括如：
		session.continuation(Item::FirstText("Hello".into())).await
	进行session延续，并将其存储于服务器状态中，
	当会话过期时对其进行警告，该逻辑应主要在
	set_cookie函数中实现。但即便如此，想要实现
	该功能依然有很高难度，至少我们需要在目前所
	有的Access网页中加入客户端js。
*/
// 后端推送信息函数
/*
async fn session_expiration_ws(
	req: HttpRequest,
	stream: web::Payload
) -> actix_web::Result<impl Responder>
{
	let (res, mut session, _) = actix_ws::handle(&req, stream)?;

	if session.text("You have logged out.").await.is_err() {
		let _ = session.close(None).await;
	}

	Ok(res)
}
*/

pub fn generate_keys() -> Key
{
	let current_time: String = chrono::Utc::now().to_string();
	let mut hasher1: Sha256 = Sha256::new();
	let mut hasher2: Sha256 = Sha256::new();

	hasher1.update(current_time + KEY_CODE);
	let result1 = hasher1.finalize();
	hasher2.update(&result1);
	let result2 = hasher2.finalize();

	Key::from(format!("{:X}{:X}", result1, result2).as_bytes())
}