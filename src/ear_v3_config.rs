// ear_v3_config.rs
// 本文件包含浮声三所需的配置参数。

//############################################
// 以下配置应根据实际情况进行修改

// 数据库相关语句
pub const SQL_URL: &'static str  = "mysql://用户:密码@数据库服务器:端口/使用的数据库";
pub const SQL_CMD_01: &'static str = "select id, passwd from user_info where name = ";												// 提取指定用户名的密码				登录页面
pub const SQL_CMD_02: &'static str = "select set_time from user_info where id = ";													// 提取用户注册时间					用户信息页面
pub const SQL_CMD_03: &'static str = "select book_images from book_info where id = ";												// 提取图书封面						浏览页面
pub const SQL_CMD_04: &'static str = "select name, author from book_info where id = ";												// 提取图书书名与作者				浏览页面
pub const SQL_CMD_05: &'static str = "select name, author, publish, public_date, introduction, class from book_info where id = ";	// 提取除id和封面之外的图书信息		详细信息页面
pub const SQL_CMD_06: &'static str = "select id from book_info order by id desc limit 1;";											// 提取当前书目最大数				浏览页面
pub const SQL_CMD_07: &'static str = "select id, name, author from book_info where class = ";										// 提取图书id和类别					浏览页面（分类）
pub const SQL_CMD_08: &'static str = "select borrowed_book from user_info where id = ";												// 提取用户借阅的图书				用户信息页面
pub const SQL_CMD_09: &'static str = "select user_id from user_info where id = ";													// 提取user_id						用户信息页面
pub const SQL_CMD_10: &'static str = "select id, name from book_info where id = ";													// 提取图书id、书名与作者			用户信息页面
pub const SQL_CMD_11: &'static str = "select borrowed_num from user_info where id = ";												// 提取用户借阅的图书数				用户信息页面
pub const SQL_CMD_12: &'static str = "select set_time, email, user_id from user_info where id = ";									// 提取除id和密码之外用户信息		用户信息页面

// Cookie的secret_key明文
pub const KEY_CODE: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

// 单页面最大数据块数量
pub const SEARCH_NUM: usize = 50;

// HTTPS标志文件
pub const KEY_FILE: &'static str = r##"HTTPS"##;

// 网站路由
pub const ROOT: [&'static str; 3] =			[
					"./Web02/main_page/main",
					"./Web02/main_page/ear_logo",
					"./Web02/main_page/main_bg1"
				];
pub const LOGIN: [&'static str; 7] =		[
					"./Web02/login_page/login",
					"./Web02/login_page/md5",
					"./Web02/login_page/ear_logo_w1",
					"./Web02/login_page/ear_logo_w2",
					"./Web02/login_page/ear_logo1",
					"./Web02/login_page/ear_logo2",
					"./Web02/login_page/reset_passwd"
				];
pub const BROWSE: [&'static str; 7] =		[
					"./Web02/browse_page/browse",
					"./Web02/browse_page/browse_c",
					"./Web02/browse_page/main_bg2",
					"./Web02/browse_page/downline",
					"./Web02/browse_page/details",
					"./Web02/browse_page/browse_detail",
					"./Web02/browse_page/main_bg3"
				];
pub const USER_INFO: [&'static str; 14] =		[
					"./Web02/user_page/author",
					"./Web02/user_page/bg01",
					"./Web02/user_page/img1",
					"./Web02/user_page/img2",
					"./Web02/user_page/ryb",
					"./Web02/user_page/ssh",
					"./Web02/user_page/sshw",
					"./Web02/user_page/caoyuan",
					"./Web02/user_page/userpl",
					"./Web02/user_page/user_info",
					"./Web02/user_page/user_info_d1",
					"./Web02/user_page/user_info_d2",
					"./Web02/user_page/user_info_d3",
					"./Web02/user_page/gansu",
				];
pub const PUBLIC: [&'static str; 2] =		[
					"./Web02/user",
					"./Web02/ear_v3_icon"
				];

pub const ACCESS: [&'static str; 1] =		[
					"./Web02/access/404"
				];

pub const REGIST: [&'static str; 3] =		[
					"./Web02/regist_page/regist",
					"./Web02/regist_page/emailnote",
					"./Web02/regist_page/bg04"
				];

pub const EMAIL: [&'static str; 1] =		[
					"./Web02/email/EAR"
				];

// 响应
pub const RES_201: &'static str = "Created：				201		资源已成功创建。";
pub const RES_400: &'static str = "Bad Request：			400		客户端发送了错误的请求。";
pub const RES_401: &'static str = "Unauthorized：			401		需要身份验证。";
pub const RES_403: &'static str = "Forbidden：				403		请求被拒绝，即使提供了正确的凭证。";
pub const RES_404: &'static str = "Not Found：				404		请求的资源不存在。";
pub const RES_405: &'static str = "Method Not Allowed：		405		请求的方法不被允许。";
pub const RES_500: &'static str = "Internal Server Error：	500		服务器遇到了意外情况，无法完成请求。";
//############################################