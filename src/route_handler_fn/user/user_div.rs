// user_div.rs
// 本文件包含用户界面处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称						参数																				返回值
----    --------    ----						----																				------
46		pub			deal_with_user				Session, web::Path<usize>, web::Data<Mutex<(usize, Vec<BookDiv>)>>					HttpResponse
80		private		user_div_01					String																				HttpResponse
113		private		user_div_02					String																				HttpResponse
169		pub			update_user					Session, web::Json<UpdateRequest>													HttpResponse
245		pub			update_cofirm				web::Path<(String, String)>, actix_session::Session									HttpResponse
268		private		user_div_03					SessionData01, web::Data<Mutex<(usize, Vec<BookDiv>)>>								HttpResponse

*/

use actix_session::Session;
use actix_web::{web, HttpResponse};
use askama::Template;
use std::sync::Mutex;
use crate::ear_v3_struct::{BookDiv, SessionData02, UserDiv01, UserDiv02, UserDiv03, UpdateRequest};
use crate::ear_v3_config::{SQL_CMD_02, SQL_CMD_09, SQL_CMD_10, SQL_CMD_12};
use crate::misc::*;
use crate::route_handler_fn::basic_fn::{
	get_cookie,
	redis_get,
	redis_get_history,
	redis_set_fn,
	redis_expire_fn,
	redis_del_fn,
	http_model,
	no_store_http_head,
	mismatch_check_form,
	option_cofirm
};
use crate::route_handler_fn::book::basic_fn::{return_browse_info_div, parse_pair};
use crate::route_handler_fn::email_send::e_mail_sender;
use crate::route_handler_fn::user::user_logout::user_logout;

// 用户 集成 界面响应函数
pub async fn deal_with_user(
	session: Session,
	page: web::Path<usize>,
	raw_data: web::Data<Mutex<(usize, Vec<BookDiv>)>>
) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m 用户界面集成函数调用。\x1b[0m");
	
	let uname: SessionData02 = match get_cookie(&session)
	{
		Some(t) => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 已登录的用户：{}，GET请求user_div\x1b[0m", t.user);
			t
		},
		None => {
			println!("    [\x1b[33m!\x1b[0m] \x1b[34m 非法访问，GET请求user_div\x1b[0m");
			return HttpResponse::Found()
				.append_header(("Location", "/user/user_login"))
				.finish();
		},
	};

	match page.into_inner()
	{
		1 => user_div_01(&uname).await,
		2 => user_div_02(&uname).await,
		3 => user_div_03(&uname, raw_data).await,
		_ => HttpResponse::Found()
				.append_header(("Location", "/access/user/user_info_d1"))
				.finish()
	}
}

