use bdk_bitcoind_rpc::bitcoincore_rpc::{self, RpcApi};
use bdk_testenv::{anyhow, TestEnv};
use bitcoin::{hashes::Hash, Address, Amount, ScriptBuf, WScriptHash};
use std::time::Instant;

/// Measures the marginal cost of the `compute_txid()` verification against the
/// `get_raw_transaction` fetch it accompanies, using a real regtest node.
#[allow(clippy::print_stdout)]
#[test]
fn measure_txid_check_cost() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let client = bitcoincore_rpc::Client::new(
        &env.bitcoind.rpc_url(),
        bitcoincore_rpc::Auth::CookieFile(env.bitcoind.params.cookie_file.clone()),
    )?;
    env.mine_blocks(500, None)?;

    let addr = Address::from_script(
        &ScriptBuf::new_p2wsh(&WScriptHash::all_zeros()),
        bitcoin::Network::Regtest,
    )?;
    let mut txids = Vec::new();
    for _ in 0..500 {
        match env.send(&addr, Amount::from_sat(1_000)) {
            Ok(txid) => txids.push(txid),
            Err(_) => break, 
        }
    }
    let n = txids.len();
    assert!(n > 0);

    // Fetch phase: exactly what mempool_at pays per new tx (RPC + hex + deserialize).
    let t = Instant::now();
    let mut txs = Vec::with_capacity(n);
    for txid in &txids {
        txs.push(client.get_raw_transaction(txid, None)?);
    }
    let rpc = t.elapsed();

    // The verification we added.
    let t = Instant::now();
    let mut mismatches = 0usize;
    for (txid, tx) in txids.iter().zip(&txs) {
        if tx.compute_txid() != *txid {
            mismatches += 1;
        }
    }
    let check = t.elapsed();

    println!("\n==== txid-check perf (n={n} mempool txs, localhost regtest) ====");
    println!("get_raw_transaction : {rpc:?}  ({:.1} us/tx)", rpc.as_secs_f64() * 1e6 / n as f64);
    println!("txid check          : {check:?}  ({:.3} us/tx)", check.as_secs_f64() * 1e6 / n as f64);
    println!("check as % of fetch : {:.3}%  (mismatches={mismatches})",
        check.as_secs_f64() / rpc.as_secs_f64() * 100.0);
    Ok(())
}
