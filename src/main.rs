// ear_v3_2.1.2
// main.rs

// #################################################################
// 考虑到项目安全性及开发成本，是此版本将中止在Shuttle上的部署计划。
// #################################################################

/*
	是此版本主要针对前后端性能优化及功能完善。
	是此版本应是完整的浮声项目，包含所有逻辑处理、日志检测、项目控制等。
	本项目应很好地延续Rust的惰性原则。
	本项目应遵循最小开销原则。
*/

/*
	本项目采用Sha256加密作为Cookie的secret_key，使用32位MD5加密作为用户密码。
*/

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称			参数								返回值
----    --------    ----			----								------
140		private		tls_set()		/									Option<SslAcceptorBuilder>
168		private		main()			/									std::io::Result<()>

*/

extern crate actix_web;
extern crate actix_session;
extern crate mysql;
extern crate tokio;

pub mod ear_v3_config;
pub mod ear_v3_struct;
#[macro_use] pub mod misc;
mod wrap_register;
#[macro_use] mod route_handler_fn;

use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_session::config::{CookieContentSecurity, PersistentSession};
use actix_web::{App, HttpServer, web};
use actix_web::cookie::{Key, time};
use actix_web::middleware::{ErrorHandlers, Logger};
use env_logger::Env;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslMethod};
use std::env;
use std::io::Write;
use std::fs::{self, File};
use std::sync::Mutex;

use crate::ear_v3_config::*;
use crate::ear_v3_struct::*;
use crate::misc::*;
use crate::route_handler_fn::basic_fn::{cap_img_gen, get_closure};
use crate::route_handler_fn::book::basic_fn::get_book_images;
use crate::route_handler_fn::book::browse_c::class_browse;
use crate::route_handler_fn::book::browse_i::initialize_browse;
use crate::route_handler_fn::book::details::{details_browse, del_details_browse, put_details_browse};
use crate::route_handler_fn::user::user_div::{deal_with_user, update_user, update_cofirm};
use crate::route_handler_fn::user::user_login::{user_login, user_login_post, reset_passwd_1, reset_passwd_2, resetp_cofirm};
use crate::route_handler_fn::user::user_logout::user_logout;
use crate::route_handler_fn::user::user_regist::{user_regist, user_regist_post, regist_cofirm};
use crate::wrap_register::*;

const HELP_MESSAGE: &'static str = r#####"
EAR_v3 浮声三 Web 服务
版本：2.1.2
Двигатель Ржавчина 2022-2026.
智能化图书管理系统 浮声三 后端服务
基于 Actix_web 框架

**该服务应使用配套启动器启动并监测。**

用法： ear_v3_web [模块 MODEL] [参数 OPTION] ...
模块：
    <IP>    启动所需的服务器网络地址。
    help    显示此帮助信息。

参数：
仅在启动时可用。
目前仅支持短选项。
    -m      以最完整的长格式列出网络日志包括大量 HTTP 标头，这
            将会导致过多的输出，应仅在调试时使用。
    -l      默认情况下的日志选项，会详细列出网络日志，包括开始
            处理请求的时间、为请求提供服务的子进程的进程 ID 等。
    -d      最简单的日志格式，仅列出关键项，如远程 IP 地址、请
            求头等。

本项目遵守 GPL-2.0 许可证
Written by Jessarin000.
"#####;

const MAX_MODE: &'static str = r#####"

[请求时间：%t] 🙂
IP：%a

请求的第一行：'%r'
为请求提供服务的子进程的进程 ID：%P
响应状态代码：%s
响应的大小：%b
处理请求所用的时间（毫秒）：%D

## 请求标头：
Accept: %{Accept}i
Content-Type: %{Content-Type}i
Date: %{Date}i
Range: %{Range}i

Cache-Control: %{Cache-Control}i
Cookie: %{Cookie}i

Host: %{Host}i
Referer: %{Referer}i
User-Agent: %{User-Agent}i

Authorization: %{Authorization}i
Proxy-Authorization: %{Proxy-Authorization}i

## 响应标头：
Content-Encoding: %{Content-Encoding}o
Content-Type: %{Content-Type}o
Date: %{Date}o
ETag: %{ETag}o
Expires: %{Expires}o
Last-Modified: %{Last-Modified}o
Location: %{Location}o
Server: %{Server}o
Set-Cookie: %{Set-Cookie}o
Transfer-Encoding: %{Transfer-Encoding}o

"#####;

