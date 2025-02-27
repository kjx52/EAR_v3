// user_logout.rs
// 本文件包含退出登录处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称				参数							返回值
----    --------    ----				----							------
21		pub			user_logout			actix_session::Session			HttpResponse

*/

use actix_web::HttpResponse;
use crate::route_handler_fn::basic_fn::{no_store_http_head, redis_update_session_id, get_cookie};

// ## 退出登录 界面响应函数
pub async fn user_logout(session: actix_session::Session) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m user_logout 检测到调用。\x1b[0m");
	if let Some(tmp) = get_cookie(&session)
	{
		if redis_update_session_id(&tmp, "reset").await
		{
			session.purge();
			println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m user_logout 成功返回。\n\x1b[0m");
			HttpResponse::Found()
				.append_header(("Location", "/user/user_login"))
				.finish()
		}
		else
		{
			println!("[\x1b[31mX\x1b[0m] \x1b[34m Redis 操作失败。\x1b[0m");
			no_store_http_head(500, "text/html".to_string())
		}
	}
	else
	{
		println!("[\x1b[31mX\x1b[0m] \x1b[34m 未提取到用户Cookie。\x1b[0m");
		println!("	如果该警告被打印出来，就说明用户成功逃逸中间件检查。应立即停止网站并检修。");
		println!("紧急情况下，可对关键函数使用get_cookie_handler()进行检查。");
		no_store_http_head(500, "text/html".to_string())
	}
}