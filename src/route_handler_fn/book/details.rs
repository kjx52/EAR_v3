// details.rs
// 本文件包含详细信息界面处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称							参数																	返回值
----    --------    ----							----																	------
37		private		update_history					&'a str, usize															bool
63		private		check_rep						Vec<(String, usize)>, usize												bool
96		private		get_borrow_book_and_num			usize																	Option<Vec<(String, usize)>>
103		pub			details_browse					Session, web::Path<usize>, web::Data<Mutex<(usize, Vec<BookDiv>)>>		HttpResponse
261		private		update_borrow_book_and_num		String, usize, usize, usize												()
280		private		de_check_captcha				String, &'a str															bool
300		pub			put_details_browse				Session, web::Path<usize>, web::Form<BorrRequest>						HttpResponse
376		pub			del_details_browse				Session, web::Path<usize>												HttpResponse

*/

use actix_session::Session;
use actix_web::{web, HttpResponse};
use askama::Template;
use std::sync::Mutex;

use crate::ear_v3_struct::{SessionData02, BookDiv, BorrRequest, DetailDiv};
use crate::ear_v3_config::{RES_201, SQL_CMD_05};
use crate::misc::*;
use crate::route_handler_fn::basic_fn::{redis_get_history, specify_redis_update, get_cookie, http_model, no_store_http_head, redis_get_fn};

use super::basic_fn::{return_img_div, parse_pair};

// 更新 Redis 浏览记录
async fn update_history<'a>(user: &'a str, book_key: usize) -> bool
{
	let mut history: Vec<usize> = if let Some(old_history) = redis_get_history(user).await
	{
		// 是此版本使用迭代适配器进行高性能过滤。
		old_history
			.into_iter()
			.filter(|&tmp| tmp != book_key)
			.collect::<Vec<usize>>()
	}
	else
	{
		println!("提取Redis历史记录失败。");
		return false;
	};

	if ! (history.len() < 15)
	{
		history = history[1..].to_vec();
	}
	history.push(book_key);

	specify_redis_update(user, history).await
}

