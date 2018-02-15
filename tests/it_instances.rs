extern crate ceres;

#[test]
fn noop_okay() {
    let result = ceres::noop();

    assert!(result.is_ok());
}
