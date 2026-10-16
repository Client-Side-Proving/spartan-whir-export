//! Export one selected standalone-WHIR schedule for Solidity parity checks.
use std::{fs, path::PathBuf, time::Instant};

use spartan_whir_export::{
    abi_export::{proof_to_abi, statement_to_abi},
    fixed_config_codegen::generate_fixed_config_source_named,
    quintic_fixture::{
        build_quintic_jb100_fixture_with_folding_schedule_and_pow_bits,
        tamper_first_initial_ood_answer_quintic, tamper_first_stir_query_quintic,
    },
    utils::write_abi_file,
};
use whir_p3::parameters::FoldingFactor;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    anyhow::ensure!(args.len() == 8, "usage: export_quintic_schedule <num_variables> <pow_bits> <first_fold> <rest_fold> <log_inv_rate> <initial_reduction> <output_directory>");
    let n = args[1].parse::<usize>()?;
    let pow = args[2].parse::<u32>()?;
    let first = args[3].parse::<usize>()?;
    let rest = args[4].parse::<usize>()?;
    let rate = args[5].parse::<usize>()?;
    let reduction = args[6].parse::<usize>()?;
    let out = PathBuf::from(&args[7]);
    anyhow::ensure!(!out.exists(), "output directory already exists");
    let folding = if first == rest {
        FoldingFactor::Constant(first)
    } else {
        FoldingFactor::ConstantFromSecondRound(first, rest)
    };
    let started = Instant::now();
    let fixture = build_quintic_jb100_fixture_with_folding_schedule_and_pow_bits(
        n, pow, folding, first, rate, reduction,
    )?;
    let generation_seconds = started.elapsed().as_secs_f64();
    fs::create_dir_all(&out)?;
    let label = format!("k{n}_pow{pow}_ff{first}_rest{rest}_lir{rate}_rsv{reduction}");
    let library = format!("QuinticWhirFixedConfig_{label}");
    fs::write(
        out.join(format!("{library}.sol")),
        generate_fixed_config_source_named(&fixture, &library),
    )?;
    write_abi_file(
        &out.join("success_statement.abi"),
        &statement_to_abi(&fixture.statement_points, &fixture.statement_evaluations),
    )?;
    write_abi_file(
        &out.join("success_proof.abi"),
        &proof_to_abi(&fixture.proof)?,
    )?;
    let mut wrong_commitment = fixture.proof.clone();
    wrong_commitment.initial_commitment[0] ^= 1;
    for (name, proof) in [
        ("bad_commitment", wrong_commitment),
        (
            "bad_ood",
            tamper_first_initial_ood_answer_quintic(&fixture.proof)?,
        ),
        ("bad_stir", tamper_first_stir_query_quintic(&fixture.proof)?),
    ] {
        write_abi_file(
            &out.join(format!("{name}_proof.abi")),
            &proof_to_abi(&proof)?,
        )?;
    }
    let report = serde_json::json!({
        "schema": "standalone-whir-schedule-export-v1",
        "label": label,
        "protocol": "standalone Keccak WHIR",
        "whir_p3_revision": env!("SPARTAN_WHIR_EXPORT_WHIR_REVISION"),
        "num_variables": n,
        "pow_bits": pow,
        "first_folding_factor": first,
        "subsequent_folding_factor": rest,
        "log_inv_rate": rate,
        "rs_domain_initial_reduction_factor": reduction,
        "security_bits_requested": 101,
        "native_verification": "passed",
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "rustflags": env!("SPARTAN_WHIR_EXPORT_RUSTFLAGS"),
        "diagnostic_fixture_generation_seconds": generation_seconds,
        "timing_scope": "polynomial and statement preparation, commitment, proving, native verification and trace capture; output serialization excluded"
    });
    fs::write(
        out.join("report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
