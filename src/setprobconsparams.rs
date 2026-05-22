// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Process-global perturb seed mirroring C++'s `opt(perturb)` / `optset_perturb`.
/// Set once at CLI dispatch; consumed by `init_probcons` so all entry points
/// (super4/5/6/7, eacluster, eadistmx, hmmdump, …) apply the same perturbation
/// as the matching C++ command.
pub(crate) static PERTURB_SEED: std::sync::Mutex<Option<uint>> = std::sync::Mutex::new(None);

/// Sets the global perturb seed used by `init_probcons`. `Some(0)` and `None`
/// both disable perturbation, matching C++ `optset_perturb && opt(perturb) > 0`.
#[track_caller]
pub fn set_perturb_seed(seed: Option<uint>) {
    *PERTURB_SEED.lock().unwrap() = seed;
}

/// Process-global `-guidetreeout` path mirroring C++ `opt(guidetreeout)` /
/// `optset_guidetreeout` (consulted in mpcflat.cpp:196). When set, MPCFlat
/// writes the freshly computed guide tree and exits before alignment.
pub(crate) static GUIDETREEOUT_PATH: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

#[track_caller]
pub fn set_guidetreeout_path(path: Option<String>) {
    *GUIDETREEOUT_PATH.lock().unwrap() = path;
}

#[track_caller]
pub fn guidetreeout_path() -> Option<String> {
    GUIDETREEOUT_PATH.lock().unwrap().clone()
}

/// Process-global `-hmmin` path: when set, `init_probcons` loads HMM params
/// from this file instead of `hmm_params_from_defaults`. Mirrors C++
/// setprobconsparams.cpp:13-19.
pub(crate) static HMMIN_PATH: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

#[track_caller]
pub fn set_hmmin_path(path: Option<String>) {
    *HMMIN_PATH.lock().unwrap() = path;
}

/// Process-global `-hmmout` path: when set, `init_probcons` writes the (
/// possibly perturbed) HMM params to this file before publishing them to
/// the global pair-HMM tables. Mirrors C++ setprobconsparams.cpp:37-38.
pub(crate) static HMMOUT_PATH: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

#[track_caller]
pub fn set_hmmout_path(path: Option<String>) {
    *HMMOUT_PATH.lock().unwrap() = path;
}

/// Process-global `-randomchaintree` flag: when true, MPCFlat builds a
/// caterpillar guide tree (mpc_flat_calc_guide_tree_random_chain) instead of
/// running UPGMA. Mirrors C++ mpcflat.cpp:185-189.
pub(crate) static RANDOMCHAINTREE_ENABLED: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[track_caller]
pub fn set_randomchaintree_enabled(enabled: bool) {
    *RANDOMCHAINTREE_ENABLED.lock().unwrap() = enabled;
}

#[track_caller]
pub fn randomchaintree_enabled() -> bool {
    *RANDOMCHAINTREE_ENABLED.lock().unwrap()
}

/// One-shot initialization of ProbCons HMM parameters from defaults for the active alphabet.
#[track_caller]
pub fn init_probcons() -> bool {
    {
        let mut done = INIT_PROBCONS_DONE.lock().unwrap();
        if *done {
            return false;
        }
        *done = true;
    }

    let alpha = ALPHA_STATE.lock().unwrap().alpha;
    assert!(alpha == ALPHA::ALPHA_Amino || alpha == ALPHA::ALPHA_Nucleo);
    let nucleo = alpha == ALPHA::ALPHA_Nucleo;
    // Mirror C++ setprobconsparams.cpp:13-19: `-hmmin` overrides built-in
    // defaults.
    let mut hp = if let Some(path) = HMMIN_PATH.lock().unwrap().clone() {
        let _ = progress_log(&format!("Reading HMM parameters from {path}\n"));
        hmm_params_from_file(&path)
    } else {
        hmm_params_from_defaults(nucleo)
    };
    // Mirror C++ setprobconsparams.cpp:26-35: if -perturb is set, ResetRand
    // and PerturbProbs (which itself ResetRands again) before publishing
    // to the global pair-HMM tables.
    if let Some(seed) = *PERTURB_SEED.lock().unwrap() {
        if seed > 0 {
            reset_rand(seed);
            hmm_params_perturb_probs(&mut hp, seed);
        }
    }
    // Mirror C++ setprobconsparams.cpp:37-38: optional dump after perturb.
    if let Some(path) = HMMOUT_PATH.lock().unwrap().clone() {
        let _ = hmm_params_to_file(&hp, &path);
    }
    hmm_params_to_pair_hmm(&hp);
    true
}
