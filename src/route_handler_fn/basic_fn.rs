// basic_fn.rs
// 本文件包含基础路由处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称						参数								返回值
----    --------    ----						----								----
80		pub			no_store_http_head			u16, String							HttpResponse
92		private		standard_http_head			u16, String							HttpResponse
104		pub			http_model					u16, String, T, &'a str				HttpResponse

165		pub			redis_set_fn				K, V								()
183		pub			redis_expire_fn				K, i64								()
201		pub			redis_get_fn				K									Option<RV>
226		pub			redis_del_fn				K									()
245		private		redis_set_in				&SessionData02						()
281		pub			redis_update_session_id		&SessionData02, &'a str				bool
322		pub			specify_redis_update		&'a str, Vec<usize>					bool
359		pub			redis_get_history			&'a str								Option<Vec<usize>>
374		pub			redis_get					&'a str								Option<SessionData01>
396		pub			check_login_time			&'a str								bool
444		private		redis_control_model			&SessionData02						()
465		private		redis_session_id_get		&'a str								Option<String>
479		private		session_id_compare			&SessionData02						bool
499		pub			set_cookie					Session, &SessionData02				bool
535		pub			get_cookie					&Session							Option<SessionData02>
558		pub			get_cookie_handler			&Session							bool

573		pub			standard_local_read_file	&'a str								(i32, Box<Vec<u8>>)
602		private		local_read_file				&'a str								(i32, Box<Vec<u8>>)
633		private		http_read_file				String, &'a str						HttpResponse
651		pub			get_closure					&'a str, &'a str					HttpResponse

713		pub			cap_img_gen					Session								HttpResponse

757		private		letter_closure				&'a str								bool
775		private		check_username				&'a str								bool
809		private		check_passwd				&'a str								bool
827		private		check_email					&'a str								bool
859		pub			check_form					&UpdateRequest						bool
877		pub			mismatch_check_form			&UpdateRequest						Option<Vec<String>>

931		pub			filte_option				&'a str								Option<()>
950		pub			check_option				&'a str								Option<String>
960		pub			option_cofirm				String, &'a str						Option<String>

#==============================#
	本模块定义的宏有：
行号	是否公有	名称			可接受参数数量
----    --------    ----			--------------
120		export		http_model		3或4

*/

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use captcha::filters::{Cow, Noise, Wave, Dots};
use captcha::{Captcha, Geometry};
use chrono::{DateTime, Duration, Utc};
use json::object;
use mime_guess::from_path;
use redis::{AsyncCommands, Client, ToRedisArgs, FromRedisValue};
use std::io::Read;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::ear_v3_struct::{SessionData01, SessionData02, UpdateRequest};
use crate::misc::*;

// HTTP 响应函数 ############################################

