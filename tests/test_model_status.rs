use highs_sys::*;

#[test]
fn model_status_constants_match_the_c_api() {
    let pairs = [
        (MODEL_STATUS_NOTSET, kHighsModelStatusNotset),
        (MODEL_STATUS_LOAD_ERROR, kHighsModelStatusLoadError),
        (MODEL_STATUS_MODEL_ERROR, kHighsModelStatusModelError),
        (MODEL_STATUS_PRESOLVE_ERROR, kHighsModelStatusPresolveError),
        (MODEL_STATUS_SOLVE_ERROR, kHighsModelStatusSolveError),
        (
            MODEL_STATUS_POSTSOLVE_ERROR,
            kHighsModelStatusPostsolveError,
        ),
        (MODEL_STATUS_MODEL_EMPTY, kHighsModelStatusModelEmpty),
        (MODEL_STATUS_OPTIMAL, kHighsModelStatusOptimal),
        (MODEL_STATUS_INFEASIBLE, kHighsModelStatusInfeasible),
        (
            MODEL_STATUS_UNBOUNDED_OR_INFEASIBLE,
            kHighsModelStatusUnboundedOrInfeasible,
        ),
        (MODEL_STATUS_UNBOUNDED, kHighsModelStatusUnbounded),
        (
            MODEL_STATUS_OBJECTIVE_BOUND,
            kHighsModelStatusObjectiveBound,
        ),
        (
            MODEL_STATUS_OBJECTIVE_TARGET,
            kHighsModelStatusObjectiveTarget,
        ),
        (MODEL_STATUS_REACHED_TIME_LIMIT, kHighsModelStatusTimeLimit),
        (
            MODEL_STATUS_REACHED_ITERATION_LIMIT,
            kHighsModelStatusIterationLimit,
        ),
        (MODEL_STATUS_UNKNOWN, kHighsModelStatusUnknown),
        (
            MODEL_STATUS_REACHED_SOLUTION_LIMIT,
            kHighsModelStatusSolutionLimit,
        ),
        (MODEL_STATUS_REACHED_INTERRUPT, kHighsModelStatusInterrupt),
        (
            MODEL_STATUS_REACHED_MEMORY_LIMIT,
            kHighsModelStatusMemoryLimit,
        ),
        (
            MODEL_STATUS_HIGHS_INTERRUPT,
            kHighsModelStatusHighsInterrupt,
        ),
    ];
    for (ours, c_api) in pairs {
        assert_eq!(ours, c_api);
    }
    assert_eq!(MODEL_STATUS_MAX, kHighsModelStatusHighsInterrupt);
}
