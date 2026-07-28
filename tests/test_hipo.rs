#![cfg(feature = "hipo")]
//! Guards that the `hipo` feature produces a HiGHS build whose HiPO solver is
//! actually available. HiGHS 1.15+ keeps HiPO (and its AMD/BLAS/METIS/RCM
//! dependencies) in an "extras" component and rejects the `hipo` solver options
//! at set-time when that component is not linked in — so a mislinked build
//! fails here, at the highs-sys layer, instead of three repos downstream.

use highs_sys::*;
use std::convert::TryInto;
use std::ffi::CString;
use std::ptr::null;

fn c(n: usize) -> HighsInt {
    n.try_into().unwrap()
}

#[test]
fn hipo_solver_is_available_and_solves() {
    unsafe {
        let highs = Highs_create();

        let output_flag = CString::new("output_flag").unwrap();
        Highs_setBoolOptionValue(highs, output_flag.as_ptr(), 0);

        // The availability gate: HiGHS accepts "hipo" only when the extras are
        // linked. A non-OK status here is the exact failure the version bump hit.
        let solver = CString::new("solver").unwrap();
        let hipo = CString::new("hipo").unwrap();
        let status = Highs_setStringOptionValue(highs, solver.as_ptr(), hipo.as_ptr());
        assert_eq!(
            status, STATUS_OK,
            "HiPO must be available: the extras (AMD/BLAS/METIS/RCM) must be statically linked"
        );

        // Solve a small LP through HiPO and confirm it reaches optimality.
        // Max f = 2x_0 + 3x_1  s.t.  x_1 <= 6; 10 <= x_0 + 2x_1 <= 14; 8 <= 2x_0 + x_1
        // 0 <= x_0 <= 3; 1 <= x_1
        let inf = Highs_getInfinity(highs);
        let colcost: &mut [f64] = &mut [2.0, 3.0];
        let collower: &mut [f64] = &mut [0.0, 1.0];
        let colupper: &mut [f64] = &mut [3.0, inf];
        let rowlower: &mut [f64] = &mut [-inf, 10.0, 8.0];
        let rowupper: &mut [f64] = &mut [6.0, 14.0, inf];
        let arstart: &mut [HighsInt] = &mut [0, 1, 3];
        let arindex: &mut [HighsInt] = &mut [1, 0, 1, 0, 1];
        let arvalue: &mut [f64] = &mut [1.0, 1.0, 2.0, 2.0, 1.0];

        let success = Highs_addCols(
            highs,
            c(2),
            colcost.as_mut_ptr(),
            collower.as_mut_ptr(),
            colupper.as_mut_ptr(),
            0,
            null(),
            null(),
            null(),
        );
        assert_eq!(STATUS_OK, success, "addCols");
        let success = Highs_addRows(
            highs,
            c(3),
            rowlower.as_mut_ptr(),
            rowupper.as_mut_ptr(),
            c(5),
            arstart.as_mut_ptr(),
            arindex.as_mut_ptr(),
            arvalue.as_mut_ptr(),
        );
        assert_eq!(STATUS_OK, success, "addRows");

        let success = Highs_changeObjectiveSense(highs, OBJECTIVE_SENSE_MAXIMIZE);
        assert_eq!(success, STATUS_OK);

        let status = Highs_run(highs);
        assert_eq!(status, STATUS_OK, "Highs_run with HiPO");
        let model_status = Highs_getModelStatus(highs);
        assert_eq!(
            model_status, MODEL_STATUS_OPTIMAL,
            "HiPO must solve the LP to optimality"
        );

        let mut objective: f64 = 0.0;
        let info = CString::new("objective_function_value").unwrap();
        Highs_getHighsDoubleInfoValue(highs, info.as_ptr(), (&mut objective) as *mut f64);
        assert!(
            (objective - (2.0 * 3.0 + 3.0 * 5.5)).abs() < 1e-6,
            "unexpected HiPO objective {objective}"
        );

        Highs_destroy(highs);
    }
}