// 非缓存http包头响应函数
pub fn no_store_http_head(code: u16, type_str: String) -> HttpResponse
{
	HttpResponse::build(StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
		.content_type(type_str + "; charset=utf-8")
		.append_header(("Cross-Origin-Resource-Policy", "same-site"))
		.append_header(("Cache-Control",				"no-store"))
		.append_header(("X-Content-Type-Options",		"nosniff"))
		.append_header(("X-Frame-Options",				"DENY"))
		.finish()
}

// 标准http包体响应函数
fn standard_http_head(code: u16, type_str: String) -> HttpResponse
{
	HttpResponse::build(StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
		.content_type(type_str + "; charset=utf-8")
		.append_header(("Cross-Origin-Resource-Policy", "same-site"))
		.append_header(("Cache-Control",				"private, must-revalidate, max-age=259200"))
		.append_header(("X-Content-Type-Options",		"nosniff"))
		.append_header(("X-Frame-Options",				"DENY"))
		.finish()
}

// ## 集成 标准http响应函数
pub fn http_model<'a, T: actix_web::body::MessageBody + 'static>(code: u16, type_str: String, tmp: T, mode: &'a str) -> HttpResponse
{
	match mode
	{
		"standard" => standard_http_head(code, type_str),
		"no_store" => no_store_http_head(code, type_str),
		&_ => {
			println!("未知响应模式。");
			HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
		}
	}
	.set_body(tmp.boxed())
}

// http_model 可变参数宏
#[macro_export]
macro_rules! http_model
{
	($tmp1: expr,
	$tmp2: expr,
	$tmp3: expr,
	$tmp4: expr) =>
	{
		http_model($tmp1, $tmp2, $tmp3, $tmp4)
	};
	($tmp1: expr,
	$tmp2: expr,
	$tmp3: expr) =>
	{
		http_model(200, $tmp1, $tmp2, $tmp3)
	};
}

// Cookie 处理函数 ############################################
// 在不使用await调用时，异步函数默认惰性

// const TIME_STAY: &'static str = r#"User_requests_are_too_frequent_(Less_than_5_minutes)"#;

/*
	“actix_session”组件中Session的
	存储机制

	实验表明，actix_session会将将随
	机密文作为 Session 密钥存储在用
	户 cookie 中，而 Session 数据存
	储在 Redis 中，并以另一段密文为
	字段名 KEY_NAME ，该密钥用作访问
	Session 数据的凭证。该实验还指出，
	从 Redis 中删除 Session 数据会导
	致服务器不再识别用户 cookie，即
	使 cookie 完好无损。
*/

/*
	由于actix_session在Redis中存储的
	字段名是随机密文，是次版本采用二
	级存储方式存储Cookie数据并实现单
	会话和跨设备登录。
*/

// Redis SET 函数
pub async fn redis_set_fn<'a, K: ToRedisArgs + Send + Sync + 'a, V: ToRedisArgs + Send + Sync + 'a>(
	key: K,
	value: V,
) -> ()
{
	let mut redis_connect = Client::open("") // 你的 Redis 服务器地址
		.expect("打开Redis失败：")
		.get_multiplexed_async_connection()
		.await
		.expect("获取Redis链接失败：");

	let _: () = redis_connect
		.set(key, value)
		.await
		.expect("Redis SET失败：");
}

// Redis EXPIRE 函数
pub async fn redis_expire_fn<'a, K: ToRedisArgs + Send + Sync + 'a>(
	key: K,
	seconds: i64,
) -> ()
{
	let mut redis_connect = Client::open("") // 你的 Redis 服务器地址
		.expect("打开Redis失败：")
		.get_multiplexed_async_connection()
		.await
		.expect("获取Redis链接失败：");

	let _: () = redis_connect
		.expire(key, seconds)
		.await
		.expect("Redis EXPIRE失败：");
}

// Redis GET 函数
pub async fn redis_get_fn<'a, K: ToRedisArgs + Send + Sync + 'a, RV>(
	key: K,
) -> Option<RV>
where
	RV: FromRedisValue,
{
	let mut redis_connect = Client::open("") // 你的 Redis 服务器地址
		.expect("打开Redis失败：")
		.get_multiplexed_async_connection()
		.await
		.expect("获取Redis链接失败：");

	match redis_connect
		.get::<K, RV>(key)
		.await
	{
		Ok(t) => Some(t),
		Err(e) => {
			println!("Redis GET失败：{e}");
			None
		},
	}
}

// Redis DEL 函数
pub async fn redis_del_fn<'a, K: ToRedisArgs + Send + Sync + 'a>(
	key: K,
) -> ()
{
	let mut redis_connect = Client::open("") // 你的 Redis 服务器地址
		.expect("打开Redis失败：")
		.get_multiplexed_async_connection()
		.await
		.expect("获取Redis链接失败：");

	let _: () = redis_connect
		.del(key)
		.await
		.expect("Redis DEL失败：");
}

// 一级存储
// Redis初始化核心
// 三代集成
async fn redis_set_in(data: &SessionData02) -> ()
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m 初始化Redis数据。\x1b[0m");

	let tmp2: (usize, String) = if let Some(tmp) = sql_inline!((usize, String),
		Vec::new(),
		format!("select user_id, email from user_info where id = {}", data.id),
		Some(1)
	)
	{
		tmp[0].clone()
	}
	else
	{
		println!("[\x1b[1;31mX\x1b[0m] \x1b[1;34m 未查询到用户SQL信息。\x1b[0m");
		(0, String::new())
	};

	let session_data = object!{
		id:				data.id.clone(),
		user:			data.user.clone(),
		user_id:		tmp2.0,
		email:			tmp2.1,
		session_id:		data.session_id.clone(),
		last_login:		Utc::now().format("%Y %b %d %H:%M:%S%.3f %z").to_string(),
		history:		Vec::<usize>::with_capacity(16),
	};

	let name: &str = &data.user;
	redis_set_fn(name, session_data.dump()).await;
	redis_expire_fn(name, 3*24*3600).await;

	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m redis_set_in 成功返回。\n\x1b[0m");
}

// 更新 Redis Session_ID
pub async fn redis_update_session_id<'a>(data: &SessionData02, mode: &'a str) -> bool
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m 更新Redis Session_ID。\x1b[0m");

	let session_id: &str = &data.session_id;
	let name: &str = &data.user;

	if let Some(mut tmp) = redis_get(name).await
	{
		match mode
		{
			"update"	=> {
				tmp.session_id = session_id.to_string();
			},
			"reset"		=> {
				if tmp.session_id == session_id
				{
					tmp.session_id = "".to_string();
				}
			},
			_ => {
				println!("[\x1b[31mX\x1b[0m] \x1b[34m redis_update_session_id 模式错误。\x1b[0m");
				return false;
			}
		}

		tmp.last_login = Utc::now().format("%Y %b %d %H:%M:%S%.3f %z").to_string();
		let data_str: String = serde_json::to_string(&tmp).expect("结构体序列化失败01:");
		redis_set_fn(name, data_str).await;

		println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m redis_update_session_id 成功返回。\n\x1b[0m");
		true
	}
	else
	{
		println!("[\x1b[31mX\x1b[0m] \x1b[34m 应调用redis_set_in()函数建立Session数据。\x1b[0m");
		false
	}
}

// 使用指定结构体更新 Redis 数据
pub async fn specify_redis_update<'a>(user: &'a str, data: Vec<usize>) -> bool
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m 更新Redis数据。\x1b[0m");

	if let Some(mut tmp) = redis_get(user).await
	{
		/*
			此处有两种写法，第一种是如
			下面这种，可以使用迭代器对
			其进行分解，然后逐步修改各
			值。

			另一种方法是使用类似于Linux
			权限的识别机制，如1、2，4三
			种权限标志，它们可以不重复
			地组成0~7，即每种权限组合拥
			有唯一识别码。

			经过研究，决定是此版本引入部分SQL数据
		*/
		// 由于目前只有一个可修改属性，故不使用迭代器。
		tmp.history = data;

		let data_str: String = serde_json::to_string(&tmp).expect("结构体序列化失败01:");
		redis_set_fn(user, data_str).await;

		println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m redis_update 成功返回。\n\x1b[0m");
		true
	}
	else
	{
		println!("[\x1b[31mX\x1b[0m] \x1b[34m 提取Redis数据失败。\x1b[0m");
		false
	}
}

// 提取Redis history
pub async fn redis_get_history<'a>(user: &'a str) -> Option<Vec<usize>>
{
	if let Some(tmp) = redis_get(user).await
	{
		Some(tmp.history.clone())
	}
	else
	{
		println!("[\x1b[31mX\x1b[0m] \x1b[34m 提取Redis数据失败。\x1b[0m");
		None
	}
}

// 二级存储
// 将Redis数据读取到结构体中
pub async fn redis_get<'a>(user: &'a str) -> Option<SessionData01>
{
	if let Some(t) = redis_get_fn::<&str, String>(
		user
	).await
	{
		match serde_json::from_str::<SessionData01>(&t)
		{
			Ok(t2) => Some(t2),
			Err(e) => {
				println!("    [\x1b[31mX\x1b[0m] \x1b[34m JSON解析失败：{e}\x1b[0m");
				None
			},
		}
	}
	else
	{
		None
	}
}

// 5分钟内不得连续登录
pub async fn check_login_time<'a>(user: &'a str) -> bool
{
	if let Some(tmp) = redis_get(user).await
	{
		match DateTime::parse_from_str(&tmp
			.last_login,
			"%Y %b %d %H:%M:%S%.3f %z"
		)
		{
			Ok(t3) => if let Some(tmp) = t3
					.checked_add_signed(Duration::minutes(5))
				{
					if tmp < Utc::now()
					{
						true
					}
					else
					{
						println!("    [\x1b[31mX\x1b[0m] \x1b[34m 用户请求过于频繁。\x1b[0m");
						false
					}
				}
				else
				{
					println!("    [\x1b[31mX\x1b[0m] \x1b[34m 时间栈溢出。\x1b[0m");
					false
				},
			Err(e) => {
				println!("UTC时间解析失败：{e}");
				false
			}
		}
	}
	else
	{
		true
	}
}

