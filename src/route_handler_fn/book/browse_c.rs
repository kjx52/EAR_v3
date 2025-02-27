// browse_c.rs
// 本文件包含分类界面处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称					参数							返回值
----    --------    ----					----							------
34		pub			class_browse			web::Path<(String, usize)>		HttpResponse

*/

/*
	1	马列主义、毛泽东思想
	2	哲学
	3	社会科学
	4	自然科学
	5	综合性图书
*/

use actix_web::{web, HttpResponse};
use askama::Template;

use crate::ear_v3_struct::SearchDiv;
use crate::ear_v3_config::{SEARCH_NUM, SQL_CMD_07};
use crate::misc::*;
use crate::route_handler_fn::basic_fn::{http_model, no_store_http_head};

// ## 分类 界面响应函数
pub async fn class_browse(
	class: web::Path<(String, usize)>
) -> HttpResponse
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m browse_c{}_page_{} 检测到调用。\x1b[0m", class.0, class.1);

	if class.1 > 9
	|| class.1 == 0
	{
		println!("    [\x1b[33m!\x1b[0m] \x1b[34m 非法访问，页数过大。\x1b[0m");
		return HttpResponse::Found()
			.append_header(("Location", "/access/browse_i1"))
			.finish()
	}

	let (tmp, class_str) = match class.0.as_str()
	{
		"mar" => (1, "马列主义、毛泽东思想".to_string()),
		"phi" => (2, "哲学".to_string()),
		"soc" => (3, "社会科学".to_string()),
		"nat" => (4, "自然科学".to_string()),
		"com" => (5, "综合性图书".to_string()),
		_ => return HttpResponse::Found()
				.append_header(("Location", "/access/browse_i1"))
				.finish(),
	};
	// 页数记得减一
	println!("    [\x1b[34m*\x1b[0m] \x1b[34m browse_c 用户请求分类：{class_str}\x1b[0m");
	let mut res: SearchDiv = SearchDiv {
		class_res: class_str,
		search_div_push: String::new(),
	};

	let tmp: String = format!("{}\'{}\' limit {} offset {}", SQL_CMD_07, tmp, SEARCH_NUM, SEARCH_NUM * (class.1 - 1));
	let sql_data: Vec<(i32, String, String)> = match sql_inline!((i32, String, String), Vec::new(), tmp, None)
	{
		Some(t) => t,
		None => return no_store_http_head(500, "text/html".to_string()),
	};

	sql_data
	.iter()
	.for_each(|tmp2|
	{
		res.search_div_push.push_str(&format!("
		<a href=\"/access/book/details_b{}\"><div class=\"search_div\">{}  {}</div></a>",
		&tmp2.0,
		&tmp2.1,
		&tmp2.2));
	});

	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m browse_c 成功返回。\n\x1b[0m");
	http_model!("text/html".to_string(),
		res.render().expect("Browse渲染失败："),
		"no_store"
	)
}