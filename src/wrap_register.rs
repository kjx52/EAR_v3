// wrap_register.rs
// 本文件包含中间件注册

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称						参数								返回值
----    --------    ----						----								------
124		private		redis_get_sync				&'a str								Option<SessionData01>
152		private		redis_session_id_get		&'a str								Option<String>
164		private		get_cookie_handler			&actix_session::Session				bool

#==============================#
	本模块定义的中间件有：
行号	是否公有	名称				描述
----    --------    ----				----
50		pub			PermyE				用于检查用户 Cookie

#==============================#
	本模块定义的中间件函数有：
行号	是否公有	名称				参数						返回值
----    --------    ----				----						----
193		pub			add_error_body		ServiceResponse<B>			Result<ErrorHandlerResponse<B>, Error>

*/

use actix_session::{Session, SessionExt};
use actix_web::dev::{forward_ready, Transform, Service, ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpResponse};
use actix_web::http::header::{self, HeaderMap, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::middleware::ErrorHandlerResponse;
use askama::Template;
use redis::{Client, Commands};
use std::future::{Future, Ready, ready};
use std::pin::Pin;

use crate::ear_v3_config::{RES_400, RES_401, RES_403, RES_500};
use crate::ear_v3_struct::{SessionData01, LoginPath, ErrorHandler};
use crate::route_handler_fn::basic_fn::{get_cookie, standard_local_read_file};

/*
	注册用于检查 Cookie 的中间件 PermyE
*/
// #=========================
pub struct PermyE;

impl<S> Transform<S, ServiceRequest> for PermyE
where
	S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
	S::Future: 'static,
{
	type Response = ServiceResponse;
	type Error = Error;
	type InitError = ();
	type Transform = PermyEMiddleware<S>;
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ready(Ok(PermyEMiddleware { service }))
	}
}

pub struct PermyEMiddleware<S> {
	service: S,
}

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

impl<S> Service<ServiceRequest> for PermyEMiddleware<S>
where
	S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
	S::Future: 'static,
{
	type Response = ServiceResponse;
	type Error = Error;
	type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

	// 确保当下一个服务准备就绪时，此服务也已准备就绪
	forward_ready!(service);

	fn call(&self, req: ServiceRequest) -> Self::Future {
		let session: Session = req.get_session();

		// 二代检查
		if get_cookie_handler(&session)
		{
			println!("\x1b[1;30m【中间件集中检查】 允许访问。\x1b[0m");
			let fut = self.service.call(req);

			Box::pin(async move {
				let res = fut.await?;

				println!("\x1b[1;30m【中间件集中检查】 成功响应。\n\n\x1b[0m");
				Ok(res)
			})
		}
		else
		{
			println!("\x1b[1;31m【中间件集中检查】 无效会话。\x1b[0m");
			let bad_res = HttpResponse::build(StatusCode::OK)
				.content_type("text/html; charset=utf-8")
				.append_header(("Cross-Origin-Resource-Policy", "same-site"))
				.append_header(("Cache-Control",				"no-store"))
				.append_header(("X-Content-Type-Options",		"nosniff"))
				.append_header(("X-Frame-Options",				"DENY"))
				.body(LoginPath
					{
						req_path: req.path().to_string(),
					}
					.render()
					.expect("Browse渲染失败："));
			Box::pin(async move {
				Ok(req.into_response(bad_res))
			})
		}
	}
}

fn redis_get_sync<'a>(user: &'a str) -> Option<SessionData01>
{
	let mut redis_connect = Client::open("redis://127.0.0.1:6379")
		.expect("打开Redis失败：")
		.get_connection()
		.expect("获取Redis链接失败：");

	match redis_connect
		.get::<&str, String>(user)
	{
		Ok(t) => {
			match serde_json::from_str::<SessionData01>(&t)
			{
				Ok(t2) => Some(t2),
				Err(e) => {
					println!("    [\x1b[31mX\x1b[0m] \x1b[34m JSON解析失败：{e}\x1b[0m");
					None
				}
			}
		},
		Err(e) => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 未检测到现有Redis数据：{e}\x1b[0m");
			None
		},
	}
}

#[inline]
fn redis_session_id_get<'a>(user: &'a str) -> Option<String>
{
	if let Some(t) = redis_get_sync(user)
	{
		Some(t.session_id.clone())
	}
	else
	{
		None
	}
}

fn get_cookie_handler(session: &actix_session::Session) -> bool
{
	if let Some(tmp) = get_cookie(session)
	{
		if let Some(t) = redis_session_id_get(&tmp.user)
		{
			if t == tmp.session_id
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
	else
	{
		false
	}
}

// #=========================

// ## 中间件 添加默认错误响应体
pub fn add_error_body<B>(result: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, Error> {
	let (res_head, res_body) = result.into_parts();
	let res_code: u16 = res_body.status().as_u16();

	println!("\x1b[1;30m[中间件] {res_code}响应");
	let res = res_body.set_body(
		if res_code == 404
		{
			String::from_utf8(
				*(standard_local_read_file("./Web02/access/404.html")
				.1)
			)
			.expect("404页面解析错误：")
		}
		else
		{
			ErrorHandler
			{
				error_detial: match res_code
					{
						400 => RES_400,
						401 => RES_401,
						403 => RES_403,
						_ => RES_500,
					}
					.to_string(),
			}
			.render()
			.expect("Browse渲染失败：")
		}
	);

	let mut res = ServiceResponse::new(res_head, res)
		.map_into_boxed_body()
		.map_into_right_body();

	let res_headers: &mut HeaderMap = res.headers_mut();
	let _ = res_headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));

	Ok(ErrorHandlerResponse::Response(res))
}