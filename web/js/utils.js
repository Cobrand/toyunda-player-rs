function human_duration(time) {
	var minutes = Math.floor(time / 1000  / 60);
	var seconds = Math.floor(time / 1000) % 60;
	seconds = (seconds > 9) ? seconds : "0"+seconds;
	return minutes + "m" + seconds;
}

function is_status_error(status) {
	return status < 200 || status >= 400 ;
}

var AJAX = {
	get : function(url,response_fun,error_fun) {
		var request = new XMLHttpRequest();
		request.open('GET',url, true);
		request.onload = function() {
			let responseText = JSON.parse(request.responseText) ;
			if (responseText == null) {
				responseText = request.responseText ;
			}
			response_fun(request.status,responseText);
		};
		request.onerror = function() {
			if (error_fun) {
				error_fun();
			}
		};
		request.send();
	},
	// parameters must be an object
	post : function(url,parameters,response_fun,error_fun) {
		var request = new XMLHttpRequest();
		request.open('POST',url, true);
		request.setRequestHeader("Content-Type", "application/json");
		request.onload = function() {
			if (response_fun) {
				response_fun(request.status,request.responseText);
			}
		};
		request.onerror = function() {
			if (error_fun) {
				error_fun();
			}
		};
		request.send(JSON.stringify(parameters));
	}
}

function shuffle(a) {
	var j, x, i;
	for (i = a.length; i; i--) {
		j = Math.floor(Math.random() * i);
		x = a[i - 1];
		a[i - 1] = a[j];
		a[j] = x;
	}
}

function special_trim(s) {
	// TODO remove trailing "-"
	return s.trim();
}
