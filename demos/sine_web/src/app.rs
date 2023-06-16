use std::ops::Mul;

use leptos::{component, view, IntoView, Scope, spawn_local, create_signal, SignalSet};
use resonix::Sine;

use crate::audio::set_up_audio_stream;

const NUM_SAMPLES: u32 = 1000;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (_, set_audio) = create_signal(cx, None);

    let mut visual_sine = Sine::new();
    visual_sine.set_frequency(1.0).set_sample_rate(NUM_SAMPLES);
    let mut bars = Vec::with_capacity(NUM_SAMPLES as usize);
    for _ in 0..270 {
        bars.push(visual_sine.next_sample());
    }

    visual_sine.set_frequency(3.12);

    for _ in 270..510 {
        bars.push(visual_sine.next_sample());
    }

    visual_sine.set_frequency(51.0);

    for _ in 510..830 {
        bars.push(visual_sine.next_sample());
    }

    visual_sine.set_frequency(1.0);

    for _ in 830..1000 {
        bars.push(visual_sine.next_sample());
    }
     
     let bars = bars.into_iter().map(|amplitude| {
        let height = format!("{}%", amplitude.abs().mul(50.0));
        let translate_y = format!("{}%", amplitude.signum().mul(50.0));
        let width = format!("{}%", 1.0 / NUM_SAMPLES as f32 * 100.0);
        view! {
            cx,
            <div style={format!("height: {height}; background-color: black; width: {width}; transform: translateY({translate_y});", )} />
        }
     }).collect::<Vec<_>>();

    view! { cx,
        <p>"Sine wave: "</p>
        <button
            on:click=move |_| {
                spawn_local(async move {
                    let audio_out = set_up_audio_stream().await;
                    set_audio.set(Some(audio_out));
                });
            }
        >
            "Start"
        </button>
        <button
            on:click=move |_| {
              set_audio.set(None);
            }
        >
            "Stop"
        </button>
        <div style="width: 100%; height: 100px; background-color: lightblue; display: flex; justify-content: center; align-items: center;">
            {bars}
        </div>
    }
}
