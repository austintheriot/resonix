use resonix::{
    graph::implementations::{OutputNode, SineNode},
    implementations::{Graph, cpal::CpalAudioOutput, ringbuf::RingbufChannel},
    traits::SystemAudioOutput,
};

fn main() {
    let mut graph = Graph::with_block_size(64);

    // TODO: remove this. Pre-initializing is just for testing on web
    let sine_node = SineNode::new_with_frequency(&mut graph, 440.0);
    let output_node = OutputNode::new(&mut graph);

    let sine_node_handle = graph.add_audio_node(sine_node).unwrap();
    let output_node_handle = graph.add_audio_node(output_node).unwrap();

    graph
        .connect(
            sine_node_handle.output_port_address(),
            output_node_handle.input_port_address(),
        )
        .unwrap();

    let mut audio_output = CpalAudioOutput::new::<RingbufChannel>();

    let sample_rate = audio_output.config().sample_rate.0;
    let mut next_value = move || {
        graph.run();
    };

    // just fill the buffer on every loop
    loop {
        while audio_output.ready_for_sample() {
            audio_output.try_write_sample(next_value()).unwrap();
        }
    }
}
