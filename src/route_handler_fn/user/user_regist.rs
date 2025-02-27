// user_regist.rs
// 本文件包含用户注册处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称				参数									返回值
----    --------    ----				----									------
37		pub			user_regist			Session									HttpResponse
57		private		check_captcha		&'a str									bool
78		pub			user_regist_post	Session, web::Json<RegistRequest>		HttpResponse
142		pub			regist_cofirm		web::Path<(String, String)>				HttpResponse

*/

use actix_session::Session;
use actix_web::{HttpResponse, web};
use chrono::Utc;
use crate::ear_v3_config::REGIST;
use crate::ear_v3_struct::{RegistRequest, UpdateRequest};
use crate::route_handler_fn::basic_fn::{
	get_cookie_handler,
	get_closure,
	no_store_http_head,
	redis_get_fn,
	redis_set_fn,
	redis_expire_fn,
	check_form,
	option_cofirm
};
use crate::route_handler_fn::email_send::e_mail_sender;

// ## 用户注册 GET界面响应函数
pub async fn user_regist(session: Session) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m GET: user_regist 检测到调用。\x1b[0m");

	if get_cookie_handler(&session).await
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m 已登录的用户，GET请求regist\x1b[0m");
		/*
			在浏览器重定向异常（如重定向到未知URL），但是BurpSuit抓包发送结果正常的情况下，尝试清除浏览器缓存的数据。
		*/
		return HttpResponse::Found()
			.append_header(("Location", "/access/user/user_info_d1"))
			.finish();
	};

	println!("    [\x1b[34m*\x1b[0m] \x1b[34m 未知用户，GET请求user_regist\x1b[0m");
	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m GET: user_regist 成功返回。\n\x1b[0m");
	get_closure(REGIST[0], "html").await
}

async fn check_captcha<'a>(cap_string: &'a str) -> bool
{
	if let Some(t) = redis_get_fn::<String, String>(
		"Tmp_captcha_code".to_string()
	).await
	{
		if cap_string == t
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

pub async fn user_regist_post(
	session: Session,
	form: web::Json<RegistRequest>
) -> HttpResponse
{
	let form: RegistRequest = form.into_inner();
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m POST: user_regist 检测到调用。\x1b[0m");

	if get_cookie_handler(&session).await
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m 已登录的用户，POST请求regist\x1b[0m");
		/*
			在浏览器重定向异常（如重定向到未知URL），但是BurpSuit抓包发送结果正常的情况下，尝试清除浏览器缓存的数据。
		*/
		return HttpResponse::Found()
			.append_header(("Location", "/access/user/user_info_d1"))
			.finish();
	};

	println!("    [\x1b[34m*\x1b[0m] \x1b[34m 未知用户，POST请求user_regist\x1b[0m");

	println!("    [\x1b[34m*\x1b[0m] \x1b[34m user_regist 校验验证码。\x1b[0m");
	if ! check_captcha(&form.cap_num).await
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m user_regist 验证码比对失败。\x1b[0m");
		return no_store_http_head(401, "text/html".to_string());
	}
	println!("    [\x1b[1;32m+\x1b[0m] \x1b[34m user_regist 验证码校验成功。\x1b[0m");

	// 邀请码
	if form.right_code != "你的邀请码"
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m user_regist 邀请码错误。\x1b[0m");
		return no_store_http_head(401, "text/html".to_string());
	}

	if check_form(&UpdateRequest::from(&form))
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m user_regist 请求数据不合规。\x1b[0m");
		return no_store_http_head(401, "text/html".to_string());
	}

	let name: String = format!("{}_regist_data", form.username);
	let set_time: String = Utc::now().format("%Y%m%d").to_string();
	
	// 执行下列语句之前请确保执行了 sql_add_user.sql 脚本
	redis_set_fn(name.clone(), format!("CALL sql_add_user(\'{}\', \'{}\', \'{}\', \'{}\')",
		form.username,
		set_time,
		form.password.to_ascii_uppercase(),
		form.email.to_ascii_lowercase()
	)).await;
	redis_expire_fn(name, 5*60).await;

	if ! e_mail_sender(&form.email, &form.username, 1).await
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m user_regist 发送邮件失败。\x1b[0m");
		return no_store_http_head(401, "text/html".to_string());
	}

	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m POST: user_regist 成功返回。\n\x1b[0m");
	HttpResponse::Ok().finish()
}

pub async fn regist_cofirm(path: web::Path<(String, String)>) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m GET: regist_cofirm 检测到调用。\x1b[0m");

	let key_code: String = path.1.clone();
	let name: &str = "_regist_data";
	if let None = option_cofirm(key_code, name).await
	{
		no_store_http_head(400, "text/html".to_string())
	}
	else
	{
		println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m GET: regist_cofirm 成功返回。\n\x1b[0m");
		HttpResponse::Found()
			.append_header(("Location", "/user/user_login"))
			.finish()
	}
}