use resonix::{AudioContext, ConstantNode, DACNode};

#[tokio::test]
async fn example() {
    let mut audio_context = AudioContext::new();

    let constant_node= ConstantNode::new();
    let constant_node_handle = audio_context.add_node(constant_node).await.unwrap();
    let dac_node = DACNode::new();
    let dac_node_handle = audio_context.add_node(dac_node).await.unwrap();
    audio_context.connect(&constant_node_handle, &dac_node_handle, ()).await.unwrap();
}