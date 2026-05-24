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
    run_muscle3: FRunMuscle3,
) -> Vec<String>
where
    FRunMuscle3: Fn(&mut Muscle3, &M3AlnParams, &MultiSequence) -> MultiSequence + Sync,
{
    let names = read_strings_from_file(batch_file_name);
    let mut input_dir = input_dir.to_string();
    let mut output_dir = output_dir.to_string();
    dirize(&mut input_dir);
    dirize(&mut output_dir);
    with_quiet(true, || {
        let thread_count = get_requested_thread_count().max(1);
        let n = names.len() as uint;
        let counter = std::sync::Mutex::new(0 as uint);
        let output_file_names = std::sync::Mutex::new(vec![String::new(); names.len()]);

        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for thread_index in 0..thread_count {
                let start = (n * thread_index) / thread_count;
                let end = (n * (thread_index + 1)) / thread_count;
                let names = &names;
                let input_dir = &input_dir;
                let output_dir = &output_dir;
                let counter = &counter;
                let output_file_names = &output_file_names;
                let run_muscle3 = &run_muscle3;

                handles.push(scope.spawn(move || {
                    let mut m3 = Muscle3::default();
                    for i in start..end {
                        let name = &names[i as usize];
                        let fasta_file_name = format!("{input_dir}{name}");
                        {
                            let mut counter = counter.lock().unwrap();
                            let _ = progress_step_unquiet(
                                *counter,
                                n,
                                &format!("Aligning {n} sets {fasta_file_name}"),
                            );
                            *counter += 1;
                        }

                        let mut ms = MultiSequence::default();
                        multi_sequence_load_mfa_l8(&mut ms, &fasta_file_name, true);

                        let output_file_name = format!("{output_dir}{name}");
                        let final_msa = run_muscle3(&mut m3, ap, &ms);
                        multi_sequence_write_mfa(&final_msa, &output_file_name);
                        output_file_names.lock().unwrap()[i as usize] = output_file_name;
                    }
                }));
            }
            for handle in handles {
                handle.join().unwrap();
            }
        });

        output_file_names.lock().unwrap().clone()
    })
}
