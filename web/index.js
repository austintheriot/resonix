import './style.scss';

import('./pkg').then((module) => {
    let handle = null;
    const play_button = document.getElementById("play");
    play_button.addEventListener("click", (event) => {
        handle = module.beep();
    });
    const stop_button = document.getElementById("stop");
    stop_button.addEventListener("click", (event) => {
        if (handle != null) {
            handle.free();
	        handle = null;
        }
    });
})

