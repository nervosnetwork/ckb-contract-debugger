use core::script::Script;
use core::transaction::{CellInput, CellOutput, Transaction};
use flatbuffers::{FlatBufferBuilder, WIPOffset};
use protocol::{
    Bytes as FbsBytes, CellInput as FbsCellInput, CellInputBuilder, CellOutput as FbsCellOutput,
    CellOutputBuilder, OutPoint as FbsOutPoint, Transaction as FbsTransaction, TransactionBuilder,
};
use serde_json::from_slice;

pub fn build_tx<'b>(
    fbb: &mut FlatBufferBuilder<'b>,
    tx: &Transaction,
) -> WIPOffset<FbsTransaction<'b>> {
    let vec = tx
        .deps()
        .iter()
        .map(|out_point| FbsOutPoint::build(fbb, out_point))
        .collect::<Vec<_>>();
    let deps = fbb.create_vector(&vec);

    let vec = tx
        .inputs()
        .iter()
        .map(|cell_input| build_input(fbb, cell_input))
        .collect::<Vec<_>>();
    let inputs = fbb.create_vector(&vec);

    let vec = tx
        .outputs()
        .iter()
        .map(|cell_output| build_output(fbb, cell_output))
        .collect::<Vec<_>>();
    let outputs = fbb.create_vector(&vec);

    let mut builder = TransactionBuilder::new(fbb);
    builder.add_version(tx.version());
    builder.add_deps(deps);
    builder.add_inputs(inputs);
    builder.add_outputs(outputs);
    builder.finish()
}

fn build_output<'b>(
    fbb: &mut FlatBufferBuilder<'b>,
    output: &CellOutput,
) -> WIPOffset<FbsCellOutput<'b>> {
    let lock = FbsBytes::build(fbb, &output.lock);
    let mut builder = CellOutputBuilder::new(fbb);
    builder.add_capacity(output.capacity);
    builder.add_lock(lock);
    builder.finish()
}

fn build_input<'b>(
    fbb: &mut FlatBufferBuilder<'b>,
    input: &CellInput,
) -> WIPOffset<FbsCellInput<'b>> {
    let hash = FbsBytes::build(fbb, &input.previous_output.hash);
    let mut builder = CellInputBuilder::new(fbb);
    builder.add_hash(hash);
    builder.add_index(input.previous_output.index);
    builder.finish()
}

// Convert JSON based tx data into flatbuffer based data
pub fn convert_tx(json_bytes: &[u8]) -> Vec<u8> {
    let t: Transaction = from_slice(json_bytes).unwrap();

    let mut tx_builder = FlatBufferBuilder::new();
    let tx_offset = build_tx(&mut tx_builder, &t);
    tx_builder.finish(tx_offset, None);

    tx_builder.finished_data().to_vec()
}

// Convert JSON based script data into script object
pub fn parse_script(json_bytes: &[u8]) -> Script {
    from_slice(json_bytes).unwrap()
}
