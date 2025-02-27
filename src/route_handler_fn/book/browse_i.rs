// browse_i.rs
// 本文件包含浏览界面处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称						参数																	返回值
----    --------    ----						----																	------
29		private		return_browse_page_div		usize, usize															String
54		private		build_serv_data				usize, &web::Data<Mutex<(usize, Vec<BookDiv>)>>							bool
150		pub			initialize_browse			web::Path<usize>, web::Data<Mutex<(usize, Vec<BookDiv>)>>				HttpResponse

*/

use actix_web::{web, HttpResponse};
use askama::Template;
use std::sync::Mutex;

use crate::ear_v3_struct::BookDiv;
use crate::route_handler_fn::basic_fn::http_model;

use super::basic_fn::return_browse_info_div;

// 浏览界面底部页码函数
fn return_browse_page_div(max_book: usize, page: usize) -> String
{
	let max_num: usize = max_book / 20 + 1;
	let mut head: String = format!("<li><a href=\"/access/browse_i1\" >«</a></li>
				");
	let end: &str = &format!("<li><a href=\"/access/browse_i{max_num}\" >»</a></li>
				");
	(1..max_num + 1)
	.for_each(|tmp|
		if tmp == page
		{
			head += &format!("<li><a href=\"/access/browse_i{tmp}\" class=\"active\" >{tmp}</a></li>
				");
		}
		else
		{
			head += &format!("<li><a href=\"/access/browse_i{tmp}\" >{tmp}</a></li>
				");
		}
	);
	
	head + end
}

// ## 标准服务器状态格式化函数
fn build_serv_data(
	u_page: usize,
	raw_data: &web::Data<Mutex<(usize, Vec<BookDiv>)>>
) -> bool
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m 请求构建服务器状态。\x1b[0m");
	let mut data = raw_data.lock().unwrap();
	
	// 提前检查
	if u_page - 1 < data.1.len()
	&& data.1[u_page - 1].key_code
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m 检测到状态已构建，返回。\x1b[0m");
		return false;
	}

	let max_book: usize = data.0;
	let req_num: usize = 20 * (u_page);
	let mut null_book_div: String = String::new();
	let mut upper_num: usize = max_book;

	println!("    [\x1b[32m+\x1b[0m] \x1b[34m 用户请求页数 u_page：{u_page}，请求书数 req_num：{req_num}。\x1b[0m");

	if req_num > 0
	&& req_num <= max_book
	{
		upper_num = req_num;
	}
	else if req_num > max_book
	&& req_num <= max_book + 20
	{
		null_book_div = return_browse_info_div(0);
	}
	else
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m 请求数目超过上线过多（20）。\x1b[0m");
		return true;
	}

	println!("    [\x1b[32m+\x1b[0m] \x1b[34m 修改后的书数上线：{upper_num}。\x1b[0m");

	let under_num: usize = req_num - 19;
	let tmp = under_num..(upper_num + 1);
	let mut div_num_vec: Vec<usize> = tmp.collect::<Vec<usize>>();
	div_num_vec.reverse();

	// 核心闭包
	let mut div_push = || -> String
	{
		let tmp = *(div_num_vec.pop().get_or_insert(0));
		if tmp == 0
		{
			null_book_div.clone()
		}
		else
		{
			return_browse_info_div(tmp)
		}
	};

	println!("    [\x1b[32m+\x1b[0m] \x1b[34m 现有服务器状态长度为：{}。\x1b[0m", &data.1.len());

	// 这一段是管理员构建网站时执行的。
	data.1[u_page - 1] = BookDiv
	{
		key_code: true,
		div_01: div_push(),
		div_02: div_push(),
		div_03: div_push(),
		div_04: div_push(),
		div_05: div_push(),
		div_06: div_push(),
		div_07: div_push(),
		div_08: div_push(),
		div_09: div_push(),
		div_10: div_push(),
		div_11: div_push(),
		div_12: div_push(),
		div_13: div_push(),
		div_14: div_push(),
		div_15: div_push(),
		div_16: div_push(),
		div_17: div_push(),
		div_18: div_push(),
		div_19: div_push(),
		div_20: div_push(),
		page_div_push: return_browse_page_div(max_book, u_page),
	};

	println!("        [\x1b[32m+\x1b[0m] \x1b[34m 现有服务器状态长度为：{}。\x1b[0m", &data.1.len());
	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m 构建状态 成功返回。\n\x1b[0m");

	false
}

// ## 浏览 界面响应函数
pub async fn initialize_browse(
	page: web::Path<usize>,
	raw_data: web::Data<Mutex<(usize, Vec<BookDiv>)>>
) -> HttpResponse
{
	let b_page: usize = page.into_inner();
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m browse_i{b_page} 检测到调用。\x1b[0m");

	if b_page == 0
	|| build_serv_data(b_page.clone(), &raw_data)
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m browse_i 非法请求数目。\x1b[0m");
		return HttpResponse::Found()
			.append_header(("Location", "/access/browse_i1"))
			.finish();
	}
	let data = raw_data.lock().unwrap();

	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m browse_i 成功返回。\n\x1b[0m");
	http_model!("text/html"
			.to_string(),
		data.1[b_page - 1]
			.clone()
			.render()
			.expect("Browse渲染失败："),
		"no_store"
	)
}