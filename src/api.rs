use crate::*;
use std::path::PathBuf;
use std::sync::Mutex;

static API_LOCK: Mutex<()> = Mutex::new(());

/// High-level library entry point for MUSCLE alignment.
///
/// The translated core preserves several C++ process-global settings
/// (`-threads`, alphabet state, HMM parameters, progress flags). The builder
/// serializes calls so library users can call the API safely without racing
/// those globals.
#[derive(Clone, Debug, Default)]
pub struct Muscle;

/// Builder for in-process MUSCLE alignment.
#[derive(Clone, Debug)]
pub struct MuscleBuilder {
    input: InputSource,
    threads: Option<uint>,
    input_order: bool,
    consistency_iters: Option<uint>,
    refine_iters: Option<uint>,
    perturb_seed: Option<uint>,
    tree_perm: Option<TREEPERM>,
    quiet: bool,
}

#[derive(Clone, Debug, Default)]
enum InputSource {
    #[default]
    None,
    Fasta(String),
    File(PathBuf),
}

/// Result of a high-level alignment run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alignment {
    fasta: String,
}

/// Error returned by the high-level library API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MuscleError {
    message: String,
}

impl Muscle {
    /// Start a builder-style in-process alignment.
    pub fn builder() -> MuscleBuilder {
        MuscleBuilder::default()
    }
}

impl Default for MuscleBuilder {
    fn default() -> Self {
        Self {
            input: InputSource::None,
            threads: None,
            input_order: false,
            consistency_iters: None,
            refine_iters: None,
            perturb_seed: None,
            tree_perm: None,
            quiet: true,
        }
    }
}

impl MuscleBuilder {
    /// Use FASTA text as the input sequence set.
    pub fn input_fasta(mut self, fasta: impl Into<String>) -> Self {
        self.input = InputSource::Fasta(fasta.into());
        self
    }

    /// Use a FASTA file as the input sequence set.
    pub fn input_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.input = InputSource::File(path.into());
        self
    }

    /// Set the worker thread count used by translated parallel paths.
    pub fn threads(mut self, threads: uint) -> Self {
        self.threads = Some(threads.max(1));
        self
    }

    /// Preserve input order in the final alignment where the core algorithm supports it.
    pub fn input_order(mut self, input_order: bool) -> Self {
        self.input_order = input_order;
        self
    }

    /// Override the number of consistency iterations.
    pub fn consistency_iters(mut self, iters: uint) -> Self {
        self.consistency_iters = Some(iters);
        self
    }

    /// Override the number of refinement iterations.
    pub fn refine_iters(mut self, iters: uint) -> Self {
        self.refine_iters = Some(iters);
        self
    }

    /// Set the HMM perturbation seed.
    pub fn perturb_seed(mut self, seed: uint) -> Self {
        self.perturb_seed = Some(seed);
        self
    }

    /// Set the guide-tree permutation.
    pub fn tree_perm(mut self, perm: TREEPERM) -> Self {
        self.tree_perm = Some(perm);
        self
    }

    /// Enable or disable translated progress output while the alignment runs.
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Run the alignment and return aligned FASTA text.
    pub fn build(self) -> Result<Alignment, MuscleError> {
        let _guard = API_LOCK.lock().expect("MUSCLE API lock poisoned");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.build_inner()));
        match result {
            Ok(result) => result,
            Err(payload) => Err(MuscleError::new(panic_message(payload))),
        }
    }

    fn build_inner(self) -> Result<Alignment, MuscleError> {
        let mut input_seqs = match &self.input {
            InputSource::None => return Err(MuscleError::new("missing input FASTA")),
            InputSource::Fasta(fasta) => multi_sequence_from_fasta_text(fasta)?,
            InputSource::File(path) => {
                let mut ms = MultiSequence::default();
                let path = path
                    .to_str()
                    .ok_or_else(|| MuscleError::new("input path is not valid UTF-8"))?;
                multi_sequence_load_mfa_l8(&mut ms, path, true);
                ms
            }
        };
        let fasta = run_alignment(&mut input_seqs, &self)?;
        Ok(Alignment { fasta })
    }
}