// 用户 全站公告 界面响应函数
async fn user_div_01(session_data: &SessionData02) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m user_div_01 调用。\x1b[0m");

	let user: &str = &session_data.user;
	let user_id: usize = match redis_get(user).await
	{
		Some(t) => t.user_id,
		None => {
			let tmp: String = format!("{}\'{}\'", SQL_CMD_09, session_data.id);
			let user_id_vec: Vec<usize> = match standard_sql::<usize>(Vec::new(), tmp, Some(1))
			{
				Some(t) => t,
				None => return no_store_http_head(500, "text/html".to_string()),
			};
			user_id_vec[0].clone()
		}
	};

	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m user_div_01 成功返回。\n\x1b[0m");
	http_model!("text/html".to_string(),
		UserDiv01
		{
			user_name: user.to_string(),
			user_id: user_id,
		}
		.render()
		.expect("Browse渲染失败："),
		"no_store"
	)
}

// 用户 个人信息 界面响应函数
async fn user_div_02(session_data: &SessionData02) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m user_div_02 调用。\x1b[0m");

	let id: usize = session_data.id;
	let user: &str = &session_data.user;
	let data: (String, String, usize) = match redis_get(user).await
	{
		Some(t) => {
			let tmp: String = format!("{}\'{}\'", SQL_CMD_02, id);
			let sql_data: Vec<String> = match standard_sql::<String>(
				Vec::new(),
				tmp,
				Some(1)
			)
			{
				Some(t) => t,
				None => return no_store_http_head(500, "text/html".to_string()),
			};
			(sql_data[0].clone(), t.email, t.user_id)
		},
		None => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m Redis数据提取失败。\x1b[0m");
			let tmp: String = format!("{}\'{}\'", SQL_CMD_12, id);
			let sql_data: Vec<(String, String, usize)> = match standard_sql::<(String, String, usize)>(
				Vec::new(),
				tmp,
				Some(1)
			)
			{
				Some(t) => t,
				None => return no_store_http_head(500, "text/html".to_string()),
			};
			sql_data[0].clone()
		},
	};

	println!("    [\x1b[34m*\x1b[0m] \x1b[34m user_div_02 SQL语句解析成功。\x1b[0m");
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m user_div_02 成功返回。\n\x1b[0m");
	http_model!("text/html".to_string(),
		UserDiv02
		{
			user_name:	user.to_string(),
			user_id:	data.2,
			set_time:	{
				let tmp: String = data.0;
				format!("{}年 {}月 {}日", &tmp[..4], &tmp[4..6], &tmp[6..])
			},
			e_mail:		data.1,
		}
		.render()
		.expect("Browse渲染失败："),
		"no_store"
	)
}

pub async fn update_user(
	session: Session,
	form: web::Json<UpdateRequest>
) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m POST: update_user 调用。\x1b[0m");

	if form.username.len() == 0
	&& form.password.len() == 0
	&& form.email.len() == 0
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m update_user 空白数据。\x1b[0m");
		return no_store_http_head(405, "text/html".to_string());
	}

	let uname: SessionData02 = match get_cookie(&session)
	{
		Some(t) => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 已登录的用户：{}，POST请求update_user\x1b[0m", t.user);
			t
		},
		None => {
			println!("    [\x1b[33m!\x1b[0m] \x1b[34m 非法访问，POST请求update_user\x1b[0m");
			return HttpResponse::Found()
				.append_header(("Location", "/user/user_login"))
				.finish();
		},
	};

	let res: Vec<String> = match mismatch_check_form(&form)
	{
		Some(t) => t,
		None => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m update_user 请求数据不合规。\x1b[0m");
			return no_store_http_head(401, "text/html".to_string());
		}
	};

	let mut tmp: String = res[0].clone();
	res.iter()
		.skip(1)
		.for_each(|tmp1| tmp.push_str(&format!(", {tmp1}")));

	let name: String = format!("{}_update_data", uname.user);
	redis_set_fn(name.clone(), format!("UPDATE `你的数据库名`.`user_info` SET {} WHERE (`id` = '{}');",
		tmp,
		uname.id
	)).await;
	redis_expire_fn(name, 5*60).await;

	let email: String = if form.email.len() == 0
		{
			match redis_get(&uname.user).await
			{
				Some(t) => t.email,
				None => {
					println!("    [\x1b[31mX\x1b[0m] \x1b[34m Redis数据提取失败。\x1b[0m");
					return no_store_http_head(401, "text/html".to_string());
				}
			}
		}
		else
		{
			form.email.clone()
		};

	if ! e_mail_sender(&email, &uname.user, 2).await
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m update_user 发送邮件失败。\x1b[0m");
		return no_store_http_head(401, "text/html".to_string());
	}

	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m POST: update_user 成功返回。\n\x1b[0m");
	HttpResponse::Ok().finish()
}

pub async fn update_cofirm(path: web::Path<(String, String)>, session: actix_session::Session) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m GET: update_cofirm 检测到调用。\x1b[0m");

	let key_code: String = path.1.clone();
	let name: &str = "_update_data";
	if let Some(tmp1) = option_cofirm(key_code, name).await
	{
		user_logout(session).await;
		redis_del_fn(&tmp1).await;

		println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m GET: update_cofirm 成功返回。\n\x1b[0m");
		HttpResponse::Found()
			.append_header(("Location", "/user/user_login"))
			.finish()
	}
	else
	{
		no_store_http_head(400, "text/html".to_string())
	}
}

// 用户 已借阅图书 界面响应函数
async fn user_div_03(
	session_data: &SessionData02,
	raw_data: web::Data<Mutex<(usize, Vec<BookDiv>)>>
) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m user_div_03 调用。\x1b[0m");

	let tmp: String = format!("select borrowed_book, borrowed_num from user_info where id = \'{}\'", session_data.id);
	let sql_data: Vec<(String, usize)> = match sql_inline!((String, usize), Vec::new(), tmp, Some(1))
	{
		Some(t) => t,
		None => return no_store_http_head(500, "text/html".to_string()),
	};

	let mut res_borrowed_div: String = String::new();
	let mut res_checked_div: String = String::new();
	let mut borrowed_num: usize = 0;
	if sql_data[0].1 != 0
	{
		let sql_data: Vec<usize> = parse_pair(
			Vec::new(),
			&sql_data[0].0,
			','
		);

		let data = raw_data.lock().unwrap();
		let max_book: usize = data.0;
		let book_div: &Vec<BookDiv> = &data.1;
		borrowed_num = sql_data.len();

		sql_data
		.iter()
		.for_each(|tmp|
		{
			if tmp > &max_book
			{
				return
			}
			let page: usize = tmp / 20 + 1;
			if ! data.1[page - 1].key_code
			{
				res_borrowed_div += &return_browse_info_div(*tmp);
			}
			else
			{
				let book_id: usize = tmp % 20;
				res_borrowed_div += &match book_id
				{
					1	=> book_div[page - 1].div_01.clone(),
					2	=> book_div[page - 1].div_02.clone(),
					3	=> book_div[page - 1].div_03.clone(),
					4	=> book_div[page - 1].div_04.clone(),
					5	=> book_div[page - 1].div_05.clone(),
					6	=> book_div[page - 1].div_06.clone(),
					7	=> book_div[page - 1].div_07.clone(),
					8	=> book_div[page - 1].div_08.clone(),
					9	=> book_div[page - 1].div_09.clone(),
					10	=> book_div[page - 1].div_10.clone(),
					11	=> book_div[page - 1].div_11.clone(),
					12	=> book_div[page - 1].div_12.clone(),
					13	=> book_div[page - 1].div_13.clone(),
					14	=> book_div[page - 1].div_14.clone(),
					15	=> book_div[page - 1].div_15.clone(),
					16	=> book_div[page - 1].div_16.clone(),
					17	=> book_div[page - 1].div_17.clone(),
					18	=> book_div[page - 1].div_18.clone(),
					19	=> book_div[page - 1].div_19.clone(),
					20	=> book_div[page - 1].div_20.clone(),
					_	=> String::new(),
				}
			}
			res_borrowed_div += "
				";
		});
	}
	else
	{
		res_borrowed_div += r###"<div class="book_div">
			<a href="/access/book/b0"><img class="img2" src="/access/pic/image_0"></a>
			<div class="book_name"><B>无</B><br>未有借阅记录</div>
		</div>"###;
	}

	let history: Vec<usize> = match redis_get_history(&session_data.user).await
	{
		Some(t) => t,
		None => {
			println!("提取Redis历史记录失败。");
			return no_store_http_head(500, "text/html".to_string());
		}
	};

	history
		.iter()
		.for_each(|id|
			{
				println!("    [\x1b[34m*\x1b[0m] \x1b[34m Cookie存储的借阅记录：{id}\x1b[0m");
				let tmp: String = format!("{}\'{}\'", SQL_CMD_10, id.to_string());
				let checked_book_title: Vec<(String, String)> = match sql_inline!((String, String), Vec::new(), tmp, Some(1))
				{
					Some(t) => t,
					None => vec![(String::from("Error"), String::from("Error"))],
				};

				res_checked_div += &format!(r####"<a href="/access/book/details_b{}"><div class="search_div">{}</div></a>"####,
					checked_book_title[0].0,
					checked_book_title[0].1
				);
				res_checked_div += "
					";
			}
		);

	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m user_div_03 成功返回。\n\x1b[0m");
	http_model!("text/html".to_string(),
		UserDiv03 {
			user_name: session_data.user.clone(),
			book_num: borrowed_num,
			borrowed_div: res_borrowed_div,
			checked_div: res_checked_div,
		}
		.render()
		.expect("Browse渲染失败："),
		"no_store"
	)
}