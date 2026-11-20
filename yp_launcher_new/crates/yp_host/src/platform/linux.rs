use std::sync::Arc;

use yp_core::{HostEnv, ProcessRunner};

use super::wine::WineAdapter;
use crate::wine::WineContext;

pub fn linux_adapter(host: HostEnv, runner: Arc<dyn ProcessRunner>) -> WineAdapter {
    linux_adapter_with(WineContext::new(host), runner)
}

pub fn linux_adapter_with(ctx: WineContext, runner: Arc<dyn ProcessRunner>) -> WineAdapter {
    WineAdapter::new(ctx, runner)
}
