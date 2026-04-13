use criterion::{Criterion, criterion_group, criterion_main};
use resonix_graph::{
    implementations::{
        AudioBuffer, AudioBufferMut, ConstantNode, Graph, OutputNode, OutputNodePortDescriptors,
    },
    primitives::{AudioNodeCtx, BlockSize, CurrentTime, Sample, SampleRate},
    traits::{Graph as _, ModifyGraph},
};

fn create_and_run_constant_to_external_graph() {
    let channels = 1;
    let block_size = 256;
    let mut graph = Graph::with_block_size(block_size);

    let constant_node = ConstantNode::new_with_value(&mut graph, 1.0);
    let constant_node_handle = graph.add_audio_node(constant_node).unwrap();

    let output_node = OutputNode::new(&mut graph);
    let output_node_handle = graph.add_audio_node(output_node).unwrap();

    graph
        .connect(
            constant_node_handle.output_port_address(),
            output_node_handle.input_port_address(),
        )
        .unwrap();

    let inputs: [Option<AudioBuffer<'_>>; 0] = [];
    let inputs = inputs.as_slice();
    let mut raw_output_buffer = vec![Sample::default(); block_size * channels];
    let output_audio_buffer = AudioBufferMut::new(raw_output_buffer.as_mut_slice(), channels)
        .expect("should be able to make audio_buffer out of raw buffer");

    let output_external_connection_id = output_node_handle.external_output_connection_ids()
        [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
        .expect("should have an external connection id");

    let mut outputs: Vec<Option<AudioBufferMut<'_>>> = (0..=**output_external_connection_id)
        .map(|_| None)
        .collect();
    outputs[**output_external_connection_id] = Some(output_audio_buffer);

    let ctx = AudioNodeCtx::builder()
        .block_size(BlockSize::from(block_size))
        .sample_rate(SampleRate::default())
        .current_time(CurrentTime::default())
        .build();

    graph
        .run(inputs, &mut outputs, ctx)
        .expect("should be able to run the graph without errors");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("making & running Constant -> External nodes", |b| {
        b.iter(create_and_run_constant_to_external_graph)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