/*
	该机制最大的亮点就是使
	用用户名去查找 Session
	数据，这在保证数据完整
	性、实现跨设备传输的同
	时，大大提高了安全性，
	减少了性能损耗。
*/
// Redis数据控件
async fn redis_control_model(session_data: &SessionData02) -> ()
{
	if let Some(_) = redis_get(&session_data.user).await
	{
		if redis_update_session_id(session_data, "update").await
		{
			println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m 更新成功。\x1b[0m");
		}
		else
		{
			println!("[\x1b[1;31mX\x1b[0m] \x1b[1;34m 更新失败。\x1b[0m");
		}
	}
	else
	{
		redis_set_in(session_data).await
	}
}

// 读取 Redis Session_ID
#[inline]
async fn redis_session_id_get<'a>(user: &'a str) -> Option<String>
{
	if let Some(t) = redis_get(user).await
	{
		Some(t.session_id.clone())
	}
	else
	{
		None
	}
}

// 比较 Session_ID
#[inline]
async fn session_id_compare(session_data: &SessionData02) -> bool
{
	if let Some(t) = redis_session_id_get(&session_data.user).await
	{
		if t == session_data.session_id
		{
			true
		}
		else
		{
			false
		}
	}
	else
	{
		false
	}
}

// ## 设置Cookie
pub async fn set_cookie(session: Session, data: &SessionData02) -> bool
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m 设置Cookie。\x1b[0m");

	if let Some(t) = session.get::<SessionData02>("data")
			.unwrap_or(None)
	{
		println!("    [\x1b[34m*\x1b[0m] \x1b[34m 已检测到Cookie存在，更新Redis数据。\x1b[0m");
		redis_control_model(&t).await;
		true
	}
	else
	{
		println!("    [\x1b[34m*\x1b[0m] \x1b[34m 未检测到Cookie存在，新建Cookie。\x1b[0m");

		if let Err(e) = session.insert("data",
			data)
		{
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m Cookie设置失败:{e}\x1b[0m");
			false
		}
		else
		{
			redis_control_model(data).await;
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m Cookie设置成功。\x1b[0m");
			println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m set_Cookie 成功返回。\n\x1b[0m");
			true
		}
	}
}

// ## 标准 读取Cookie
/*
	注意。本函数应仅用于提取用户信息，而在身份验证方面，
	本函数已使用更强大的PermyE中间件代替。
*/
pub fn get_cookie(session: &Session) -> Option<SessionData02>
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m 读取Cookie。\x1b[0m");
	match session
		.get::<SessionData02>("data")
	{
		Ok(Some(user_data)) => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 读取到的用户为：{}。\x1b[0m", user_data.user);
			println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m get_Cookie 成功返回。\n\x1b[0m");
			Some(user_data)
		},
		Ok(None) => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m Cookie提取,返回None。\x1b[0m");
			None
		},
		Err(e) => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m Cookie提取失败:{e}\x1b[0m");
			None
		}
	}
}

// ## 校验 Redis
pub async fn get_cookie_handler(session: &Session) -> bool
{
	if let Some(t) = get_cookie(session)
	{
		session_id_compare(&t).await
	}
	else
	{
		false
	}
}

// 本地文件读取函数 ############################################

// 标准 读取本地文件函数
pub fn standard_local_read_file<'a>(file: &'a str) -> (i32, Box<Vec<u8>>)
{
	let file1: &Path = Path::new(file);
	if ! file1.starts_with("./Web02/") {
		return (403, Box::new(Vec::new()));
	}

	let mut data = match std::fs::File::open(&file)
	{
		Ok(t) => t,
		Err(e) => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 打开文件{}失败: {}\x1b[0m", file, e);
			return (404, Box::new(Vec::new()));
		}
	};

	let mut buffer: Vec<u8> = Vec::new();
	match data
		.read_to_end(&mut buffer)
	{
		Ok(_) => (200, Box::new(buffer)),
		Err(e) => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 读取文件{}失败: {}\x1b[0m", file, e);
			(500, Box::new(Vec::new()))
		}
	}
}

// 异步 读取本地文件函数
async fn local_read_file<'a>(file: &'a str) -> (i32, Box<Vec<u8>>)
{
	let file1: &Path = Path::new(file);
	if ! file1.starts_with("./Web02/") {
		return (403, Box::new(Vec::new()));
	}

	let mut data = match File::open(&file).await
	{
		Ok(t) => t,
		Err(e) => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 打开文件{}失败: {}\x1b[0m", file, e);
			return (404, Box::new(Vec::new()));
		}
	};

	let mut buffer: Vec<u8> = Vec::new();
	match data
		.read_to_end(&mut buffer)
		.await
	{
		Ok(_) => (200, Box::new(buffer)),
		Err(e) => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 读取文件{}失败: {}\x1b[0m", file, e);
			(500, Box::new(Vec::new()))
		}
	}
}

// http 读取本地文件函数
#[inline]
async fn http_read_file<'a>(type_str: String, file: &'a str) -> HttpResponse
{
	match local_read_file(file).await
	{
		(200, buffer) => http_model!(type_str, *buffer, "standard"),
		(403, _) => no_store_http_head(403, type_str),
		(404, _) => no_store_http_head(404, type_str),
		_ => no_store_http_head(500, type_str),
	}
}

// 通用响应函数 ############################################

/*
	该函数下版本可能使用actix_files进行改进
*/
// 第二代雾自适应
// ## 通用 界面响应函数
pub async fn get_closure<'a>(tmp: &'a str, tmp2: &'a str) -> HttpResponse
{
	let file: String = format!("{}.{}", tmp, tmp2);

	let file_type: String = from_path(&file)
		.first_or_octet_stream()
		.to_string();
	println!("[\x1b[32m+\x1b[0m] \x1b[34m get_closure 读取{file}，{file_type}类型。\x1b[0m");

	http_read_file(file_type, &file).await
}

// 验证码生成函数 ############################################

/*
	现有的验证码机制是：
	
	...
	// 公共部分
	...
	.route("/pic/cap",							web::get().to(cap_img_gen))
	...

	即，将验证码至于公共路由下，可以进行任意访问。
	由于注册页面也需使用该路由，故这是最好的方法
	了。

	-----

	该机制有很多缺点。首先，所有用户共用一个验证
	码生成函数。其次，验证码存储于服务器状态中，
	这些都会导致内容不受控变更，并且验证困难。

	一个想法是对单用户使用单验证码，并将用户名与
	验证码对应存储到服务器状态或Redis中。

	由于是此版本已实现了单会话和跨设备数据转移，
	上述方法可行。

	一些不成熟的想法包括，将验证码生成函数内置进
	处理函数，但这依然至少存在上述两点问题。

	-----

	若对单用户使用单验证码，但将用户名与验证码对
	应存储到服务器状态中，这可能会使验证码有效期
	变得难以控制。并且考虑到应尽量减少内存使用和
	这种方法的复杂性，故应将其存储于Redis中，初步
	设置有效期为 2 分钟。
*/
/*
	目前该函数对用户识别的机制存在一定缺陷，按照
	设想，已登录的用户不应再次请求注册页面，但这
	似乎有些过于苛刻，在下一版本考虑调整，为注册
	页面和详细信息页面各制定一个验证码生成函数。

	-----

	计划搁置，是此版本主要针对前后端性能优化及功
	能完善。
*/
// 验证码生成函数
pub async fn cap_img_gen(session: Session) -> HttpResponse
{
	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m 刷新验证码。\n\x1b[0m");
	let mut c = Captcha::new();
	c.add_chars(6)
		.apply_filter(Noise::new(0.2))
		.apply_filter(Wave::new(2.0, 20.0).horizontal())
		.apply_filter(Wave::new(2.0, 10.0).vertical())
		.view(220, 120)
		.apply_filter(Dots::new(5))
		.apply_filter(
			Cow::new()
				.min_radius(40)
				.max_radius(50)
				.circles(1)
				.area(Geometry::new(40, 150, 50, 70)),
		);

	let name: String = if let Some(tmp) = get_cookie(&session)
	{
		format!("{}_captcha", tmp.user)
	}
	else
	{
		"Tmp_captcha_code".to_string()
	};

	redis_set_fn(&name, c.chars_as_string()).await;
	redis_expire_fn(&name, 2*60).await;

	if let Some(t) = c.as_png()
	{
		http_model!("Image/Png".to_string(), t, "no_store")
	}
	else
	{
		no_store_http_head(404, "text/html".to_string())
	}
}

// 用户提交信息检查函数 ############################################

const PERMITTED_STRS: &'static str = r####"_.@0123456789abcdefghijklmnopqrstuvwxyz"####; // 预设字符，可根据需求更改。

fn letter_closure<'a>(tmp: &'a str) -> bool
{
	let tmp: String = tmp.to_ascii_uppercase();
	let res: Vec<char> = tmp
		.chars()
		.filter(|&tmp1| ! tmp1.is_ascii_alphanumeric())
		.collect::<Vec<char>>();
	if res.len() != 0
	{
		true
	}
	else
	{
		false
	}
}

// 检查用户名
fn check_username<'a>(username: &'a str) -> bool
{
	if username.len() > 10
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 用户名过长。\x1b[0m");
		return true;
	}

	if letter_closure(username)
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 用户名中含有非法字符。\x1b[0m");
		return true;
	}

	let sql_cmd: String = format!("select id from user_info where name = \'{username}\'");
	if let Some(tmp) = standard_sql::<usize>(Vec::new(), sql_cmd, None)
	{
		if tmp.len() != 0
		{
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 用户名重复。\x1b[0m");
			true
		}
		else
		{
			false
		}
	}
	else
	{
		false
	}
}

// 检查密码
fn check_passwd<'a>(password: &'a str) -> bool
{
	if password.len() != 32
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 密码长度不为32。\x1b[0m");
		return true;
	}

	if letter_closure(password)
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 密码中含有非法字符。\x1b[0m");
		return true;
	}

	false
}