fn tls_set() -> Option<SslAcceptorBuilder>
{
	match SslAcceptor::mozilla_intermediate(SslMethod::tls())
	{
		// 这是 SSL 密钥的路径，应自行修改
		Ok(mut builder) => match builder.set_private_key_file(r##"key_PATH"##, openssl::ssl::SslFiletype::PEM)
		{
			Ok(()) => match builder.set_certificate_chain_file(r##"pem_PATH"##)
			{
				Ok(()) => Some(builder),
				Err(e) => {
					println!("[\x1b[1;31mX\x1b[0m] 加载证书链失败：{e}");
					None
				},
			},
			Err(e) => {
				println!("[\x1b[1;31mX\x1b[0m] TLS私钥读取失败：{e}");
				None
			},
		}
		Err(e) => {
			println!("[\x1b[1;31mX\x1b[0m] TLS构建器建立失败：{e}");
			None
		}
	}
}

// #### Actix_web主函数
#[actix_web::main]
async fn main() -> std::io::Result<()>
{
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m Actix_web主函数 检测到调用。\x1b[0m");

	env_logger::init_from_env(Env::default().default_filter_or("info"));
	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m 日志初始化完成。\x1b[0m");

	let builder = tls_set();
	match builder
	{
		Some(_) => println!("    [\x1b[1;34m*\x1b[0m] \x1b[1;34m TLS 初始化完成，执行检查步骤。\x1b[0m"),
		None => writeln!(std::io::stderr(), "    [\x1b[33m!\x1b[0m] \x1b[34m HTTPS启动失败，尝试使用HTTP启动。\x1b[0m").unwrap(),
	}

	let ip: Vec<String> = env::args().collect();

	if ip.len() == 1
	{
		writeln!(std::io::stderr(), "    [\x1b[34m*\x1b[0m] \x1b[34m 用法： {} <服务器 IP 地址> <日志模式> ...\x1b[0m", ip[0]).unwrap();
		writeln!(std::io::stderr(), "    [\x1b[34m*\x1b[0m] \x1b[34m 使用 `{} help` 来获取更多信息。\x1b[0m", ip[0]).unwrap();
		std::process::exit(1);
	}

	if ip.len() > 3
	{
		writeln!(std::io::stderr(), "    [\x1b[31mX\x1b[0m] \x1b[34m 错误：检测到多个参数。\x1b[0m").unwrap();
		std::process::exit(2);
	}

	if &ip[1] == "help"
	{
		println!("{}", HELP_MESSAGE);
		std::process::exit(0);
	}

	/*
		%% 百分号
		%a 远程 IP 地址（如果使用反向代理，则为代理的 IP 地址）
		%t 开始处理请求的时间
		%P 为请求提供服务的子进程的进程 ID
		%r 请求的第一行
		%s 响应状态代码
		%b 响应的大小（以字节为单位），包括 HTTP 标头
		%T 处理请求所用的时间（以秒为单位，浮点分数为 .06f 格式）
		%D 处理请求所用的时间（以毫秒为单位）
		%{FOO}i request.headers['FOO']
		%{FOO}o response.headers['FOO']
		%{FOO}e os.environ['FOO']
	*/

	let mode: String = if ip.len() == 3
	{
		// 127.0.0.1:54278  "GET /test HTTP/1.1"  404_<20b>  0.1074ms  "-"  "HTTPie/2.2.0"
		match ip[2].as_str()
		{
			"-m" => MAX_MODE,
			"-l" => "[%t]  %a  \"%r\"  %P  %s_<%bb>  %Dms  \"%{Referer}i\"  \"%{User-Agent}i\"",
			"-d" => "%a  \"%r\"  %s_<%bb>  %Dms  \"%{Referer}i\"  \"%{User-Agent}i\"",
			_ => {
				writeln!(std::io::stderr(), "    [\x1b[31mX\x1b[0m] \x1b[34m 错误：未知参数。\x1b[0m").unwrap();
				std::process::exit(3);
			},
		}.to_string()
	}
	else
	{
		"[%t]  %a  \"%r\"  %P  %s_<%bb>  %Dms  \"%{Referer}i\"  \"%{User-Agent}i\"".to_string()
	};

	println!("[\x1b[1;34m*\x1b[0m] \x1b[1;34m Actix_web主函数 检查步骤完成，执行启动步骤。\x1b[0m");

	/*
		Actix_Session 组件是需要前置组件用于存储数据
		的，这里选用的是“redis-session”特征的
		Actix_Session，故需要Redis，请确保服务器安装
		并启动了该服务后运行本程序，否则有关Cookie的
		处理机制将不会生效。
	*/
	let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379")
		.await
		.expect("链接Redis服务失败。你启动Redis了吗？");

	/*
		“随机密钥并不是指每一次访问数值都不同的密钥，
		Cookie解码时需要此密钥，故若密钥变化过快则可
		能会导致Cookie即刻失效。”

		上述言论是正确的，但它并不能解释密钥变化过快
		的原因。详见下文。
	*/
	let secret_key: Key = generate_keys();
	let max_book: usize = standard_sql::<usize>(Vec::new(), SQL_CMD_06.to_string(), Some(1)).expect("未提取到最大书目信息：")[0];
	let book_data: Vec<BookDiv> = (0..(max_book / 20 + 1))
		.map(|_| BookDiv::new())
		.collect::<Vec<BookDiv>>();
	let total_web_data: web::Data<Mutex<(usize, Vec<BookDiv>)>> = web::Data::new(
		Mutex::new(
			(max_book, book_data)
		)
	);

	// 第八代中央集成Closure
	let actix_serv = move || {
		/*
			如果将服务器状态定义在闭包中，其内部数据随时有可
			能被重新生成的状态所覆盖，正如Actix_Web的作者所说：
			
			“Note that Data should be constructed outside
			the HttpServer::new closure if shared, 
			potentially mutable state is desired. Data
			is cheap to clone; internally, it uses an Arc.”
			
			出于同样的原因，Cookie的密钥也不能定义在闭包中。
		*/
		App::new()
			.app_data(total_web_data.clone())
			.wrap(Logger::new(&mode))
			.wrap(
				SessionMiddleware::builder(
						redis_store.clone(),
						secret_key.clone()
					)
					.cookie_name("Sound".to_string())
					.cookie_secure(false)
					.cookie_http_only(true)
					.session_lifecycle(
						PersistentSession::default().session_ttl(time::Duration::days(3)),
					)
					.cookie_path("/".to_string())
					.cookie_content_security(CookieContentSecurity::Private)
					.build(),
			)
			.wrap(ErrorHandlers::new().default_handler(add_error_body))
			.route("/",					web::get().to( || get_closure(ROOT[0], "html")))
			.route("/style/root_style",	web::get().to( || get_closure(ROOT[0], "css")	))
			.route("/pic/ear_logo",		web::get().to( || get_closure(ROOT[1], "txt")	))
			.route("/pic/main_bg1",		web::get().to( || get_closure(ROOT[2], "txt")	))

			.route("/pic/404",			web::get().to( || get_closure(ACCESS[0], "txt")	))

			// login 部分
			/*
				原来这样的做法是不可取的，它将Cookie仅置于
				路由/user/user_login之下，而全站其他路由无
				法访问，这样做就失去验证的意义了。
			*/
			.service(
				web::resource("/user/user_login")
					.route(web::get().to(user_login))
					.route(web::post().to(user_login_post))
			)
			.route("/user/reset_passwd",			web::post().to(reset_passwd_1))
			.route("/user/style/user_login_style",	web::get().to( || get_closure(LOGIN[0], "css")	))
			.route("/user/js/md5_js",				web::get().to( || get_closure(LOGIN[1], "js")	))
			.route("/user/pic/ear_logo_w1",			web::get().to( || get_closure(LOGIN[2], "txt")	))
			.route("/user/pic/ear_logo_w2",			web::get().to( || get_closure(LOGIN[3], "txt")	))
			.route("/user/pic/ear_logo1",			web::get().to( || get_closure(LOGIN[4], "txt")	))
			.route("/user/pic/ear_logo2",			web::get().to( || get_closure(LOGIN[5], "txt")	))

			// regist 部分
			.service(
				web::resource("/user/user_regist")
					.route(web::get().to(user_regist))
					.route(web::post().to(user_regist_post))
			)

			.route("/user/option_note",				web::get().to( || get_closure(REGIST[1], "html")))
			.route("/user/pic/regist_bg",			web::get().to( || get_closure(REGIST[0], "txt")	))
			.route("/user/pic/bg04",				web::get().to( || get_closure(REGIST[2], "txt")	))

			// 邮件验证
			.service(
				web::scope("/user/email_checked/{random_id}")
					.service(web::resource("/regist_cofirm{path}").route(web::get().to(regist_cofirm)))
					.service(web::resource("/update_cofirm{path}").route(web::get().to(update_cofirm)))
					.service(
						web::resource("/rset_password{path}")
							.route(web::get().to(resetp_cofirm))
							.route(web::post().to(reset_passwd_2))
					)
			)

			// user 部分
			.route("/user/pic/author",					web::get().to( || get_closure(USER_INFO[0], "txt")	))
			.route("/user/pic/bg01",					web::get().to( || get_closure(USER_INFO[1], "txt")	))
			.route("/user/pic/img1",					web::get().to( || get_closure(USER_INFO[2], "txt")	))
			.route("/user/pic/img2",					web::get().to( || get_closure(USER_INFO[3], "txt")	))
			.route("/user/pic/ryb",						web::get().to( || get_closure(USER_INFO[4], "txt")	))
			.route("/user/pic/ssh",						web::get().to( || get_closure(USER_INFO[5], "txt")	))
			.route("/user/pic/sshw",					web::get().to( || get_closure(USER_INFO[6], "txt")	))
			.route("/user/pic/bg02",					web::get().to( || get_closure(USER_INFO[7], "txt")	))
			.route("/user/pic/userpl",					web::get().to( || get_closure(USER_INFO[8], "txt")	))
			.route("/user/pic/bg03",					web::get().to( || get_closure(USER_INFO[13], "txt")	))
			.route("/user/style/user_info",				web::get().to( || get_closure(USER_INFO[9], "css")	))

			/*
				“所有access目录下的主要页面都应进行身份
				验证，未来应设置集成函数：

				.route("/access",					web::get().to(authentication))
				”

				该思路已被废弃，本项目采用更加强大的PermyE
				中间件进行用户身份验证。
			*/

			// browse 部分
			/*
				与大多数网站不同，本网站传参均使用路径
				而不是Get参数。
			*/
			.route("/style/browse_style2",				web::get().to( || get_closure(BROWSE[1], "css")	))
			.route("/pic/main_bg2",						web::get().to( || get_closure(BROWSE[2], "txt")	))
			.route("/pic/down_line",					web::get().to( || get_closure(BROWSE[3], "txt")	))

			// 拟使用详细信息页面处理订阅和归还请求。
			.route("/book/style/browse_detail",			web::get().to( || get_closure(BROWSE[5], "css")	))
			.route("/book/pic/main_bg3",				web::get().to( || get_closure(BROWSE[6], "txt")	))

			// 公共部分
			.route("/pic/user",							web::get().to( || get_closure(PUBLIC[0], "txt")	))
			.route("/pic/ear_v3_icon",					web::get().to( || get_closure(PUBLIC[1], "ico")	))
			.route("/pic/cap",							web::get().to(cap_img_gen))

			// 邮件部分
			.route("/pic/ear.png",							web::get().to( || get_closure(EMAIL[0], "png")	))

			/*
				对每个网页均进行权限检查过于繁琐，也没
				有必要。
				是次版本使用PermyE中间件担当守卫，
				进行集中权限检查和部分路由管理。
			*/
			// Access集中检查部分
			.service(
				web::scope("/access")
					.wrap(PermyE)
					.service(web::resource(r"/browse_i{id:\d+}").route(					web::get().to(initialize_browse)))
					.service(web::resource(r"/browse_c{class}_page_{page:\d+}").route(	web::get().to(class_browse)))
					.service(web::resource(r"/pic/image_{id:\d+}").route(				web::get().to(get_book_images)))
					.service(
						web::resource(r"/book/details_b{id:\d+}")
							.route(web::get().to(details_browse))
							.route(web::put().to(put_details_browse))
							.route(web::delete().to(del_details_browse))
					)
					.service(web::resource(r"/user/user_info_d{id:\d+}").route(			web::get().to(deal_with_user)))
					.service(web::resource("/user/user_update").route(					web::post().to(update_user)))
					.service(web::resource("/user/user_logout").route(					web::get().to(user_logout)))
			)
	};

	let res: std::io::Result<()> = match builder
	{
		Some(tmp) => {
			let mut current_ip: String = ip[1].to_string();
			current_ip.push_str(":443");

			if let Err(e) = File::create(KEY_FILE)
			{
				println!("创建标志文件失败： {}: {}", KEY_FILE, e);
			}

			println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m 浮声将开启于 https://{} ...\x1b[0m\n\n\n", current_ip);
			HttpServer::new(actix_serv)
			.bind_openssl(current_ip, tmp)?
			.run()
			.await
		},
		None => {
			let mut current_ip: String = ip[1].to_string();
			current_ip.push_str(":80");
			println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m 浮声将开启于 http://{} ...\x1b[0m\n\n\n", current_ip);
			HttpServer::new(actix_serv)
			.bind(current_ip)?
			.run()
			.await
		},
	};

	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m 浮声已关闭。\x1b[0m");
	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m 清除环境。\x1b[0m");
	if let Err(e) = fs::remove_file(KEY_FILE)
	{
		println!("删除标志文件失败： {}: {}", KEY_FILE, e);
	}

	println!("[\x1b[1;32m+\x1b[0m] \x1b[1;34m Actix_web主函数 成功返回。\n\x1b[0m");
	res
}