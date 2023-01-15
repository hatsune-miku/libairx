// A universal writable global variable that can be
// multiply mutably referenced at a time.

// safe得不得了

pub struct GlobalVariables {
    pub skip_next_send: bool,
}

pub static mut GLOBAL: GlobalVariables = GlobalVariables {
    skip_next_send: false,
};
