use gpui::{Context, EventEmitter};

use crate::env::AppEnv;
use crate::prefs::{LAUNCH_WRAPPER, MINIMIZE_ON_LAUNCH, PREFER_DEDICATED_GPU, save_setting};
use crate::session::LaunchRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LaunchOptionsEvent {
    Changed,
}

pub struct LaunchOptions {
    env: AppEnv,
    vanilla: bool,
    show_launch_warnings: bool,
    minimize_on_launch: bool,
    prefer_dedicated_gpu: bool,
    launch_wrapper: String,
}

impl EventEmitter<LaunchOptionsEvent> for LaunchOptions {}

impl LaunchOptions {
    pub fn new(env: AppEnv, _cx: &mut Context<Self>) -> Self {
        let minimize_on_launch = env.settings.get(&MINIMIZE_ON_LAUNCH);
        let prefer_dedicated_gpu = env.settings.get(&PREFER_DEDICATED_GPU);
        let launch_wrapper = yp_host::launch::wrapper::read(&env.settings);
        Self {
            env,
            vanilla: false,
            show_launch_warnings: false,
            minimize_on_launch,
            prefer_dedicated_gpu,
            launch_wrapper,
        }
    }

    pub fn vanilla(&self) -> bool {
        self.vanilla
    }

    pub fn show_launch_warnings(&self) -> bool {
        self.show_launch_warnings
    }

    pub fn minimize_on_launch(&self) -> bool {
        self.minimize_on_launch
    }

    pub fn prefer_dedicated_gpu(&self) -> bool {
        self.prefer_dedicated_gpu
    }

    pub fn launch_wrapper(&self) -> &str {
        &self.launch_wrapper
    }

    pub fn request(&self) -> LaunchRequest {
        LaunchRequest {
            vanilla: self.vanilla,
        }
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(LaunchOptionsEvent::Changed);
        cx.notify();
    }

    pub fn set_vanilla(&mut self, vanilla: bool, cx: &mut Context<Self>) {
        if self.vanilla != vanilla {
            self.vanilla = vanilla;
            self.changed(cx);
        }
    }

    pub fn set_show_launch_warnings(&mut self, show: bool, cx: &mut Context<Self>) {
        if self.show_launch_warnings != show {
            self.show_launch_warnings = show;
            self.changed(cx);
        }
    }

    pub fn set_minimize_on_launch(&mut self, minimize: bool, cx: &mut Context<Self>) {
        if self.minimize_on_launch != minimize {
            self.minimize_on_launch = minimize;
            save_setting(&self.env.settings, &MINIMIZE_ON_LAUNCH, &minimize, cx);
            self.changed(cx);
        }
    }

    pub fn set_prefer_dedicated_gpu(&mut self, prefer: bool, cx: &mut Context<Self>) {
        if self.prefer_dedicated_gpu != prefer {
            self.prefer_dedicated_gpu = prefer;
            save_setting(&self.env.settings, &PREFER_DEDICATED_GPU, &prefer, cx);
            self.changed(cx);
        }
    }

    pub fn set_launch_wrapper(&mut self, wrapper: impl Into<String>, cx: &mut Context<Self>) {
        let wrapper = wrapper.into().trim().to_owned();
        if self.launch_wrapper != wrapper {
            save_setting(&self.env.settings, &LAUNCH_WRAPPER, &wrapper, cx);
            self.launch_wrapper = wrapper;
            self.changed(cx);
        }
    }
}
