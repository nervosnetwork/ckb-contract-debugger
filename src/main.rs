extern crate bigint;
extern crate ckb_core as core;
extern crate ckb_script as script;
extern crate clap;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use bigint::H256;
use clap::{App, Arg};
use core::cell::{CellStatus, CellProvider};
use core::header::Header;
use core::transaction::{OutPoint, Transaction};
use jsonrpc_client_http::HttpTransport;
use script::TransactionScriptsVerifier;
use serde_json::from_slice;
use std::fs::File;
use std::io::{self, Read};

fn main() {
    let matches = App::new("debugger")
        .about("CKB VM debugger")
        .arg(
            Arg::with_name("rpc")
                .short("r")
                .long("rpc")
                .help("ckb rpc address")
                .takes_value(true),
        ).arg(
            Arg::with_name("transaction")
                .short("t")
                .long("transaction")
                .help("transaction filename, stdin will be used when missing")
                .takes_value(true),
        ).get_matches();

    let mut buffer = Vec::new();
    match matches.value_of("transaction") {
        Some(filename) => {
            let mut file = File::open(filename).unwrap();
            file.read_to_end(&mut buffer)
        }
        None => io::stdin().read_to_end(&mut buffer),
    }.unwrap();
    let tx: Transaction = from_slice(&buffer).unwrap();

    let rpc_address = matches.value_of("rpc").unwrap_or("http://localhost:8114");
    let provider = RpcCellProvider::new(rpc_address);
    let resolved_tx = provider.resolve_transaction(&tx);

    let verifier = TransactionScriptsVerifier::new(&resolved_tx);
    let result = verifier.verify();
    println!("Result: {:?}", result);
}

#[derive(Deserialize)]
pub struct TransactionWithHash {
    pub hash: H256,
    pub transaction: Transaction,
}

jsonrpc_client!(pub struct CkbClient {
    pub fn get_transaction(&mut self, h: H256) -> RpcRequest<Option<TransactionWithHash>>;

    pub fn get_tip_header(&mut self) -> RpcRequest<Header>;
});


pub struct RpcCellProvider<'a> {
    address: &'a str,
    // With a pre-defined tip, we can ensure atomicity when fetching all
    // cells.
    tip: H256,
}

impl<'a> RpcCellProvider<'a> {
    fn new(address: &'a str) -> RpcCellProvider {
        let transport = HttpTransport::new().standalone().unwrap();
        let transport_handle = transport
            .handle(address)
            .unwrap();
        let mut client = CkbClient::new(transport_handle);
        let tip_header = client.get_tip_header().call().expect("tip header must exist!");
        RpcCellProvider {
            address,
            tip: tip_header.hash(),
        }
    }
}

impl<'a> CellProvider for RpcCellProvider<'a> {
    fn cell(&self, out_point: &OutPoint) -> CellStatus {
        self.cell_at(out_point, &self.tip)
    }

    fn cell_at(&self, out_point: &OutPoint, _parent: &H256) -> CellStatus {
        let transport = HttpTransport::new().standalone().unwrap();
        let transport_handle = transport
            .handle(self.address)
            .unwrap();
        let mut client = CkbClient::new(transport_handle);

        // TODO: right now this solution cannot tell if a cell has been
        // spent, we will need a new RPC method in CKB to solve this.
        let tx = client.get_transaction(out_point.hash).call().unwrap_or(None);
        match tx {
            Some(tx) => tx.transaction.outputs().get(out_point.index as usize).map(|cell_output| CellStatus::Current(cell_output.clone())).unwrap_or(CellStatus::Unknown),
            None => CellStatus::Unknown,
        }
    }
}
