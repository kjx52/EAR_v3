// email_send.rs
// 本文件包含用户注册处理函数

/*
	模块重要资源列表

	*列表顺序按定义顺序排列（不包含结构体成员顺序）。

#==============================#
	本模块定义的函数有：
行号	是否公有	名称				参数							返回值
----    --------    ----				----							------
37		private		gen_rand_string		&'a str, i32					String
70		pub			e_mail_sender		&'a str, &'b str, i32			bool

*/

use askama::Template;
use lettre_email::{Email, EmailBuilder};
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};
use rand::{rng, Rng};
use rand::distr::Alphanumeric;
use std::path::Path;
use crate::ear_v3_config::KEY_FILE;
use crate::ear_v3_struct::EmailDiv01;
use super::basic_fn::{redis_set_fn, redis_expire_fn};

// 邮件设置
const SOSR: &'static str = "";		// 你的邮件地址
const SMTP: &'static str = "";		// 邮件 smtp 服务器的地址
const PAWD: &'static str = "";		// 邮件授权码

const HTTP: &'static str = r##""##;			// 你的服务器 HTTP 模式下的地址
const HTTPS: &'static str = r##""##;		// 你的服务器 HTTPS 模式下的地址

async fn gen_rand_string<'a>(user: &'a str, mode: i32) -> String
{
	let row_string: String = rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect::<String>();
	let name: String = row_string[..16].to_string();
	redis_set_fn(name.clone(), user.to_string()).await;
	redis_expire_fn(name, 5*60).await;

	let url: String = match Path::new(KEY_FILE).try_exists()
	{
		Ok(t) => match t
		{
			true => HTTPS.to_string(),
			false => HTTP.to_string(),
		},
		Err(e) => {
			println!("由于{e}，确定标志文件存在与否，默认HTTPS模式。");
			HTTPS.to_string()
		},
	};

	match mode
	{
		1 => format!("{url}/user/email_checked/{}/regist_cofirm{}", &row_string[16..], &row_string[..16]),
		2 => format!("{url}/user/email_checked/{}/update_cofirm{}", &row_string[16..], &row_string[..16]),
		3 => format!("{url}/user/email_checked/{}/rset_password{}", &row_string[16..], &row_string[..16]),
		_ => "Error_Mode".to_string()
	}
}

pub async fn e_mail_sender<'a, 'b>(target: &'a str, user: &'b str, mode: i32) -> bool
{
	let email: EmailBuilder = Email::builder()
		.to(target)
		.from(SOSR);

	let email = match mode
	{
		1 => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 发送注册验证邮件。\x1b[0m");
			match email.subject("浮声 注册确认邮件")
				.html(EmailDiv01{
						div1: "注册确认".to_string(),
						div2: "注册账户".to_string(),
						div3: "注册".to_string(),
						div4: "完成注册".to_string(),
						account_confirmation: gen_rand_string(user, mode).await
					}
					.render()
					.expect("Email渲染失败：")
				)
				.build()
			{
				Ok(t) => t,
				Err(e) => {
					println!("    [\x1b[31mX\x1b[0m] \x1b[34m 新建邮件失败：{e}\x1b[0m");
					return false;
				}
			}
		},
		2 => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 发送账户信息更新邮件。\x1b[0m");
			match email.subject("浮声 账户信息更新邮件")
				.html(EmailDiv01{
						div1: "账户信息更新".to_string(),
						div2: "更新账户".to_string(),
						div3: "更新".to_string(),
						div4: "完成更新".to_string(),
						account_confirmation: gen_rand_string(user, mode).await
					}
					.render()
					.expect("Email渲染失败：")
				)
				.build()
			{
				Ok(t) => t,
				Err(e) => {
					println!("    [\x1b[31mX\x1b[0m] \x1b[34m 新建邮件失败：{e}\x1b[0m");
					return false;
				}
			}
		},
		3 => {
			println!("    [\x1b[34m*\x1b[0m] \x1b[34m 发送账户信息更新邮件。\x1b[0m");
			match email.subject("浮声 账户信息更新邮件")
				.html(EmailDiv01{
						div1: "重置密码".to_string(),
						div2: "重置账户密码".to_string(),
						div3: "操作".to_string(),
						div4: "重置密码".to_string(),
						account_confirmation: gen_rand_string(user, mode).await
					}
					.render()
					.expect("Email渲染失败：")
				)
				.build()
			{
				Ok(t) => t,
				Err(e) => {
					println!("    [\x1b[31mX\x1b[0m] \x1b[34m 新建邮件失败：{e}\x1b[0m");
					return false;
				}
			}
		},
		_ => {
			println!("    [\x1b[31mX\x1b[0m] \x1b[34m 未知模式。\x1b[0m");
			return false;
		},
	};

	let tmp2 = Credentials::new(
		SOSR.to_string(),
		PAWD.to_string(),
	);

	let mut mailer = SmtpClient::new_simple(SMTP)
	.expect("新建SMTP服务器失败。")
	.credentials(tmp2)
	.transport();

	if let Err(e) = mailer.send(email.into())
	{
		println!("    [\x1b[31mX\x1b[0m] \x1b[34m 发送邮件失败：{e}。\x1b[0m");
		return false;
	}

	mailer.close();
	true
}