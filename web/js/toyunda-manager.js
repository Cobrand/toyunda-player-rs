function format_info2name(song_info) {
	var s = "" ;
	if (song_info.media_title) {
		s += song_info.media_title + " " ;
		if (song_info.music_type) {
			var music_t = song_info.music_type ;
			if (music_t.toLowerCase() == "ending") {
				s += "ED" ;
			} else if (music_t.toLowerCase() == "opening") {
				s += "OP" ;
			} else if (music_t.toLowerCase() == "insert") {
				s += "INS" ;
			} else {
				s += music_t.toUpperCase();
			}
			if (song_info.music_number) {
				s += song_info.music_number ;
			}
			if (song_info.version) {
				if (isNaN(song_info.version)) {
					s += " ("+song_info.version+")"
				} else {
					s += "v" + song_info.version ;
				}
			}
		}
	}
	if (song_info.song_name) {
		if (s == "") {
			s = song_info.song_name ;
		} else {
			s += " - " + song_info.song_name ;
		}
	}
	if (s.length != 0) {
		if (song_info.language) {
			if (song_info.language == "Jp") {
				s = "[JAP] " + s;
			} else if (song_info.language == "Eng") {
				s = "[ENG] " + s;
			} else if (song_info.language == "Fr") {
				s = "[FR] " + s;
			} else {
				s = "[" + song_info.language.uppercase() + "] " + s ;
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

function format_name(song_info,video_path) {
	var candidate = format_info2name(song_info);
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
		draft_indexes : [],
		announcement_message: ""
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
			return format_name(this.currently_playing.song_info,this.currently_playing.video_path);
		},
		play_next_value: function() {
			if (this.currently_playing == null) {
				return "Commencer";
			} else {
				return "Suivant";
			}
		},
		draft_panel_disabled : function(){
			return this.draft_indexes < 1
		},
		play_next_disabled : function() {
			return (this.currently_playing == null && this.playlist.length == 0);
		},
		stop_button_disabled : function() {
			return this.currently_playing == null;
		},
		announcement_button_disabled : function(){
			return this.announcement_message.length <= 0;
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
		format_info2name:format_info2name,
		draft_el_up:function(index) {
			if (index > 0) {
				var to_be_replaced = this.draft_indexes[index - 1];
				Vue.set(this.draft_indexes, index - 1, this.draft_indexes[index]);
				Vue.set(this.draft_indexes, index, to_be_replaced);
			}
		},
		draft_el_down:function(index) {
			if (index < this.draft_indexes.length - 1) {
				var to_be_replaced = this.draft_indexes[index + 1];
				Vue.set(this.draft_indexes, index + 1, this.draft_indexes[index]);
				Vue.set(this.draft_indexes, index, to_be_replaced);
			}
		},
		draft_transfer_beginning:function(index) {
			toyunda_command("add_to_queue",this.draft_indexes[index],function(){
				this.draft_indexes.splice(index,1);
			}.bind(this))
		},
		draft_transfer_single:function(index) {
			toyunda_command("add_to_queue",this.draft_indexes[index],function(){
				this.draft_indexes.splice(index,1);
			}.bind(this))
		},
		draft_delete:function(index){
			this.draft_indexes.splice(index,1);
		},
		add_to_queue:function(entry) {
			toyunda_command("add_to_queue",entry.index);
		},
		add_to_draft:function(entry) {
			this.draft_indexes.push(entry.index);
		},
		play_next:function() {
			toyunda_command("play_next");
		},
		stop_current:function() {
			toyunda_command("stop");
		},
		clear_queue:function() {
			toyunda_command("clear_queue");
		},
		toggle_subtitles:function() {
			toyunda_command("toggle_subtitles");
		},
		quit:function() {
			swal({
				title: 'Quitter ?',
				text: "Le lecteur se fermera",
				type: 'warning',
				showCancelButton: true,
				confirmButtonColor: '#3085d6',
				cancelButtonColor: '#d33',
				confirmButtonText: 'Oui'
			}).then(function() {
				toyunda_command("quit");
			})
		},
		quit_on_finish:function() {
			toyunda_command("quit_on_finish");
		},
		pause_after_next:function() {
			toyunda_command("quit_on_finish");
		},
		draft_shuffle:function() {
			shuffle(this.draft_indexes);
			this.draft_indexes.push(-1);
			this.draft_indexes.pop(); // <^notify Vue of a change
		},
		draft_transfer:function(){
			AJAX.post("/api/command",{
				command:"add_multiple_to_queue",
				list:this.draft_indexes
			},function(){
				this.draft_indexes.splice(0);
			}.bind(this));
		},
		send_announcement:function(){
			AJAX.post("/api/command",{
				command:"announcement",
				text:this.announcement_message
			},function(){
				this.announcement = "";
			}.bind(this))
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
				e.formatted_name = format_name(e.song_info,e.video_path);
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
			for (var i = 0 ; i < len ; i++ ) {
				var entry = answer[i] ;
				entry.formatted_name = format_name(entry.song_info,entry.video_path);
				entry.index = i;
			}
			vue.listing = answer
		} else {
			console.error("Error when updating listing ; answer is not an Array");
			console.error(answer);
		}
	}
});

update();
