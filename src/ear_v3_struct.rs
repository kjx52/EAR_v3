// ear_v3_struct.rs
// 本文件包含浮声三所需的结构体。

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的结构体有：
行号	是否公有	名称
----    --------    ----
45		pub			SessionData01
76		pub			SessionData02
119		pub			LoginRequest
126		pub			ResetPasswdRequest01
133		pub			ResetPasswdRequest02
139		pub			RegistRequest
149		pub			UpdateRequest
177		pub			BorrRequest

190		pub			ErrorHandler
198		pub			LoginPath
209		pub			BookDiv
277		pub			SearchDiv
289		pub			DetailDiv
306		pub			UserDiv01
314		pub			UserDiv02
324		pub			UserDiv03
334		pub			EmailDiv01

*/

//	*本模块定义的渲染结构体路径可能需要根据本地计算机重新定义

//############################################

use askama::Template;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

// Redis存储格式
#[derive(Serialize, Deserialize)]
pub struct SessionData01 {
	pub id:				usize,
	pub user:			String,
	pub user_id:		usize,
	pub email:			String,
	pub session_id:		String,
	pub last_login:		String,
	pub history:		Vec<usize>,
}

impl SessionData01 {
	pub fn new() -> SessionData01
	{
		SessionData01 {
			id:			0,
			user:		String::new(),
			user_id:	0,
			email:		String::new(),
			session_id: String::new(),
			last_login:	String::new(),
			history:	Vec::new(),
		}
	}
}

/*
	处于完整度考量，该结构
	体中加入了user_id。
*/
// Cookie数据格式
#[derive(Serialize, Deserialize)]
pub struct SessionData02 {
	pub id: usize,
	pub user: String,
	pub session_id: String,
}

impl SessionData02 {
	pub fn new() -> SessionData02
	{
		SessionData02 {
			id: 0,
			user: String::new(),
			session_id: String::new(),
		}
	}

	pub fn make(data: (usize, String)) -> SessionData02
	{
		let session_id: String = Uuid::new_v4().to_string();
		SessionData02 {
			id: data.0,
			user: data.1,
			session_id: session_id,
		}
	}
}

/*
	是次版本启用二级存储，
	故此Cookie数据格式废弃

// Cookie数据格式
#[derive(Serialize, Deserialize)]
pub struct SessionData01 {
	pub id: usize,
	pub user: String,
	pub history: Vec<usize>,
	// borrowed_num: usize, // 移除。
}
*/

// 登录POST请求数据格式
#[derive(Deserialize)]
pub struct LoginRequest {
	pub username: String,
	pub password: String,
}

// 置密码POST请求数据格式
#[derive(Deserialize)]
pub struct ResetPasswdRequest01 {
	pub name: String,
	pub email: String,
}

// 重置密码POST请求数据格式
#[derive(Deserialize)]
pub struct ResetPasswdRequest02 {
	pub password: String,
}

// 注册POST请求数据格式
#[derive(Deserialize)]
pub struct RegistRequest {
	pub username: String,
	pub password: String,
	pub email: String,
	pub cap_num: String,
	pub right_code: String,
}

// 更新POST请求数据格式
#[derive(Deserialize)]
pub struct UpdateRequest {
	pub username: String,
	pub password: String,
	pub email: String,
}

impl From<RegistRequest> for UpdateRequest {
	fn from(data: RegistRequest) -> Self {
		UpdateRequest {
			username:	data.username.clone(),
			password:	data.password.clone(),
			email:		data.email.clone(),
		}
	}
}

impl From<&RegistRequest> for UpdateRequest {
	fn from(data: &RegistRequest) -> Self {
		UpdateRequest {
			username:	data.username.clone(),
			password:	data.password.clone(),
			email:		data.email.clone(),
		}
	}
}

// 订阅PUT请求数据格式
#[derive(Deserialize)]
pub struct BorrRequest {
	pub cap_num: String,
}

// 下面的结构体均被用于渲染网页
//############################################

