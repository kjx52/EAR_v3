function post_req(uname, passwd, remark_place) {
	fetch('/user/user_login',{
		method: 'POST',
		headers: {
			'Content-Type': 'application/json;charset=utf-8;',
			'Cache-Control': 'max-age=0',
			'Upgrade-Insecure-Requests': '1'
		},
		body: JSON.stringify({ username: `${uname}`, password: `${passwd}` })
		// body: `username=${uname}&password=${passwd}`
	})
	.then(response => {
		console.log(response);
		if (response.ok) {
			alert('登陆成功。');
			window.location.replace(remark_place);
		} else if (response.status == 400) {
			alert('登录过于频繁，请5分钟后再登录。');
		} else {
			alert('用户名或密码错误。');
			location.reload();
		}
	})
	.catch(error => console.error('Error:', error))
}