// 邮件名检查
fn check_email<'a>(email: &'a str) -> bool
{
	let email: String = email.to_ascii_lowercase();
	let email_lenghth: usize = email.len();
	if email_lenghth > 25
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 邮件过长。\x1b[0m");
		return true;
	}

	let mut email_new: Vec<char> = Vec::with_capacity(email_lenghth);
	for tmp1 in email.chars()
	{
		for tmp2 in PERMITTED_STRS.chars()
		{
			if tmp1 == tmp2
			{
				email_new.push(tmp1);
				break;
			}
		}
	}

	if email_new.len() != email_lenghth
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 邮件中含有非法字符。\x1b[0m");
		return true;
	}

	false
}

pub fn check_form(form: &UpdateRequest) -> bool
{
	let username: &str = &form.username;
	let password: &str = &form.password;
	let email: &str = &form.email;
	
	if check_username(username)
	|| check_passwd(password)
	|| check_email(email)
	{
		true
	}
	else
	{
		false
	}
}

pub fn mismatch_check_form(form: &UpdateRequest) -> Option<Vec<String>>
{
	let mut res: Vec<String> = Vec::with_capacity(3);

	let username: &str = &form.username;
	if username.len() > 0
	{
		if check_username(username)
		{
			return None;
		}
		else
		{
			println!("用户修改name为：{username}");
			res.push(format!("`name` = '{username}'"));
		}
	}

	// 该处修复了空密码验证的问题
	let password: &str = &form.password;
	let big_passwd: String = password.to_ascii_uppercase();
	if password.len() > 0
	&& big_passwd != "D41D8CD98F00B204E9800998ECF8427E" // [空密码] MD5值。
	{
		if check_passwd(password)
		{
			return None;
		}
		else
		{
			println!("用户修改password为：{big_passwd}");
			res.push(format!("`passwd` = '{}'", big_passwd));
		}
	}

	let email: &str = &form.email;
	if email.len() > 0
	{
		if check_email(email)
		{
			return None;
		}
		else
		{
			println!("用户修改email为：{email}");
			res.push(format!("`email` = '{}'", email.to_ascii_lowercase()));
		}
	}

	Some(res)
}

