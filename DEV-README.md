# Hacker Guide
aka "Letter to future self : help you understand what the fuck you've done"

## Introduction
This player is written in Rust. It might be hard to hack because of the design of the language (google Rust borrow checker for instance), but it is fairly easy to read. Well, the language is easy to read, but my code is mostly unreadable, so good luck with that.

This will help you understand some of the mess I've done.

## External libraries
This player uses SDL2 for the windowing interface (key handling, window resizing, ...), and mpv to display the video in the window itself. mpv is a standalone multimedia player, but it can also output to an OpenGL buffer. SDL2 happens to be able to use that buffer to display things, by sharing the same OpenGL context.
mpv has a very simple API made in C, hence its API in Rust (made in the mpv-rs crate), is fairly low-level stuff.

Basically, the mpv core player runs asynchronously at first, so every call you make can be synchronous of asynchronous. Every new frame of video, the core will answer what you've asked him (for instance the current position of the video, the size of the buffer, the level of the volume, ...), so you will either receive these answer asynchronously (via the event loop), or get an answer back from mpv in synchronous mode (meaning your program will hang until that core player answer). At some point you will have to ask the player to print to the dedicated OpenGL framebuffer, and the call will hang until mpv has printed its frame.

After that print, you will need to get the events from the SDL event loop and the mpv event loop, print your own subtitles and overlayy, and then ask SDL2 to render. If you have a look in `src/toyunda_player/toyunda.rs` in the method `main_loop`, that's exactly what's being done.

## Code Structure
At first, the code was messy and monolithic. Now the code is only half as messy, and way less monolithic, at the cost of some redundancy (There might be 3 structs `Color` in this program, in 3 separate folders).

`toyunda_player` is the main core of the player. It handles everything else, but mostly SDL2 inputs (what happens when I press the key V ?) and mpv events, which are for example, when a new video start or the video stops (stopping for an error, or because the file has reached its end). The core of this directory is located in `toyunda.rs`, where you will find the main structure. As of late October 2016, the structure looks like this :

```rust
pub struct ToyundaPlayer<'a> {
    pub subtitles: Option<Subtitles>,
    pub mpv: Box<MpvHandlerWithGl>,
    pub displayer: SDLDisplayer<'a>,
    pub options: ToyundaOptions,
    pub state: Arc<RwLock<State>>,
    pub manager: Option<Manager>,
    pub editor_state: Option<EditorState>,
    pub announcements: Vec<(String,DateTime<Local>)>,
    mpv_cache : MpvCache,
    unsaved_changes : bool,
}
```
Most of them are really straightforward, but we will still describe them as best as possible.

* `subtitles` is the structure for the subtitles. If subtitles is `None`, then subtitles failed to load somehow. Note that if subtitles failed to load, you will get an error on screen on normal mode, and the file will be skipped on Karaoke Mode. 
* `mpv` is the structure managing mpv. It allows to do things like `self.mpv.set_property("pause",true)`. See the mpv-rs crate for more info. 
* `displayer` the structure implementing a Displayer. We will get onto that later, but as of now the displayer is implemented using SDL2, SDL\_ttf, SDL2\_image, but there can probably be other implementations using Cairo instead of SDL2_ttf, for instance.
* `options` lists options that cannot be changed after the player is loaded.
* `state` lists options that can be changed after the player is loaded, and other things like the current playlist, ... Since it can be used by the Manager, which is an HTTP interface working in another thread, this variable has to be a Mutex or a Read-Write Lock.
* `manager` : the object managing the web interface. It should run on its own after it has been started. It contains the listing if there is any, and the object which handles the HTTP connection. When this object is dropped (meaning when this struct is dropped), the HTTP connection will stop listening.
* `editor_state` handles everything related to the editor, meaning whether of not a key is being held, whoch is the current syllable being timed (if any), ...
* `announcements` : Messages to be displayed as announcements. Announcements are a special case of messages. Most of the messages are displayed directly from the log API (things like `error!("Some error here")` will be displayed directly on screen if not in karaoke mode). Announcement doesn't have such a call, so it must be stored somewhere. This element is where it is stored.
* `mpv_cache`is a cache for mpv values. mpv calls aren't very costly, but they can wait for the next frame to answer if used synchronously. To avoid that, all of the values needed by the program (percent-pos, current-pos, ...) are queried right after mpv draw its frame, and are then stored in this cache. This cache can be used by the program at will, without the need for another mpv call.
* `unsaved_change` : if the Subtitles struct has been changed, but the corresponding subtitles file is unchanged, this variable is set to true. A warning will be displayed if the player is being quit while there are unsaved changed. You might think that this ought to be moved in editor, but the Subtitles can change without the editor, too! (See the keys J and K in normal and editor modes).

### Displaying Subtitles
Okay, we know how to display mpv, how to handle keys, call the editor to do stuff, handle the playlist, ... But we still don't know how to display the overlay, meaning the subtitles, the credits, the messages, ...
#### How to display stuff
You mainly need 2 things to display things :
* A displayer (in our case, `SDL_Displayer`, but the implementation isn't important in this case)
* An `OverlayFrame` object

the trait Display is defined like so :

```rust
pub trait Display {
    type Parameters;
	fn display(&mut self,&OverlayFrame,&Self::Parameters) -> Vec<Rect>;
}
```
Parameters is an object specific to the displayer. For instance, parameters to display in a subarea of the screen, ...
Basically, this function display takes an `OverlayFrame` and returns a `Vec` of `Rect`s.

But first, let's see what OverlayFrame is made of from:

```rust
pub struct OverlayFrame {
    pub text_units:Vec<TextUnit>
}

pub struct TextUnit {
    pub text: Vec<TextSubUnit>,
    pub size: Size,
    pub pos: (PosX, PosY),
    pub anchor: (f32, f32),
}

pub struct TextSubUnit {
    pub text: String,
    pub color: AlphaColor,
    pub outline: Outline,
    pub shadow: Option<Color>,
    pub attach_logo: bool,
}
```

Imagine `TextUnit` as an object describing a text, which must fit a certain `Size` (90% of the screen width maximum, 50px height maximum, ...), at a certain position. A text might be made of different styles. The first letter might have a different color than the rest, or a different border. This is made using multiple `TextSubUnit`, which only have graphic properties.

As such, multiple `TextUnit` can be used in a single Frame.

`TextUnit`, `OverlayFrame` and the like is self contained in its own directory, and must never use anything else that might use it (no cross references). This will allow modulable programming and easy coding.

### The Subtitles struct
The Subtitles struct is self contained, and can be directly parsed from a json file accordingly. The whole structure can be found in [this directory](src/subtitles). It should also be able to convert itself into an `OverlayFrame` given the position of the video.

