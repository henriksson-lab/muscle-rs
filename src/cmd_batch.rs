// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Runs MUSCLE3 on each FASTA listed in a batch file, writing the aligned MSA
/// for each input to the output directory. Returns the list of output paths.
#[track_caller]
pub fn cmd_batch<FRunMuscle3>(
    batch_file_name: &str,
    input_dir: &str,
    output_dir: &str,
    ap: &M3AlnParams,
    mut run_muscle3: FRunMuscle3,
) -> Vec<String>
where
    FRunMuscle3: FnMut(&mut Muscle3, &M3AlnParams, &MultiSequence) -> MultiSequence,
{
    let names = read_strings_from_file(batch_file_name);
    let mut input_dir = input_dir.to_string();
    let mut output_dir = output_dir.to_string();
    dirize(&mut input_dir);
    dirize(&mut output_dir);

    let thread_count = get_requested_thread_count();
    let mut m3s = Vec::<Muscle3>::new();
    for _ in 0..thread_count {
        m3s.push(Muscle3::default());
    }
    if m3s.is_empty() {
        m3s.push(Muscle3::default());
    }

    let mut output_file_names = Vec::<String>::new();
    for name in &names {
        let fasta_file_name = format!("{input_dir}{name}");
        let mut ms = MultiSequence::default();
        multi_sequence_load_mfa_l8(&mut ms, &fasta_file_name, true);

        let output_file_name = format!("{output_dir}{name}");
        let final_msa = run_muscle3(&mut m3s[0], ap, &ms);
        multi_sequence_write_mfa(&final_msa, &output_file_name);
        output_file_names.push(output_file_name);
    }
    output_file_names
}