// 借阅查重函数
fn check_rep(sql_data: Vec<(String, usize)>, req_book: usize) -> bool
{
	println!("    [\x1b[34m*\x1b[0m] \x1b[34m 借阅查重。\x1b[0m");
	let borrowed_book: &str = &sql_data[0].0;
	let sql_borrowed_num: usize = sql_data[0].1;

	if sql_borrowed_num != 0
	{
		let sql_data_pair: Vec<usize> = parse_pair(
			Vec::new(),
			borrowed_book,
			','
		);

		if sql_data_pair
			.iter()
			.find(|&&tmp| tmp == req_book)
			.is_some()
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

// 带检查的 用于提取借阅书目和数量的 数据库连接函数
fn get_borrow_book_and_num(id: usize) -> Option<Vec<(String, usize)>>
{
	let tmp: String = format!("select borrowed_book, borrowed_num from user_info where id = \'{}\'", id);
	sql_inline!((String, usize), Vec::new(), tmp, Some(1))
}

// ## 详细信息 界面响应函数
pub async fn details_browse(
	session: Session,
	page: web::Path<usize>,
	raw_data: web::Data<Mutex<(usize, Vec<BookDiv>)>>
) -> HttpResponse
{
	let book_key: usize = page.into_inner();
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m GET: details_b{book_key} 检测到调用，执行检查步骤。\x1b[0m");


	let uname: SessionData02 = match get_cookie(&session)
	{
		Some(t) => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 已登录的用户：{}，GET请求details_b\x1b[0m", t.user);
			t
		},
		None => {
			println!("    [\x1b[33m!\x1b[0m] \x1b[34m 非法访问，GET请求details_b\x1b[0m");
			return HttpResponse::Found()
				.append_header(("Location", "/user/user_login"))
				.finish();
		},
	};

	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b 用户令牌校验成功。\x1b[0m");

	let data = raw_data.lock().unwrap();
	let max_book: usize = data.0;

	if book_key > max_book
	|| book_key == 0
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m details_b 用户传入非法索引。\x1b[0m");
		return HttpResponse::Found()
			.append_header(("Location", "/access/book/details_b1"))
			.finish()
	}
	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b 索引检索成功。\x1b[0m");
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m GET: details_b 检查步骤完成，执行GET步骤。\x1b[0m");

	let tmp: String = format!("{}\'{}\'", SQL_CMD_05, book_key);
	let mut res: DetailDiv = DetailDiv {
		book_name: String::new(),
		book_info: String::new(),
		book_intro: String::new(),
		book_class: String::new(),
		book_image: String::new(),
		borrowed_num: String::new(),
		operation: String::new(),
	};

	let sql_data: Vec<(String, String, String, String, String, i32)> = match sql_inline!(
		(String, String, String, String, String, i32),
		Vec::new(),
		tmp,
		Some(1)
	)
	{
		Some(t) => t,
		None => return no_store_http_head(500, "text/html".to_string()),
	};

	sql_data
	.iter()
	.for_each(|tmp2|
	{
		res.book_name = tmp2.0.clone();
		res.book_info = format!("{}<br>
				{} 著<br><br>
				&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;{}<br>
				&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;{}",
			tmp2.0.clone(),
			tmp2.1.clone(),
			tmp2.2.clone(),
			tmp2.3.clone()
		);
		res.book_intro = tmp2.4.clone();
		let class_str: (&str, &str) = match tmp2.5.clone()
			{
				1 => ("/access/browse_cmar_page_1", "马列主义、毛泽东思想"),
				2 => ("/access/browse_cphi_page_1", "哲学"),
				3 => ("/access/browse_csoc_page_1", "社会科学"),
				4 => ("/access/browse_cnat_page_1", "自然科学"),
				5 => ("/access/browse_ccom_page_1", "综合性图书"),
				_ => {
					println!("[\x1b[31mX\x1b[0m] \x1b[34mDetails请求的图书类别解析失败。\x1b[0m");
					("", "")
				},
			};
		res.book_class = format!("<a href=\"{}\">{}</a>", class_str.0, class_str.1);
		res.book_image = return_img_div(book_key, "div_2".to_string());
	});

	let sql_data2: Vec<(String, usize)> = match get_borrow_book_and_num(uname.id)
	{
		Some(t) => t,
		None => return no_store_http_head(500, "text/html".to_string()),
	};
	let sql_borrowed_num: usize = sql_data2[0].1;

	// 共定义了三种状态
	if check_rep(sql_data2, book_key)
	{
		res.operation = format!("归还📑");
		res.borrowed_num = r#####"<div id="popup" class="popup">
			<h2>您确定要归还此书吗？</h2>
			<button class="btn" onclick="hidePopup()"><B>取消</B></button>
			<button class="btn2" onclick="returnbook()"><B>确定</B></button>
		</div>"#####.to_string();
	}
	else
	{
		res.operation = format!("订阅🔔");
		res.borrowed_num = if sql_borrowed_num < 5
		{
			format!(r#####"<div id="popup" class="popup">
				<h2>您已借阅了 <B>{}</B> 本书</h2><br>
				<p>您可以借阅的图书数量上限为：5。<br>
				输入下列验证码确认借阅：（区分大小写）</p>
				<img id="cap_img" src="/pic/cap">
				<font id="re_font" style="color:green">刷新成功。</font><br>
				<button id="refresh_cap" onclick="refresh_cap()">刷新验证码</button><br>
				<form id="form01" method="PUT">
					<input type="text" name="cap_num" id="cap_num" required pattern="[A-Za-z0-9]+" title="请输入验证码" />
					<button>提交</button>
				</form>
				<button class="btn" onclick="hidePopup()"><B>关闭</B></button>
			</div>"#####, sql_borrowed_num)
		}
		else
		{
			format!(r#####"<div id="popup" class="popup">
				<h2>您已借阅了 <B>{}</B> 本书</h2><br>
				<p>您已达到借阅的图书数量上限：5。<br>
				若要继续借阅请归还已借阅图书后再进行尝试。</p>
			
				<button class="btn" onclick="hidePopup()"><B>关闭</B></button>
			</div>"#####, sql_borrowed_num)
		};
	}

	if update_history(&uname.user, book_key).await
	{
		println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b Cookie更新成功。\x1b[0m");
		println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m GET: details_b 成功返回。\n\x1b[0m");
		http_model!("text/html".to_string(),
			res.render().expect("Browse渲染失败："),
			"no_store"
		)
	}
	else
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m details_b Cookie更新失败。\x1b[0m");
		no_store_http_head(400, "text/html".to_string())
	}
}

// 用于更新借阅书目和数量的 数据库连接函数
fn update_borrow_book_and_num(
	tmp1: String,
	tmp2: usize,
	tmp3: usize,
	tmp4: usize
) -> ()
{
	let tmp: String = format!(
		r###"UPDATE `你的数据库名`.`user_info` SET `borrowed_book` = '{}' WHERE (`id` = '{}');
			UPDATE `你的数据库名`.`user_info` SET `borrowed_num` = '{}' WHERE (`id` = '{}');"###,
		tmp1,
		tmp2,
		tmp3,
		tmp4
	);
	sql_inline!(tmp);
}

// 验证码检查函数
async fn de_check_captcha<'a>(cap_name: String, cap_string: &'a str) -> bool
{
	if let Some(t) = redis_get_fn::<String, String>(cap_name).await
	{
		if cap_string == t
		{
			false
		}
		else
		{
			true
		}
	}
	else
	{
		true
	}
}

// ## 详细信息 PUT请求处理函数
pub async fn put_details_browse(
	session: Session,
	page: web::Path<usize>,
	form: web::Form<BorrRequest>
) -> HttpResponse
{
	let book_key: usize = page.into_inner();
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m PUT: details_b{book_key} 检测到调用，执行检查步骤。\x1b[0m");

	let uname: SessionData02 = match get_cookie(&session)
	{
		Some(t) => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 已登录的用户：{}，PUT请求details_b\x1b[0m", t.user);
			t
		},
		None => {
			println!("    [\x1b[33m!\x1b[0m] \x1b[34m 非法访问，PUT请求details_b\x1b[0m");
			return HttpResponse::Found()
				.append_header(("Location", "/user/user_login"))
				.finish();
		},
	};

	let cap_name: String = format!("{}_captcha", uname.user);
	if de_check_captcha(cap_name, &form.cap_num).await
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m 验证码错误，用户{} PUT请求details_b\x1b[0m", uname.user);
		return no_store_http_head(401, "text/html".to_string());
	}
	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b 验证码比对成功。\x1b[0m");

	let sql_data: Vec<(String, usize)> = match get_borrow_book_and_num(uname.id)
	{
		Some(t) => t,
		None => return no_store_http_head(500, "text/html".to_string()),
	};
	let borrowed_book: String = sql_data[0].0.clone();
	let sql_borrowed_num: usize = sql_data[0].1;

	if sql_borrowed_num > 4
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 前端被绕过，用户直接进行PUT请求。\x1b[0m");
		return no_store_http_head(403, "text/html".to_string());
	}
	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b 借阅数据比对成功。\x1b[0m");

	if check_rep(sql_data, book_key)
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 重复借阅。\x1b[0m");
		return no_store_http_head(403, "text/html".to_string());
	}
	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b 未查询到重复借阅。\x1b[0m");
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m PUT: details_b 检查步骤完成，执行PUT步骤。\x1b[0m");

	// Cookie 内容似乎可以被修改，故将下列语句更改为数据库数据自加。
	update_borrow_book_and_num(
		if sql_borrowed_num != 0
		{
			format!("{},{}", borrowed_book, book_key)
		}
		else
		{
			book_key.to_string()
		},
		uname.id,
		sql_borrowed_num + 1,
		uname.id
	);

	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b SQL更新成功。\x1b[0m");
	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m PUT: details_b 成功返回。\n\x1b[0m");

	http_model!(201, "text/html".to_string(), RES_201.to_string(), "no_store")
}

// 详细信息 DELETE请求处理函数
pub async fn del_details_browse(
	session: Session,
	page: web::Path<usize>
) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m DEL: details_b 检测到调用，执行检查步骤。\x1b[0m");
	
	let uname: SessionData02 = match get_cookie(&session)
	{
		Some(t) => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 已登录的用户：{}，DEL请求details_b\x1b[0m", t.user);
			t
		},
		None => {
			println!("    [\x1b[33m!\x1b[0m] \x1b[34m 非法访问，DEL请求details_b\x1b[0m");
			return HttpResponse::Found()
				.append_header(("Location", "/user/user_login"))
				.finish();
		},
	};

	let sql_data: Vec<(String, usize)> = match get_borrow_book_and_num(uname.id)
	{
		Some(t) => t,
		None => return no_store_http_head(500, "text/html".to_string()),
	};
	let sql_borrowed_num: usize = sql_data[0].1;
	let req_book: usize = page.into_inner();

	if sql_borrowed_num == 0
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 前端被绕过，用户直接进行DEL请求。\x1b[0m");
		return no_store_http_head(403, "text/html".to_string());
	}
	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b 借阅数据比对成功。\x1b[0m");

	let mut sql_data_pair: Vec<usize> = parse_pair(
		Vec::new(),
		&sql_data[0].0,
		','
	);

	let index: usize = match sql_data_pair
		.iter()
		.position(|&tmp| tmp == req_book)
	{
		Some(t) => t,
		None => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 未查询到借阅记录。\x1b[0m");
			return no_store_http_head(403, "text/html".to_string());
		}
	};
	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b 借阅记录核对完成。\x1b[0m");
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m DEL: details_b 检查步骤完成，执行DELETE步骤。\x1b[0m");

	let _ = sql_data_pair.remove(index);
	let borrowed_book_new: String = if sql_data_pair.len() == 0
	{
		String::new()
	}
	else
	{
		let sql_data_pair_len: usize = sql_data_pair.len() - 1;
		sql_data_pair[..sql_data_pair_len]
			.iter()
			.map(|tmp|
			{
				/*
				这些步骤是为解析函数而准备的，而SQL解析函数
				的逻辑是人类阅读的逻辑“{},{},{}”，或许可
				以将其改变为“{},{},{},”，可以消除冗余步骤。
				*/
				format!("{tmp},")
			})
			.collect::<String>()
		+ &sql_data_pair[sql_data_pair_len].to_string()
	};

	update_borrow_book_and_num(
		borrowed_book_new,
		uname.id,
		sql_borrowed_num - 1,
		uname.id
	);

	println!("    [\x1b[32m+\x1b[0m] \x1b[34m details_b SQL更新成功。\x1b[0m");
	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m DEL: details_b 成功返回。\n\x1b[0m");

	HttpResponse::NoContent().finish()
}