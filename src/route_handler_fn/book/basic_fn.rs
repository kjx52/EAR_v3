// /book/basic_fn.rs
// 本文件包含基础路由处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称						参数						返回值
----    --------    ----						----						------
30		pub			get_book_images				web::Path<i32>				HttpResponse
51		pub			return_img_div				usize, String				String
56		private		return_browse_title_div		usize						String
70		pub			return_browse_info_div		usize						String
88		pub			parse_pair					Vec<usize>, &'a str, char	Vec<usize>

*/

use actix_web::{web, HttpResponse};
use std::str::FromStr;

use crate::misc::*;
use crate::route_handler_fn::basic_fn::{http_model, no_store_http_head};

use crate::ear_v3_config::{SQL_CMD_03, SQL_CMD_04};

// ## 图书封面 通用 界面响应函数
pub async fn get_book_images(id: web::Path<i32>) -> HttpResponse
{
	let tmp: String = format!("{}\'{}\'", SQL_CMD_03, id.to_string());
	let book_images: Vec<Vec<u8>> = match sql_inline!(Vec<u8>, Vec::new(), tmp, Some(1))
	{
		Some(t) => t,
		None => return no_store_http_head(500, "text/html".to_string()),
	};
	if book_images.len() != 0
	{
		println!("[\x1b[32m+\x1b[0m] \x1b[34m 读取{id}号图书封面。\x1b[0m");
		http_model!("Image/Png".to_string(), book_images[0].clone(), "standard")
	}
	else
	{
		println!("[\x1b[31mX\x1b[0m] \x1b[34m 未查询到{id}号图书封面。\x1b[0m");
		no_store_http_head(404, "text/html".to_string())
	}
}

// 下面三个函数是用于格式化浏览页面数据的函数
pub fn return_img_div(id: usize, tmp: String) -> String
{
	format!(r####"<img class="{tmp}" src="/access/pic/image_{id}">"####)
}

fn return_browse_title_div(id: usize) -> String
{
	let tmp: String = format!("{}\'{}\'", SQL_CMD_04, id.to_string());
	let book_title: Vec<(String, String)> = match standard_sql::<(String, String)>(Vec::new(), tmp, Some(1))
	{
		Some(t) => t,
		None => vec![(String::from("Error"), String::from("Error"))],
	};

	format!(r####"<div class="book_name"><B>{}</B><br>{}</div>"####,
		book_title[0].0,
		book_title[0].1)
}

pub fn return_browse_info_div(id: usize) -> String
{
	let book_img: String = return_img_div(id, "img2".to_string());
	let book_title: String = return_browse_title_div(id);
	format!(r####"<div class="book_div">
			<a href="/access/book/details_b{}">{}</a>
			{}
		</div>"####, id, book_img, book_title)
}

// 递归的SQL解析函数
/*
	这个函数是根据人们阅读习惯进
	行编写的，是人类阅读的逻辑“
	{},{},{}”，或许可以将其改变
	为“{},{},{},”，可以消除冗余
	步骤。
*/
pub fn parse_pair<'a>(mut tmp1: Vec<usize>, tmp2: &'a str, separator: char) -> Vec<usize>
{
	match tmp2.find(separator)
	{
		None =>
		{
			tmp1.push(usize::from_str(tmp2).expect("非有效数字："));
			tmp1
		}
		Some(index) =>
		{
			tmp1.push(usize::from_str(&tmp2[..index]).expect("非有效数字："));
			parse_pair(tmp1, &tmp2[index + 1..], separator)
		}
	}
}