// user_login.rs
// 本文件包含登陆界面处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称						参数																返回值
----    --------    ----						----																------
41		pub			user_login					Session																HttpResponse
71		pub			user_login_post				Session, web::Json<LoginRequest>									HttpResponse
117		private		set_cookie_handler			(usize, String), actix_session::Session								HttpResponse
130		pub			reset_passwd_1				web::Json<ResetPasswdRequest01>										HttpResponse
160		pub			resetp_cofirm				web::Path<(String, String)>											HttpResponse
175		pub			reset_passwd_2				web::Path<(String, String)>, web::Json<ResetPasswdRequest02>		HttpResponse

*/

use actix_session::Session;
use actix_web::{web, HttpResponse};
use askama::Template;
use crate::ear_v3_config::{SQL_CMD_01, LOGIN};
use crate::ear_v3_struct::{LoginPath, LoginRequest, SessionData02, ResetPasswdRequest01, ResetPasswdRequest02};
use crate::misc::*;
use crate::route_handler_fn::basic_fn::{
	check_login_time,
	check_option,
	filte_option,
	set_cookie,
	get_closure,
	get_cookie_handler,
	http_model,
	no_store_http_head
};
use crate::route_handler_fn::email_send::e_mail_sender;

// ## 登录 界面响应函数
pub async fn user_login<'a>(session: Session) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m GET: user_login 检测到调用。\x1b[0m");

	if get_cookie_handler(&session).await
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m 已登录的用户，GET请求login\x1b[0m");
		/*
			在浏览器重定向异常（如重定向到未知URL），但是BurpSuit抓包发送结果正常的情况下，尝试清除浏览器缓存的数据。
		*/
		return HttpResponse::Found()
			.append_header(("Location", "/access/browse_i1"))
			.finish();
	};

	println!("    [\x1b[34m*\x1b[0m] \x1b[34m 未知用户，GET请求login\x1b[0m");
	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m GET: user_login 成功返回。\n\x1b[0m");
	http_model!("text/html"
		.to_string(),
	LoginPath
	{
		req_path: "/access/browse_i1".to_string(),
	}
		.render()
		.expect("Browse渲染失败："),
		"standard"
	)
}

// ## 登录 POST请求处理函数
pub async fn user_login_post(
	session: Session,
	form: web::Json<LoginRequest>
) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m POST: user_login 检测到调用。\x1b[0m");

	let username: &str = &form.username;

	if ! check_login_time(username).await
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 登陆时间检查失败。\x1b[0m");
		return no_store_http_head(403, "text/html".to_string());
	}

	// 防止并发登录
	if get_cookie_handler(&session).await
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m 已登录的用户，POST请求login\x1b[0m");
		return HttpResponse::Ok().finish();
	};
	
	let password: &str = &form.password;

	println!("    [\x1b[32m+\x1b[0m] \x1b[34m user_login 检测到用户名为：{username}，密码为：{password}。\x1b[0m");

	let tmp: String = format!("{}\'{}\'", SQL_CMD_01, username);
	let sql_data: Vec<(usize, String)> = match standard_sql::<(usize, String)>(Vec::new(), tmp, Some(1))
	{
		Some(t) => t,
		None => return no_store_http_head(401, "text/html".to_string()),
	};

	if sql_data[0].1 != password.to_ascii_uppercase()
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m user_login 密钥序列比对失败。\x1b[0m");
		return no_store_http_head(401, "text/html".to_string());
	}

	println!("    [\x1b[32m+\x1b[0m] \x1b[34m user_login 密钥序列比对成功。\x1b[0m");
	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m POST: user_login 成功返回。\n\x1b[0m");

	set_cookie_handler((sql_data[0].0.clone(), username.to_string()), session).await
}

// post设置Cookie
async fn set_cookie_handler(data: (usize, String), session: actix_session::Session) -> HttpResponse
{
	if set_cookie(session, &SessionData02::make((data.0, data.1))).await
	{
		HttpResponse::Ok().finish()
	}
	else
	{
		no_store_http_head(401, "text/html".to_string())
	}
}

// 重置密码函数
pub async fn reset_passwd_1(
	form: web::Json<ResetPasswdRequest01>
) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m POST: reset_passwd_1 检测到调用。\x1b[0m");

	let name: &str = &form.name;
	let email: &str = &form.email;
	

	let sql_cmd: String = format!("select id from user_info where email = \'{email}\' and name = '{name}'");
	let id: String = match standard_sql::<usize>(Vec::new(), sql_cmd, Some(1))
	{
		Some(t) => t[0].to_string(),
		None => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 未检测到与该邮箱关联的账号。\x1b[0m");
			return no_store_http_head(400, "text/html".to_string());
		}
	};

	if ! e_mail_sender(&email, &id, 3).await
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m reset_passwd 发送邮件失败。\x1b[0m");
		return no_store_http_head(400, "text/html".to_string());
	}

	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m POST: reset_passwd 成功返回。\n\x1b[0m");
	HttpResponse::Ok().finish()
}

pub async fn resetp_cofirm(path: web::Path<(String, String)>) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m GET: resetp_cofirm 检测到调用。\x1b[0m");

	let key_code: String = path.1.clone();
	if let None = filte_option(&key_code).await
	{
		no_store_http_head(400, "text/html".to_string())
	}
	else
	{
		get_closure(LOGIN[6], "html").await
	}
}

pub async fn reset_passwd_2(
	path: web::Path<(String, String)>,
	form: web::Json<ResetPasswdRequest02>
) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m POST: reset_passwd_2 检测到调用。\x1b[0m");

	let key_code: String = path.1.clone();
	if let Some(tmp1) = check_option(&key_code).await
	{
		let passwd: &str = &form.password.to_ascii_uppercase();
		let tmp2: String = format!("UPDATE `你的数据库名`.`user_info` SET `passwd` = '{passwd}' WHERE (`id` = '{tmp1}');");
		noback_sql_inline(tmp2);

		println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m POST: reset_passwd_2 成功返回。\n\x1b[0m");
		HttpResponse::Ok().finish()
	}
	else
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 在Redis中未提取到注册邮件记录。\x1b[0m");
		no_store_http_head(401, "text/html".to_string())
	}
}