// 操作确认函数 ############################################

pub async fn filte_option<'a>(key_code: &'a str) -> Option<()>
{
	if key_code.len() != 16
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 路径长度不为16。\x1b[0m");
		return None;
	}
	let res: Vec<char> = key_code.chars()
		.filter(|&tmp| ! tmp.is_ascii_alphanumeric())
		.collect::<Vec<char>>();
	if res.len() != 0
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 路径中含有非法字符。\x1b[0m");
		return None;
	}

	Some(())
}

pub async fn check_option<'a>(key_code: &'a str) -> Option<String>
{
	if let None = filte_option(key_code).await
	{
		return None;
	}

	redis_get_fn::<&str, String>(key_code).await
}

pub async fn option_cofirm<'a>(key_code: String, name: &'a str) -> Option<String>
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m GET: option_cofirm 检测到调用。\x1b[0m");

	if let Some(tmp1) = check_option(&key_code).await
	{
		redis_del_fn(&key_code).await;
		let name: String = format!("{tmp1}{name}");
		if let Some(tmp2) = redis_get_fn::<&str, String>(&name).await
		{
			redis_del_fn(&name).await;
			noback_sql_inline(tmp2);

			println!("    [\x1b[32m+\x1b[0m] \x1b[34m 操作成功。\n\x1b[0m");
			println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m GET: option_cofirm 成功返回。\n\x1b[0m");

			Some(tmp1)
		}
		else
		{
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 在Redis中未提取到用户注册数据。\x1b[0m");
			None
		}
	}
	else
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 在Redis中未提取到注册邮件记录。\x1b[0m");
		None
	}
}