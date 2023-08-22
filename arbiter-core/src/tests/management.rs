use super::*;

#[test_log::test]
fn add_environment() {
    let mut manager = Manager::new();
    let params = EnvironmentParameters {
        block_rate: 1.0,
        seed: 1,
    };
    manager.add_environment(TEST_ENV_LABEL, params).unwrap();
    assert!(manager
        .environments
        .contains_key(&TEST_ENV_LABEL.to_string()));
    assert_eq!(
        manager
            .environments
            .get(&TEST_ENV_LABEL.to_string())
            .unwrap()
            .state
            .load(std::sync::atomic::Ordering::Relaxed),
        State::Initialization
    );
}

#[test_log::test]
fn run_environment() {
    let mut manager = Manager::new();
    let params = EnvironmentParameters {
        block_rate: 1.0,
        seed: 1,
    };
    manager.add_environment(TEST_ENV_LABEL, params).unwrap();
    manager.start_environment(TEST_ENV_LABEL).unwrap();
    assert_eq!(
        manager
            .environments
            .get(&TEST_ENV_LABEL.to_string())
            .unwrap()
            .state
            .load(std::sync::atomic::Ordering::Relaxed),
        State::Running
    );
}

#[test_log::test]
fn pause_environment() {
    let mut manager = Manager::new();
    let params = EnvironmentParameters {
        block_rate: 1.0,
        seed: 1,
    };
    manager.add_environment(TEST_ENV_LABEL, params).unwrap();
    manager.start_environment(TEST_ENV_LABEL).unwrap();
    manager.pause_environment(TEST_ENV_LABEL).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert_eq!(
        manager
            .environments
            .get(&TEST_ENV_LABEL.to_string())
            .unwrap()
            .state
            .load(std::sync::atomic::Ordering::Relaxed),
        State::Paused
    );
    manager.start_environment(TEST_ENV_LABEL).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert_eq!(
        manager
            .environments
            .get(&TEST_ENV_LABEL.to_string())
            .unwrap()
            .state
            .load(std::sync::atomic::Ordering::Relaxed),
        State::Running
    );
}

#[test_log::test]
fn stop_environment() {
    let mut manager = Manager::new();
    let params = EnvironmentParameters {
        block_rate: 1.0,
        seed: 1,
    };
    manager.add_environment(TEST_ENV_LABEL, params).unwrap();
    manager.start_environment(TEST_ENV_LABEL).unwrap();
    manager.stop_environment(TEST_ENV_LABEL).unwrap();
    assert_eq!(
        manager
            .environments
            .get(&TEST_ENV_LABEL.to_string())
            .unwrap()
            .state
            .load(std::sync::atomic::Ordering::Relaxed),
        State::Stopped
    );
}