impl Alignment {
    /// Return the aligned FASTA text.
    pub fn as_fasta(&self) -> &str {
        &self.fasta
    }

    /// Consume the result and return the aligned FASTA text.
    pub fn into_fasta(self) -> String {
        self.fasta
    }
}

impl MuscleError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for MuscleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MuscleError {}

fn run_alignment(
    input_seqs: &mut MultiSequence,
    builder: &MuscleBuilder,
) -> Result<String, MuscleError> {
    set_die_process_exit_enabled(false);
    set_quiet(builder.quiet);
    set_perturb_seed(builder.perturb_seed);
    reset_cmd_opt_state();
    {
        let mut argv = G_ARGV.lock().unwrap();
        argv.clear();
        argv.push("muscle".to_string());
        argv.push("-align".to_string());
        if let Some(threads) = builder.threads {
            argv.push("-threads".to_string());
            argv.push(threads.to_string());
        }
    }

    set_global_input_ms(input_seqs);
    if input_seqs.seqs.is_empty() {
        return Ok(String::new());
    }

    let is_nucleo = multi_sequence_guess_is_nucleo(input_seqs);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let mut m = MPCFlat::default();
    if let Some(consistency_iter_count) = builder.consistency_iters {
        m.consistency_iter_count = consistency_iter_count;
    }
    if let Some(refine_iter_count) = builder.refine_iters {
        m.refine_iter_count = refine_iter_count;
    }

    let (_hp, fasta) = align(
        &mut m,
        input_seqs,
        builder.perturb_seed.unwrap_or(0),
        builder.tree_perm.unwrap_or(TREEPERM::TP_None),
        false,
        true,
        |hp| {
            hmm_params_cmd_line_update(hp, None, None, None, None, None, None);
        },
        |mpc, input_seqs| run_mpc_default(mpc, input_seqs, builder.input_order),
    );
    Ok(fasta)
}

fn multi_sequence_from_fasta_text(fasta: &str) -> Result<MultiSequence, MuscleError> {
    let mut ms = MultiSequence::default();
    let mut label: Option<String> = None;
    let mut seq = Vec::<char>::new();

    for line in fasta.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(next_label) = line.strip_prefix('>') {
            if let Some(prev_label) = label.replace(next_label.to_string()) {
                ms.seqs.push(Sequence {
                    label: prev_label,
                    char_vec: std::mem::take(&mut seq),
                });
                ms.owners.push(true);
            }
        } else {
            if label.is_none() {
                return Err(MuscleError::new("expected FASTA header starting with '>'"));
            }
            for mut ch in line.bytes() {
                if ch.is_ascii_whitespace() || ch == b'-' || ch == b'.' {
                    continue;
                }
                ch = ch.to_ascii_uppercase();
                seq.push(ch as char);
            }
        }
    }

    if let Some(label) = label {
        ms.seqs.push(Sequence {
            label,
            char_vec: seq,
        });
        ms.owners.push(true);
    }
    Ok(ms)
}

fn run_mpc_default(
    mpc: &mut MPCFlat,
    input_seqs: &MultiSequence,
    input_order: bool,
) -> MultiSequence {
    mpc_flat_run(
        mpc,
        input_seqs,
        input_order,
        |mpc| mpc_flat_calc_posteriors(mpc),
        |mpc| {
            mpc_flat_calc_guide_tree(mpc, |u, guide_tree| upgma5_run_l75(u, "biased", guide_tree))
        },
        |mpc| mpc_flat_consistency_parallel_pairs(mpc),
        |mpc| {
            mpc_flat_progressive_align(mpc, |mpc, msa1, msa2| {
                mpc_flat_align_alns(mpc, msa1, msa2).0
            })
        },
        |mpc| {
            mpc_flat_refine(
                mpc,
                || libc_rand(),
                |mpc, msa1, msa2| mpc_flat_align_alns(mpc, msa1, msa2).0,
            )
        },
    );
    mpc.msa
        .as_ref()
        .expect("library alignment missing final MSA")
        .clone()
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else {
        "alignment failed".to_string()
    }
}