/*
	该结构体用于存储标准错误界面错误信息
*/
#[derive(Template)]
#[template(path = "./../src/Web02/access/ErrorHandler.html", escape = "none")]
#[derive(Clone)]
pub struct ErrorHandler {
	pub error_detial: String,
}

// 登陆界面重定向路径
#[derive(Template)]
#[template(path = "./../src/Web02/login_page/login.html", escape = "none")]
#[derive(Clone)]
pub struct LoginPath {
	pub req_path: String,
}

/*
	下面的结构体用于存储浏览页面的HTML数据块
	一页20个。
*/
#[derive(Template)]
#[template(path = "./../src/Web02/browse_page/browse.html", escape = "none")]
#[derive(Clone)]
pub struct BookDiv {
	/*
		<div class="book_div">
			<img class="img2" src="./images/1.jpg">
			<div class="book_name"><B>{{ book_name }}<br>{{ author }}</B></div>
		</div>
	*/
	pub key_code: bool,
	pub div_01: String,
	pub div_02: String,
	pub div_03: String,
	pub div_04: String,
	pub div_05: String,
	pub div_06: String,
	pub div_07: String,
	pub div_08: String,
	pub div_09: String,
	pub div_10: String,
	pub div_11: String,
	pub div_12: String,
	pub div_13: String,
	pub div_14: String,
	pub div_15: String,
	pub div_16: String,
	pub div_17: String,
	pub div_18: String,
	pub div_19: String,
	pub div_20: String,
	pub page_div_push: String,
}

impl BookDiv {
	pub fn new() -> BookDiv
	{
		BookDiv {
			key_code: false,
			div_01: String::new(),
			div_02: String::new(),
			div_03: String::new(),
			div_04: String::new(),
			div_05: String::new(),
			div_06: String::new(),
			div_07: String::new(),
			div_08: String::new(),
			div_09: String::new(),
			div_10: String::new(),
			div_11: String::new(),
			div_12: String::new(),
			div_13: String::new(),
			div_14: String::new(),
			div_15: String::new(),
			div_16: String::new(),
			div_17: String::new(),
			div_18: String::new(),
			div_19: String::new(),
			div_20: String::new(),
			page_div_push: String::new(),
		}
	}
}

/*
	下面的结构体用于存储浏览页面用于分类的
	HTML数据块一页50个。
*/
#[derive(Template)]
#[template(path = "./../src/Web02/browse_page/browse_c.html", escape = "none")]
#[derive(Clone)]
pub struct SearchDiv {
	pub class_res: String,
	pub search_div_push: String,
}

/*
	下面的结构体用于存储详细信息页面的HTML
	数据块。
*/
#[derive(Template)]
#[template(path = "./../src/Web02/browse_page/details.html", escape = "none")]
#[derive(Clone)]
pub struct DetailDiv {
	pub book_name: String,
	pub book_info: String,
	pub book_intro: String,
	pub book_class: String,
	pub book_image: String,
	pub borrowed_num: String,
	pub operation: String,
}

/*
	下面的三个结构体均用于存储详细信息页面
	的HTML数据块。
*/
#[derive(Template)]
#[template(path = "./../src/Web02/user_page/user_info_d1.html", escape = "none")]
#[derive(Clone)]
pub struct UserDiv01 {
	pub user_name: String,
	pub user_id: usize,
}

#[derive(Template)]
#[template(path = "./../src/Web02/user_page/user_info_d2.html", escape = "none")]
#[derive(Clone)]
pub struct UserDiv02 {
	pub user_name: String,
	pub user_id: usize,
	pub set_time: String,
	pub e_mail: String,
}

#[derive(Template)]
#[template(path = "./../src/Web02/user_page/user_info_d3.html", escape = "none")]
#[derive(Clone)]
pub struct UserDiv03 {
	pub user_name: String,
	pub book_num: usize,
	pub borrowed_div: String,
	pub checked_div: String,
}

#[derive(Template)]
#[template(path = "./../src/Web02/email/regist_email.html", escape = "none")]
#[derive(Clone)]
pub struct EmailDiv01 {
	pub div1: String,
	pub div2: String,
	pub div3: String,
	pub div4: String,
	pub account_confirmation: String,
}

//############################################