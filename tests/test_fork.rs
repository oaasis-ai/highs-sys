use std::convert::TryInto;
use std::ffi::CString;
use std::ptr::{null, null_mut};

use highs_sys::*;

fn c(n: usize) -> HighsInt {
    n.try_into().unwrap()
}

fn option(name: &str) -> CString {
    CString::new(name).unwrap()
}

/// The fork's MIP options, which upstream HiGHS does not have. A submodule bump
/// that drops them would make `Highs_setOptionValue` reject their names.
#[test]
fn fork_mip_options_are_available() {
    let options = [
        ("mip_max_stall_time", kHighsOptionTypeDouble),
        ("mip_max_lp_iterations", kHighsOptionTypeInt),
        ("mip_root_heuristics_first", kHighsOptionTypeBool),
        ("mip_max_root_sep_rounds", kHighsOptionTypeInt),
    ];
    unsafe {
        let highs = Highs_create();
        for (name, expected_type) in options {
            let mut option_type: HighsInt = -1;
            let status = Highs_getOptionType(highs, option(name).as_ptr(), &mut option_type);
            assert_eq!(status, STATUS_OK, "{name} is not a HiGHS option");
            assert_eq!(option_type, expected_type, "{name} has the wrong type");
        }
        Highs_destroy(highs);
    }
}

/// Solve `flugpl.mps` plus an empty column of cost 1 on [0, 10], optionally from
/// a start solution, and return the model status, the reported objective, c^T x
/// for the returned x, and x.
unsafe fn solve_flugpl_with_empty_col(start: Option<&[f64]>) -> (HighsInt, f64, f64, Vec<f64>) {
    let model_file = format!(
        "{}/HiGHS/check/instances/flugpl.mps",
        env!("CARGO_MANIFEST_DIR")
    );
    let highs = Highs_create();
    Highs_setBoolOptionValue(highs, option("output_flag").as_ptr(), 0);
    assert_ne!(
        Highs_readModel(highs, CString::new(model_file).unwrap().as_ptr()),
        STATUS_ERROR
    );
    assert_eq!(
        Highs_addCol(highs, 1.0, 0.0, 10.0, 0, null(), null()),
        STATUS_OK
    );
    if let Some(col_value) = start {
        assert_eq!(
            Highs_setSolution(highs, col_value.as_ptr(), null(), null(), null()),
            STATUS_OK
        );
    }
    assert_eq!(Highs_run(highs), STATUS_OK);

    let num_col = Highs_getNumCol(highs) as usize;
    let num_nz = Highs_getNumNz(highs) as usize;
    let mut costs = vec![0.0; num_col];
    let (mut lower, mut upper) = (vec![0.0; num_col], vec![0.0; num_col]);
    let (mut start_, mut index, mut value) = (vec![0; num_col], vec![0; num_nz], vec![0.0; num_nz]);
    let (mut got_col, mut got_nz): (HighsInt, HighsInt) = (0, 0);
    Highs_getColsByRange(
        highs,
        0,
        c(num_col - 1),
        &mut got_col,
        costs.as_mut_ptr(),
        lower.as_mut_ptr(),
        upper.as_mut_ptr(),
        &mut got_nz,
        start_.as_mut_ptr(),
        index.as_mut_ptr(),
        value.as_mut_ptr(),
    );
    let mut offset = 0.0;
    Highs_getObjectiveOffset(highs, &mut offset);
    let mut col_value = vec![0.0; num_col];
    Highs_getSolution(
        highs,
        col_value.as_mut_ptr(),
        null_mut(),
        null_mut(),
        null_mut(),
    );
    let objective = offset
        + costs
            .iter()
            .zip(&col_value)
            .map(|(cost, x)| cost * x)
            .sum::<f64>();
    let result = (
        Highs_getModelStatus(highs),
        Highs_getObjectiveValue(highs),
        objective,
        col_value,
    );
    Highs_destroy(highs);
    result
}

/// A MIP start whose empty positive-cost column sits above its lower bound costs
/// more than its image after presolve, which removes that column at its lower
/// bound. HiGHS v1.16.0 returned such a start as Optimal with the objective of
/// the image, which its values do not attain (ERGO-Code/HiGHS#3634).
#[test]
fn mip_start_objective_matches_returned_values() {
    unsafe {
        let (status, optimum, _, mut start) = solve_flugpl_with_empty_col(None);
        assert_eq!(status, MODEL_STATUS_OPTIMAL);
        *start.last_mut().unwrap() = 5.0;

        let (status, reported, objective, _) = solve_flugpl_with_empty_col(Some(&start));
        assert_eq!(status, MODEL_STATUS_OPTIMAL);
        let tolerance = 1e-6 * (1.0 + objective.abs());
        assert!(
            (reported - objective).abs() <= tolerance,
            "reported objective {} but the returned solution costs {}",
            reported,
            objective
        );
        assert!(
            (reported - optimum).abs() <= tolerance,
            "reported objective {} but the optimum is {}",
            reported,
            optimum
        );
    }
}
