use expander_rs::{utils::*, FieldType};
use expander_rs::{
    BN254ConfigKeccak, BN254ConfigSha2, Circuit, CircuitLayer, Config, GF2ExtConfigKeccak,
    GF2ExtConfigSha2, GKRConfig, GKRScheme, GateAdd, GateMul, M31ExtConfigKeccak, M31ExtConfigSha2,
    Prover, Verifier,
};
use std::panic;
use std::panic::AssertUnwindSafe;

use rand::Rng;
use sha2::Digest;

#[allow(dead_code)]
fn gen_simple_circuit<C: GKRConfig>() -> Circuit<C> {
    let mut circuit = Circuit::default();
    let mut l0 = CircuitLayer::default();
    l0.input_var_num = 2;
    l0.output_var_num = 2;
    l0.add.push(GateAdd {
        i_ids: [0],
        o_id: 0,
        coef: C::CircuitField::from(1),
        is_random: false,
        gate_type: 1,
    });
    l0.add.push(GateAdd {
        i_ids: [0],
        o_id: 1,
        coef: C::CircuitField::from(1),
        is_random: false,
        gate_type: 1,
    });
    l0.add.push(GateAdd {
        i_ids: [1],
        o_id: 1,
        coef: C::CircuitField::from(1),
        is_random: false,
        gate_type: 1,
    });
    l0.mul.push(GateMul {
        i_ids: [0, 2],
        o_id: 2,
        coef: C::CircuitField::from(1),
        is_random: false,
        gate_type: 1,
    });
    circuit.layers.push(l0.clone());
    circuit
}

#[test]
fn test_gkr_correctness() {
    test_gkr_correctness_helper::<GF2ExtConfigSha2>(&Config::<GF2ExtConfigSha2>::new(
        GKRScheme::Vanilla,
    ));
    test_gkr_correctness_helper::<GF2ExtConfigKeccak>(&Config::<GF2ExtConfigKeccak>::new(
        GKRScheme::Vanilla,
    ));
    test_gkr_correctness_helper::<M31ExtConfigSha2>(&Config::<M31ExtConfigSha2>::new(
        GKRScheme::Vanilla,
    ));
    test_gkr_correctness_helper::<M31ExtConfigKeccak>(&Config::<M31ExtConfigKeccak>::new(
        GKRScheme::Vanilla,
    ));
    test_gkr_correctness_helper::<BN254ConfigSha2>(&Config::<BN254ConfigSha2>::new(
        GKRScheme::Vanilla,
    ));
    test_gkr_correctness_helper::<BN254ConfigKeccak>(&Config::<BN254ConfigKeccak>::new(
        GKRScheme::Vanilla,
    ));
}

fn test_gkr_correctness_helper<C: GKRConfig>(config: &Config<C>) {
    println!("Config created.");
    let mut circuit_path = KECCAK_M31_CIRCUIT;
    if C::FIELD_TYPE == FieldType::GF2 {
        circuit_path = KECCAK_GF2_CIRCUIT;
    }

    let mut circuit = Circuit::<C>::load_circuit(circuit_path);
    // circuit.layers = circuit.layers[6..7].to_vec(); //  for only evaluate certain layer
    // let mut circuit = gen_simple_circuit(); // for custom circuit
    println!("Circuit loaded.");

    circuit.set_random_input_for_test();

    // for fixed input
    // for i in 0..(1 << circuit.log_input_size()) {
    //     circuit.layers.first_mut().unwrap().input_vals[i] = F::from((i % 3 == 1) as u32);
    // }

    let mut prover = Prover::new(config);
    prover.prepare_mem(&circuit);
    let (claimed_v, proof) = prover.prove(&mut circuit);
    println!("Proof generated. Size: {} bytes", proof.bytes.len());
    // first and last 16 proof u8
    println!("Proof bytes: ");
    proof.bytes.iter().take(16).for_each(|b| print!("{} ", b));
    print!("... ");
    proof
        .bytes
        .iter()
        .rev()
        .take(16)
        .rev()
        .for_each(|b| print!("{} ", b));
    println!();

    println!("Proof hash: ");
    sha2::Sha256::digest(&proof.bytes)
        .iter()
        .for_each(|b| print!("{} ", b));
    println!();

    // Verify
    let verifier = Verifier::new(config);
    println!("Verifier created.");
    assert!(verifier.verify(&mut circuit, &claimed_v, &proof),);
    println!("Correct proof verified.");
    let mut bad_proof = proof.clone();
    let rng = &mut rand::thread_rng();
    let random_idx = rng.gen_range(0..bad_proof.bytes.len());
    let random_change = rng.gen_range(1..256) as u8;
    bad_proof.bytes[random_idx] ^= random_change;

    // Catch the panic and treat it as returning `false`
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        verifier.verify(&mut circuit, &claimed_v, &bad_proof)
    }));

    let final_result = result.unwrap_or_default();

    assert!(!final_result,);
    println!("Bad proof rejected.");
}
