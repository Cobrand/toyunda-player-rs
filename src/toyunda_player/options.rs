pub struct ToyundaOptions {
    edit_mode:bool,
    display:bool,
    console_info:bool
}

impl Default for ToyundaOptions {
    fn default() -> ToyundaOptions {
        ToyundaOptions {
            edit_mode:false,
            display:true,
            console_info:true
        }
    }
}
