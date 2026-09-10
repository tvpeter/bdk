use bitcoin::consensus::encode;
use bitcoin::hashes::Hash;
use bitcoin::{
    absolute::LockTime, transaction::Version, Amount, OutPoint, ScriptBuf, Sequence, Transaction,
    TxIn, TxOut, Txid, Witness,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn make_tx(n_in: usize, n_out: usize) -> Transaction {
    let mut w = Witness::new();
    w.push(vec![0u8; 72]);
    w.push(vec![0u8; 33]);
    Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: (0..n_in)
            .map(|k| TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_byte_array([k as u8 + 1; 32]),
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: w.clone(),
            })
            .collect(),
        output: (0..n_out)
            .map(|k| TxOut {
                value: Amount::from_sat(1000 + k as u64),
                script_pubkey: ScriptBuf::from(vec![0u8; 22]),
            })
            .collect(),
    }
}

fn bench(c: &mut Criterion) {
    for (label, i, o) in [
        ("small_1in_2out", 1, 2),
        ("typical_2in_2out", 2, 2),
        ("large_10in_10out", 10, 10),
    ] {
        let tx = make_tx(i, o);
        let ser = encode::serialize(&tx);
        let mut g = c.benchmark_group(label);
        g.bench_function("compute_txid", |b| {
            b.iter(|| black_box(black_box(&tx).compute_txid()))
        });
        g.bench_function("deserialize", |b| {
            b.iter(|| {
                let t: Transaction = encode::deserialize(black_box(&ser)).unwrap();
                black_box(t);
            })
        });
        g.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
