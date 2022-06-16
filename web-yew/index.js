import './style.scss';

import('./pkg').then((module) => {
    let playHandle = null;
    const play_button = document.getElementById("play");
    play_button.addEventListener("click", async (event) => {
        playHandle = await module.play();
    });
    const stop_button = document.getElementById("stop");
    stop_button.addEventListener("click", (event) => {
        if (playHandle != null) {
            playHandle.free();
	        playHandle = null;
        }
    });
})

