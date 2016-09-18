# toyunda-player-rs

# Commands

## General

* Space : Pause / unpause
* F : Fullscreen
* numpad0-9 : speed from 1.0 to 0.1
* Shift numpad0-9 : speed from 1.1 to 2.0
* V : Hide/dhows subtitles
* Left / Right : -/+ 3 seconds,
* Shift + Left / Right : -/+ 15 seconds
* Alt + Left / Right : previous / next frame
* Mousewheel : volume

## Edit mode

* E : enable edit mode
* X / C : time
*  J / K : -/+ 10 ms on current syllable's begin time
* Alt J / Alt K : -/+ 10 ms on current syllable's end time
* Shift J / Shift K : -/+ 10 ms on current syllable's time (begin + end)
* Alt + Shift J / K : -/+ 10 ms for the entire subtitles
* S : Save subtitles into the json file
* R : Reload the json file (discard any changes if you might have done in the player)


# Installation

As of 09-2016, Rust nightly is needed for this to build correctly. It is required because of Serde,
whohc uses a config opion which is available in nightly only. See [The official webiste](https://www.rust-lang.org/)
for a guide on how to install rust for your distribution. This software needs some additional packages, the full list is :

* libmpv (for the video player)
* libsdl2 (for key handling + graphic displayer)
* libsdl2\_ttf
* libsdl2\_image

## Windows

### mpv

Go to the website https://mpv.srsfckn.biz/ and download the "Dev" link of the lastest version of mpv.

Go then place under `C:\\Programs\\Rust Nightly GNU 1.13\\lib\\rustlib\\x86\_64-pc-windows-gnu\\lib` the files 
*.dll for your architecture from the zip you downloaded from mpv. If you don't know your architecture, it's probably 64bits.

### sdl2

https://www.libsdl.org/release/SDL2-2.0.4-win32-x64.zip

Copy all the .dll files under the same as before `rustlib\\lib` folder. Same for sdl2\_ttf and sdl2\_image below.

### sdl2\_tff

https://www.libsdl.org/projects/SDL_ttf/release/SDL2_ttf-2.0.14-win32-x64.zip

### sdl2\_image

https://www.libsdl.org/projects/SDL_image/release/SDL2_image-2.0.1-win32-x64.zip

## Linux

For linux packages libmpv, libsdl2 and libsdl2\_ttf are required for this to work correctly.

## OS X 

?
