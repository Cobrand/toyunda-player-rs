
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

function special_trim(s) {
	// TODO remove trailing "-"
	return s.trim();
}
function format_meta2name(meta_info) {
	var s = "" ;
	if (meta_info.media_title) {
		s += meta_info.media_title + " " ;
		if (meta_info.music_type) {
			var music_t = meta_info.music_type ;
			if (music_t.toLowerCase() == "ending") {
				s += "ED" ;
			} else if (music_t.toLowerCase() == "opening") {
				s += "OP" ;
			} else if (music_t.toLowerCase() == "insert") {
				s += "INS" ;
			} else {
				s += music_t.toUpperCase();
			}
			if (meta_info.music_number) {
				s += meta_info.music_number ;
			}
			if (meta_info.version) {
				if (isNaN(meta_info.version)) {
					s += " ("+meta_info.version+")"
				} else {
					s += "v" + meta_info.version ;
				}
			}
		}
	}
	if (meta_info.song_name) {
		if (s == "") {
			s = meta_info.song_name ;
		} else {
			s += " - " + meta_info.song_name ;
		}
	}
	if (s.length != 0) {
		if (meta_info.language) {
			if (meta_info.language == "Jp") {
				s = "[JAP] " + s;
			} else if (meta_info.language == "Eng") {
				s = "[ENG] " + s;
			} else if (meta_info.language == "Fr") {
				s = "[FR] " + s;
			} else {
				s = "[" + meta_info.language.uppercase() + "] " + s ;
			}
		}
	} else {
		return null ;
	}
	return s ;
}

function toyunda_command(command_type,id,response_fun,error_fun) {
	AJAX.post("/api/command",{
		command:command_type,
		id:id
	},function(s,a) {
		update();
		if (response_fun) {
			response_fun(s,a);
		}
	},error_fun);
}

function format_name(meta_info,video_path) {
	var candidate = format_meta2name(meta_info);
	if (candidate == null) {
		candidate = video_path.replace(/^.*[\\\/]/, '');
		candidate = candidate.split('.')[0];
	}
	return candidate ;
}

var vue = new Vue({
	el: '#app',
	data : {
		search : "",
		playlist : [],
		listing : [],
		currently_playing : null,
		draft_indexes : []
	},
	computed :{
		filtered_list: function() {
			var search = this.search ;
			var listing = this.listing;
			if (search != "") {
				listing = listing.filter(function(e) {
					return e.formatted_name.indexOf(search) !== -1
				});
			}
			return listing ;
		},
		now_playing: function() {
			return format_name(this.currently_playing.meta_info,this.currently_playing.video_path);
		},
		play_next_value: function() {
			if (this.currently_playing == null) {
				return "Commencer" ;
			} else {
				return "Suivant" ;
			}
		},
		play_next_disabled : function() {
			if (this.currently_playing == null && this.playlist.length == 0 ) {
				return true;
			} else {
				return false;
			}
		},
		stop_button_disabled : function() {
			if (this.currently_playing == null) {
				return true ;
			} else {
				return false ;
			}
		},
		draft : function() {
			var listing = this.listing ;
			return this.draft_indexes.map(function(e) {
				return listing[e];
			});
		}
	},
	methods : {
		format_name:format_name,
		format_meta2name:format_meta2name,
		add_to_queue:function(entry) {
			toyunda_command("add_to_queue",entry.index);
		},
		add_to_draft:function(entry) {
			this.draft_indexes.push(entry.index);
		}
	}
});

function update() {
	AJAX.get("/api/state",function(status,answer) {
		if (is_status_error(status)) {
			console.error("Error when retrieving state : "+answer);
		} else {
			var playing_state = answer.playing_state ;
			if (playing_state.playing) {
				vue.currently_playing = playing_state.playing;
			} else {
				vue.currently_playing = null;
			}
			var playlist = answer.playlist ;
			playlist = playlist.map(function(e,i) {
				e.formatted_name = format_name(e.meta_info,e.video_path);
				e.index = i ;
				return e;
			});
			vue.playlist = playlist ;
		}
	});
}

setInterval(update, 2000);

// retrieve the listing once
AJAX.get("/api/listing",function(status,answer) {
	if (is_status_error(status)) {
		console.error("Error "+status+" when retrieving listing : "+answer);
	} else {
		if (Array.isArray(answer)) {
			var len = answer.length ;
			vue.listing = answer.map(function(e) {
				return e.video_meta ;
			});
			for (var i = 0 ; i < len ; i++ ) {
				var entry = vue.listing[0] ;
				entry.formatted_name = format_name(entry.meta_info,entry.video_path);
				entry.index = i ;
			}
		} else {
			console.error("Error when updating listing ; answer is not an Array");
			console.error(answer);
		}
	}
});

update();
