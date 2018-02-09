extern crate ceres;
extern crate env_logger;

fn main() {
    let _ = env_logger::try_init();

    ceres::run();
}

