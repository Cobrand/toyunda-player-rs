function human_since(timestamp){
	if (timestamp == null) {
		return "Jamais";
	}
	var cur_timestamp = Date.now() / 1000;
	var delta = (cur_timestamp - timestamp) / 3600;
	if (delta < 1) {
		return "< 1h";
	} else if (delta < 48) {
		return Math.floor(delta)+"h";
	} else {
		return Math.floor(delta / 24) + "j";
	}
}

function sum_duration_to_string(list) {
	var i = 0;
	var len = list.length;
	var total_duration = 0;
	var unsure = false;
	for (i = 0; i < list.length; i++){
		var el = list[i];
		if (el.video_duration == 0) {
			unsure = true;
		} else {
			total_duration += el.video_duration;
		}
	};
	return (unsure ? ">" : "") + human_duration(total_duration);
}

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
			if (song_info.language == "JAP") {
				s = "[JAP] " + s;
			} else if (song_info.language == "ENG") {
				s = "[ENG] " + s;
			} else if (song_info.language == "FR") {
				s = "[FR] " + s;
			} else {
				s = "[" + song_info.language.toUpperCase() + "] " + s ;
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
	return candidate;
}

function format_fullinfo(video_meta){
	return format_name(video_meta.song_info,video_meta.video_path)
		+ " ["+ (video_meta.video_duration == 0 ? "Durée inconnue":human_duration(video_meta.video_duration))+"]"
}

var vue = new Vue({
	el: '#app',
	data : {
		screen_size: "small", // small, large or xlarge
		panel: 0,
		search : "",
		playlist : [],
		listing : [],
		currently_playing : null,
		draft_indexes : [],
		announcement_message: "",
		connected:true
	},
	computed :{
		filtered_list: function() {
			var search = this.search ;
			var listing = this.listing;
			if (search != "") {
				var searches = search.match(/\S+/g);
				if (searches != null) {
					for (var i = 0; i < searches.length ; i++) {
						var search_regexp = new RegExp(searches[i],'i');
						listing = listing.filter(function(e) {
							return search_regexp.test(e.formatted_name);
						});
					}
				}
			}
			return listing ;
		},
		now_playing: function() {
			return format_fullinfo(this.currently_playing);
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
		},
		draft_duration : function() {
			if (this.draft_indexes.length == 0){
				return null;
			} else {
				return sum_duration_to_string(this.draft);
			}
		},
		playlist_duration : function() {
			if (this.playlist.length == 0){
				return null;
			}
			return sum_duration_to_string(this.playlist);
		},
		panel_half : function() {
			return this.panel == 0;
		},
		panel_class : function() {
			return {
				size_small:this.screen_size == "small",
				size_large:this.screen_size == "large",
				size_xlarge:this.screen_size == "xlarge"
			}
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
			AJAX.post("/api/command",{
				command:"add_to_queue",
				id:this.draft_indexes[index],
				pos:0
			},function(){
				this.draft_indexes.splice(index, 1);
			}.bind(this));
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
		queue_delete_at:function(index) {
			AJAX.post("/api/command",{
				command:"delete_from_queue",
				pos:index
			});
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
		},
		draft_add_random:function(){
			var index = Math.floor(Math.random() * this.listing.length);
			this.draft_indexes.push(index);
		},
		draft_remove_last:function(){
			if (this.draft_indexes.length > 0) {
				this.draft_indexes.pop()
			}
		},
		set_panel:function(i){
			this.panel = i;
		}
	}
});

document.addEventListener("keypress",function(event){
	if (event.charCode == 97) {
		vue.draft_add_random();
	} else if (event.charCode == 120) {
		vue.draft_remove_last();
	}
})

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
				e.formatted_fullinfo = format_fullinfo(e);
				e.human_duration = human_duration(e.video_duration);
				e.human_last_played = human_since(e.last_played);
				e.index = i;
				return e;
			});
			vue.playlist = playlist ;
		}
		vue.connected = true;
	},function(){
		vue.connected = false;
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
				var entry = answer[i];
				entry.search_string = "";
				entry.search_string += (entry.artist || "") + " ";
				entry.search_string += (entry.year || "") + " ";
				entry.search_string += (entry.language || "") + " ";
				if (entry.alt_media_titles) {
					entry.alt_media_titles.forEach(function(e) {
						entry.search_string += e + " ";
					});
				}
				entry.search_string += format_name(entry.song_info,entry.video_path);
				entry.formatted_name = format_name(entry.song_info,entry.video_path);
				entry.index = i;
				entry.human_duration = human_duration(entry.video_duration);
				entry.formatted_fullinfo = format_fullinfo(entry);
				entry.human_last_played = human_since(entry.last_played);
			}
			vue.listing = answer
		} else {
			console.error("Error when updating listing ; answer is not an Array");
			console.error(answer);
		}
	}
});

update();
