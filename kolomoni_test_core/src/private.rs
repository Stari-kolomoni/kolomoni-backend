use std::sync::{LazyLock, Mutex};

pub static GLOBAL_TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
