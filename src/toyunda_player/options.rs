pub struct ToyundaOptions {
    edit_mode:bool,
    display:bool
}

impl Default for ToyundaOptions {
    fn default() -> ToyundaOptions {
        ToyundaOptions {
            edit_mode:false,
            display:true
        }
    }
}
